//====================================================================

use std::collections::{HashSet, VecDeque};

use common::{Size, Transform};
use engine::{scene::Scene, StateInner};
use hecs::{Entity, World};
use rand::Rng;
use ui::{UiMenuOutput, UiMenus};

use crate::characters::{self, Character, CharacterManager};

use self::characters::actions::ActionRepo;

mod server;
mod ui;

//====================================================================

pub struct Characters {
    friendly: HashSet<Entity>,
    enemy: HashSet<Entity>,
}

impl Characters {
    #[inline]
    pub fn friendly(&self) -> &HashSet<Entity> {
        &self.friendly
    }

    #[inline]
    pub fn enemy(&self) -> &HashSet<Entity> {
        &self.enemy
    }
}

pub struct BattleScene {
    _character_manager: CharacterManager,
    action_repo: ActionRepo,

    battle_state: BattleState,
    characters: Characters,

    current_character: Entity,
    turn_order: VecDeque<Entity>,
}

impl Scene for BattleScene {
    fn new(state: &mut StateInner) -> Self {
        crate::scenery::spawn_scenery(state);

        let mut character_manager = CharacterManager::new(state);
        let action_repo = ActionRepo::new();
        // let mut battle_manager = BattleManager::default();

        let idle_action = action_repo.find_action_name("Idle").unwrap();

        let friendly_characters = vec![character_manager.spawn(
            &mut state.world,
            "Friendly Character",
            vec![idle_action],
        )];

        let enemy_characters =
            vec![character_manager.spawn(&mut state.world, "Enemy Character", vec![idle_action])];

        Self {
            _character_manager: character_manager,
            action_repo,
            battle_state: BattleState::Initializing,
            characters: Characters {
                friendly: HashSet::from_iter(friendly_characters),
                enemy: HashSet::from_iter(enemy_characters),
            },
            current_character: Entity::DANGLING,
            turn_order: VecDeque::default(),
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

        self.tick_battle(state);

        characters::update_characters(state);
    }
}

//====================================================================

#[derive(Debug, Default)]
enum BattleState {
    #[default]
    Initializing,
    StartingRound,
    StartingTurn,
    WaitingForInput(UiMenus),
    ProcessingCpu,
}

impl BattleScene {
    fn position_characters(&self, world: &mut World) {
        self.characters
            .friendly
            .iter()
            .enumerate()
            .for_each(|(index, id)| {
                let mut transform = world.get::<&mut Transform>(*id).unwrap();

                transform.translation = glam::vec3(index as f32 * 100., 0., -100.);
                transform.rotation = glam::Quat::from_rotation_y(0.);
            });

        self.characters
            .enemy
            .iter()
            .enumerate()
            .for_each(|(index, id)| {
                let mut transform = world.get::<&mut Transform>(*id).unwrap();

                transform.translation = glam::vec3(index as f32 * 100., 0., 100.);
                transform.rotation = glam::Quat::from_rotation_y(0.);
            });
    }

    fn tick_battle(&mut self, state: &mut StateInner) {
        match &mut self.battle_state {
            BattleState::Initializing => {
                self.position_characters(&mut state.world);

                self.battle_state = BattleState::StartingRound;
            }

            BattleState::StartingRound => {
                self.start_round(&state.world);
                self.battle_state = BattleState::StartingTurn;
            }

            BattleState::StartingTurn => self.start_turn(state),

            BattleState::WaitingForInput(ui_menus) => {
                match ui_menus.tick(state, &self.action_repo, &self.characters) {
                    UiMenuOutput::None => {}
                    UiMenuOutput::SkipTurn => {
                        // next_turn = true;
                        ui_menus.drop_menus(&mut state.world);

                        self.start_turn(state);
                    }
                }
            }

            BattleState::ProcessingCpu => {}
        }
    }

    fn start_round(&mut self, world: &World) {
        log::info!("------Starting new round------");
        self.turn_order.clear();

        let mut weight = 0;
        let mut character_weights = Vec::new();

        self.characters
            .friendly
            .iter()
            .chain(self.characters.enemy.iter())
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

    fn start_turn(&mut self, state: &mut StateInner) {
        match self.turn_order.pop_front() {
            Some(next_character) => {
                self.current_character = next_character;

                let menu = UiMenus::new(state, &self.action_repo, next_character).unwrap();
                self.battle_state = BattleState::WaitingForInput(menu);
            }
            None => self.battle_state = BattleState::StartingRound,
        }
    }
}

//====================================================================
