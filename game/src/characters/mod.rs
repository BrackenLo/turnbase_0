//====================================================================

use std::{
    collections::HashSet,
    f32::consts::{FRAC_PI_2, PI, TAU},
};

use actions::ActionId;
use common::Transform;
use engine::StateInner;
use glam::Vec3Swizzles;
use hecs::{Entity, World};
use renderer::{pipelines::texture_pipeline::Sprite, texture_storage::DefaultTexture};

pub mod actions;

//====================================================================

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct CharacterId(u32);

#[derive(Debug)]
pub struct CharacterManager {
    // current_id: CharacterId,
    characters: HashSet<Entity>,

    default_texture: DefaultTexture,
}

impl CharacterManager {
    pub fn new(state: &mut StateInner) -> Self {
        Self {
            // current_id: CharacterId(0),
            characters: HashSet::default(),

            default_texture: DefaultTexture::new(state.renderer.default_texture.get()),
        }
    }

    pub fn spawn(&mut self, world: &mut World, name: &str, actions: Vec<ActionId>) -> Entity {
        assert!(actions.len() > 0);

        let character = world.spawn((
            Character {
                name: name.into(),
                player_controlled: true,
                stats: CharacterStats { speed: 5 },
                actions,
                front_facing: true,
            },
            Transform::default(),
            Sprite {
                texture: self.default_texture.get(),
                size: glam::vec2(50., 50.),
                color: [1.; 4],
            },
        ));

        self.characters.insert(character);
        character
    }
}

//====================================================================

#[allow(dead_code)]
#[derive(Debug)]
pub struct Character {
    pub name: String,
    pub player_controlled: bool,
    pub stats: CharacterStats,
    pub actions: Vec<ActionId>,

    pub front_facing: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct CharacterStats {
    pub speed: u32,
}

pub fn update_characters(state: &mut StateInner) {
    let camera = &state.renderer.camera.camera;

    state
        .world
        .query::<(&mut Transform, &mut Character)>()
        .iter()
        .for_each(|(_, (transform, character))| {
            let sprite_rot = transform.forward().xz().to_angle();

            let z = transform.translation.z - camera.translation.z;
            let x = transform.translation.x - camera.translation.x;

            let angle = f32::atan2(z, x) + sprite_rot;
            let angle = angle % TAU;
            let angle = match angle > PI {
                true => angle - TAU,
                false => angle,
            };

            let front_facing = angle > -FRAC_PI_2 && angle < FRAC_PI_2;
            character.front_facing = front_facing;
        });
}

//====================================================================
