//====================================================================

use std::collections::{HashSet, VecDeque};

use common::{Size, Transform};
use engine::{
    scene::Scene,
    tools::{Input, KeyCode},
    StateInner,
};
use rand::Rng;
use renderer::pipelines::ui3d_pipeline::Ui3d;

use crate::{
    characters::{CharacterId, CharacterManager},
    scenery::Scenery,
};

//====================================================================

pub struct BattleScene {
    scenery: Scenery,

    character_manager: CharacterManager,
    battle_manager: BattleManager,
}

impl Scene for BattleScene {
    fn new(state: &mut StateInner) -> Self {
        let mut character_manager = CharacterManager::new(state);
        let mut battle_manager = BattleManager::default();

        battle_manager.add_friendly(character_manager.spawn("Friendly Character"));
        battle_manager.add_enemy(character_manager.spawn("Enemy Character"));

        Self {
            scenery: Scenery::new(state),
            character_manager,
            battle_manager,
        }
    }

    fn resize(&mut self, state: &mut StateInner, new_size: Size<u32>) {
        state
            .renderer
            .camera
            .set_aspect(new_size.width as f32, new_size.height as f32);
    }

    fn update(&mut self, state: &mut StateInner) {
        crate::camera::move_camera(state);
        self.scenery.render(state);

        self.battle_manager.tick(state, &mut self.character_manager);

        self.character_manager.update(state);
        self.character_manager.render(state);
    }
}

//====================================================================

#[derive(Debug, Default)]
struct BattleManager {
    state: BattleState,

    friendly: HashSet<CharacterId>,
    enemy: HashSet<CharacterId>,

    turn_order: VecDeque<CharacterId>,
}

#[derive(Debug, Default)]
enum BattleState {
    #[default]
    Initializing,
    StartingRound,
    WaitingForInput(UiMenus),
    ProcessingCpu,
}

impl BattleManager {
    fn add_friendly(&mut self, id: CharacterId) {
        self.friendly.insert(id);
    }

    fn add_enemy(&mut self, id: CharacterId) {
        self.enemy.insert(id);
    }

    fn position_characters(&self, characters: &mut CharacterManager) {
        self.friendly.iter().enumerate().for_each(|(index, id)| {
            let character = characters.character_mut(id).unwrap();

            character.transform.translation = glam::vec3(index as f32 * 100., 0., -100.);
            character.transform.rotation = glam::Quat::from_rotation_y(0.);
        });

        self.enemy.iter().enumerate().for_each(|(index, id)| {
            let character = characters.character_mut(id).unwrap();

            character.transform.translation = glam::vec3(index as f32 * 100., 0., 100.);
            character.transform.rotation = glam::Quat::from_rotation_y(std::f32::consts::PI);
        });
    }
}

impl BattleManager {
    fn tick(&mut self, state: &mut StateInner, characters: &mut CharacterManager) {
        let mut next_turn = false;

        match &mut self.state {
            BattleState::Initializing => {
                self.position_characters(characters);

                self.state = BattleState::StartingRound;
            }

            BattleState::StartingRound => {
                self.start_round(characters);

                let current_character = characters
                    .character(self.turn_order.get(0).unwrap())
                    .unwrap();

                log::info!("next character = {}", current_character.name);

                self.state =
                    BattleState::WaitingForInput(UiMenus::new(state, &current_character.transform));
            }

            BattleState::WaitingForInput(ui_menus) => {
                match ui_menus.tick(state) {
                    UiMenuOutput::None => {}
                    UiMenuOutput::SkipTurn => next_turn = true,
                }
                ui_menus.render(state);
            }

            BattleState::ProcessingCpu => todo!(),
        }

        if next_turn {
            self.next_turn(state, characters);
        }
    }

