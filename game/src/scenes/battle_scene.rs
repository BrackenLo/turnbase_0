//====================================================================

use std::collections::{HashSet, VecDeque};

use common::{Size, Transform};
use engine::{scene::Scene, tools::KeyCode, StateInner};
use hecs::{Entity, World};
use rand::Rng;
use renderer::pipelines::ui3d_pipeline::Ui3d;

use crate::{
    characters::{self, Character, CharacterManager},
    scenery::Scenery,
};

//====================================================================

pub struct BattleScene {
    _scenery: Scenery,

    _character_manager: CharacterManager,
    battle_manager: BattleManager,
}

impl Scene for BattleScene {
    fn new(state: &mut StateInner) -> Self {
        let mut character_manager = CharacterManager::new(state);
        let mut battle_manager = BattleManager::default();

        battle_manager
            .add_friendly(character_manager.spawn(&mut state.world, "Friendly Character"));
        battle_manager.add_enemy(character_manager.spawn(&mut state.world, "Enemy Character"));

        Self {
            _scenery: Scenery::new(state),
            _character_manager: character_manager,
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

        // self.battle_manager.position_characters(&mut state.world);
        self.battle_manager.tick(state);

        characters::update_characters(state);
    }
}

//====================================================================

#[derive(Debug, Default)]
struct BattleManager {
    state: BattleState,

    friendly: HashSet<Entity>,
    enemy: HashSet<Entity>,

    turn_order: VecDeque<Entity>,
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
    fn add_friendly(&mut self, id: Entity) {
        self.friendly.insert(id);
    }

    fn add_enemy(&mut self, id: Entity) {
        self.enemy.insert(id);
    }

    fn position_characters(&self, world: &mut World) {
        self.friendly.iter().enumerate().for_each(|(index, id)| {
            let mut transform = world.get::<&mut Transform>(*id).unwrap();

            transform.translation = glam::vec3(index as f32 * 100., 0., -100.);
            transform.rotation = glam::Quat::from_rotation_y(0.);
        });

        self.enemy.iter().enumerate().for_each(|(index, id)| {
            let mut transform = world.get::<&mut Transform>(*id).unwrap();

            transform.translation = glam::vec3(index as f32 * 100., 0., 100.);
            transform.rotation = glam::Quat::from_rotation_y(0.);
        });
    }
}

impl BattleManager {
    fn tick(&mut self, state: &mut StateInner) {
        let mut next_turn = false;

        match &mut self.state {
            BattleState::Initializing => {
                self.position_characters(&mut state.world);

                self.state = BattleState::StartingRound;
            }

            BattleState::StartingRound => {
                self.start_round(&state.world);

                // let current_character = characters
                //     .character(self.turn_order.get(0).unwrap())
                //     .unwrap();

                // log::info!("next character = {}", current_character.name);

                self.state = BattleState::WaitingForInput(UiMenus::new(state, self.turn_order[0]));
                // self.state = BattleState::ProcessingCpu;
            }

            BattleState::WaitingForInput(ui_menus) => match ui_menus.tick(state) {
                UiMenuOutput::None => {}
                UiMenuOutput::SkipTurn => {
                    next_turn = true;
                    ui_menus.drop_menus(&mut state.world);
                }
            },

            BattleState::ProcessingCpu => {}
        }

        if next_turn {
            self.next_turn(state);
        }
    }

    fn start_round(&mut self, world: &World) {
        log::info!("------Starting new round------");
        self.turn_order.clear();

        let mut weight = 0;
        let mut character_weights = Vec::new();

        self.friendly
            .iter()
            .chain(self.enemy.iter())
            .for_each(|id| {
                let character = world.get::<&Character>(*id).unwrap();

                weight += character.stats.speed;
                character_weights.push((character.stats.speed, *id));
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
                .fold(String::new(), |acc, id| format!(
                    "{}, {}",
                    acc,
                    world.get::<&Character>(*id).unwrap().name
                ))
        );
    }

    fn next_turn(&mut self, state: &mut StateInner) {
        if self.turn_order.len() <= 1 {
            self.state = BattleState::StartingRound;
            return;
        }

        self.turn_order.pop_front();

        self.state = BattleState::WaitingForInput(UiMenus::new(state, self.turn_order[0]));
    }
}

//====================================================================

#[derive(Debug)]
struct UiMenus {
    action_menu: Entity,
    target_menu: Option<Entity>,
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
    pub fn drop_menus(&self, world: &mut World) {
        world.despawn(self.action_menu).ok();
        if let Some(target_menu) = self.target_menu {
            world.despawn(target_menu).ok();
        }
    }

    pub fn new(state: &mut StateInner, character: Entity) -> Self {
        let menu_pos = {
            let character_transform = state.world.get::<&Transform>(character).unwrap();
            character_transform.translation + character_transform.right() * 50.
        };

        let action_menu = state.world.spawn((
            Ui3d {
                options: vec!["One".into(), "Two".into(), "Three".into()],
                ..Default::default()
            },
            Transform::from_scale_translation((0.8, 0.8, 0.8), menu_pos),
        ));

        Self {
            action_menu,
            target_menu: None,
        }
    }

    fn tick(&mut self, state: &mut StateInner) -> UiMenuOutput {
        self.position_children(state);

        // Process target menu if available
        if let Some(target) = self.target_menu {
            if let Some(action) = Self::process_input(state, target) {
                match action {
                    UiMenuAction::Forward | UiMenuAction::Select => return UiMenuOutput::SkipTurn,
                    UiMenuAction::Back => {
                        state.world.despawn(target).ok();
                        self.target_menu = None;
                    }
                }
            }
        }
        // Process Actions menu
        else {
            if let Some(action) = Self::process_input(state, self.action_menu) {
                match action {
                    UiMenuAction::Forward | UiMenuAction::Select => {
                        self.target_menu = state
                            .world
                            .spawn((
                                Transform::from_scale((0.3, 0.3, 0.3)),
                                Ui3d {
                                    options: vec!["You".into(), "Me".into()],
                                    ..Default::default()
                                },
                            ))
                            .into();

                        self.position_children(state);
                    }
                    _ => {}
                }
            }
        }

        UiMenuOutput::None
    }

    fn position_children(&mut self, state: &mut StateInner) {
        if let Some(target) = self.target_menu {
            let new_pos = {
                let parent_transform = state.world.get::<&Transform>(self.action_menu).unwrap();

                parent_transform.translation
                    + parent_transform.right() * (parent_transform.scale.x * 100.)
                    + parent_transform.forward() * 2.
            };

            let mut transform = state.world.get::<&mut Transform>(target).unwrap();

            transform.translation = new_pos;
        }
    }

    fn process_input(state: &mut StateInner, target: Entity) -> Option<UiMenuAction> {
        let keys = &mut state.keys;

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

        let mut ui = state.world.get::<&mut Ui3d>(target).unwrap();

        let selected = ui.selected as i8 + dir;
        ui.selected = selected.clamp(0, ui.options.len() as i8 - 1) as u8;

        return action;
    }
}

//====================================================================
