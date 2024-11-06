//====================================================================

use std::collections::HashSet;

use engine::{scene::Scene, StateInner};

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

        battle_manager.add_friendly(character_manager.spawn("Friendly"));
        battle_manager.add_enemy(character_manager.spawn("Enemy"));

        Self {
            scenery: Scenery::new(state),
            character_manager,
            battle_manager,
        }
    }

    fn tick(&mut self, state: &mut StateInner) {
        crate::camera::move_camera(state);
        self.scenery.render(state);

        match self.battle_manager.state {
            BattleState::Starting => {
                self.battle_manager
                    .position_characters(&mut self.character_manager);
                self.battle_manager.state = BattleState::Waiting;
            }
            BattleState::Waiting => {}
        }

        self.character_manager.update(state);
        self.character_manager.render(state);
    }

    fn resize(&mut self, state: &mut StateInner, new_size: engine::tools::Size<u32>) {
        state
            .renderer
            .camera
            .set_aspect(new_size.width as f32, new_size.height as f32);
    }
}

#[derive(Debug, Default)]
struct BattleManager {
    state: BattleState,

    friendly: HashSet<CharacterId>,
    enemy: HashSet<CharacterId>,
}

#[derive(Debug, Default)]
enum BattleState {
    #[default]
    Starting,
    Waiting,
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
            let character = characters.character_mut(*id).unwrap();

            character.transform.translation = glam::vec3(index as f32 * 100., 0., -100.);
            character.transform.rotation = glam::Quat::from_rotation_y(0.);
        });

        self.enemy.iter().enumerate().for_each(|(index, id)| {
            let character = characters.character_mut(*id).unwrap();

            character.transform.translation = glam::vec3(index as f32 * 100., 0., 100.);
            character.transform.rotation = glam::Quat::from_rotation_y(std::f32::consts::PI);
        });
    }
}

//====================================================================