    fn start_round(&mut self, characters: &mut CharacterManager) {
        log::info!("------Starting new round------");
        self.turn_order.clear();

        let mut weight = 0;
        let mut character_weights = Vec::new();

        self.friendly
            .iter()
            .chain(self.enemy.iter())
            .for_each(|id| {
                let data = characters.character(id).unwrap();

                weight += data.stats.speed;
                character_weights.push((data.stats.speed, *id));
            });

        log::debug!(
            "Total weight = {}, Character Weightings = {:?}",
            weight,
            character_weights
        );

        let mut rng = rand::thread_rng();

        while !character_weights.is_empty() {
            if character_weights.len() == 1 {
                self.turn_order.push_back(character_weights[0].1);
                break;
            }

            let roll = rng.gen_range(0..weight);
            let mut acc = 0;

            let character = character_weights
                .iter()
                .enumerate()
                .find(|(_, (weight, _))| match (acc + weight) > roll {
                    true => true,
                    false => {
                        acc += weight;
                        false
                    }
                })
                .unwrap();

            self.turn_order.push_back(character.1 .1);
            weight -= character.1 .0;
            character_weights.remove(character.0);
        }

        log::debug!(
            "Turn order = {:?}",
            self.turn_order
                .iter()
                .map(|id| &characters.character(id).unwrap().name)
                .collect::<Vec<_>>()
        );
    }

    fn next_turn(&mut self, state: &mut StateInner, characters: &mut CharacterManager) {
        if self.turn_order.len() <= 1 {
            self.state = BattleState::StartingRound;
            return;
        }

        self.turn_order.pop_front();

        let current_character = characters.character(&self.turn_order[0]).unwrap();
        log::info!("next character = {}", current_character.name);

        self.state =
            BattleState::WaitingForInput(UiMenus::new(state, &current_character.transform));
    }
}

//====================================================================

#[derive(Debug)]
struct UiMenus {
    action_menu: Ui3d,
    target_menu: Option<Ui3d>,
}

enum UiMenuAction {
    Back,
    Forward,
    Select,
}

enum UiMenuOutput {
    None,
    SkipTurn,
}

impl UiMenus {
    pub fn new(state: &mut StateInner, character_transform: &Transform) -> Self {
        let menu_pos = character_transform.translation + character_transform.right() * 50.;

        let action_menu = state
            .renderer
            .create_ui(vec!["One".into(), "Two".into(), "Three".into()], menu_pos);

        Self {
            action_menu,
            target_menu: None,
        }
    }

    fn tick(&mut self, state: &mut StateInner) -> UiMenuOutput {
        // Process target menu if available
        if let Some(target) = &mut self.target_menu {
            if let Some(action) = Self::process_input(&mut state.keys, target) {
                match action {
                    UiMenuAction::Forward | UiMenuAction::Select => return UiMenuOutput::SkipTurn,
                    UiMenuAction::Back => {
                        self.target_menu = None;
                    }
                }
            }
        }
        // Process Actions menu
        else {
            if let Some(action) = Self::process_input(&mut state.keys, &mut self.action_menu) {
                match action {
                    UiMenuAction::Forward | UiMenuAction::Select => {
                        self.target_menu = state
                            .renderer
                            .create_ui(vec!["You".into(), "Me".into()], (0., 0., 0.))
                            .into();
                    }
                    _ => {}
                }
            }
        }

        UiMenuOutput::None
    }

    fn render(&mut self, state: &mut StateInner) {
        self.action_menu
            .transform
            .look_at(state.renderer.camera.camera.translation, glam::Vec3::Y);

        state.renderer.draw_ui(&self.action_menu);

        if let Some(target) = &mut self.target_menu {
            target.transform.translation = self.action_menu.transform.translation
                + self.action_menu.transform.right() * 30.
                + self.action_menu.transform.forward() * 2.;

            target
                .transform
                .look_at(state.renderer.camera.camera.translation, glam::Vec3::Y);
            state.renderer.draw_ui(target);
        }
    }

    fn process_input(keys: &mut Input<KeyCode>, ui: &mut Ui3d) -> Option<UiMenuAction> {
        let up_pressed = keys.just_pressed(KeyCode::ArrowUp);
        let down_pressed = keys.just_pressed(KeyCode::ArrowDown);
        let dir = down_pressed as i8 - up_pressed as i8;

        let action = if keys.just_pressed(KeyCode::Enter) {
            Some(UiMenuAction::Select)
        } else if keys.just_pressed(KeyCode::ArrowRight) {
            Some(UiMenuAction::Forward)
        } else if keys.just_pressed(KeyCode::ArrowLeft) {
            Some(UiMenuAction::Back)
        } else {
            None
        };

        let selected = ui.selected as i8 + dir;
        ui.selected = selected.clamp(0, ui.options.len() as i8 - 1) as u8;

        return action;
    }
}

//====================================================================
