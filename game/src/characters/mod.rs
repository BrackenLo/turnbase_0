//====================================================================

use std::{
    collections::HashMap,
    f32::consts::{FRAC_PI_2, PI, TAU},
    sync::Arc,
};

use actions::Action;
use engine::{
    renderer::texture_storage::{DefaultTexture, LoadedTexture},
    tools::Transform,
    StateInner,
};
use glam::Vec3Swizzles;

pub mod actions;

//====================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CharacterId(u32);

#[derive(Debug)]
pub struct CharacterManager {
    current_id: CharacterId,
    characters: HashMap<CharacterId, Character>,

    default_texture: DefaultTexture,
}

impl CharacterManager {
    pub fn new(state: &mut StateInner) -> Self {
        Self {
            current_id: CharacterId(0),
            characters: HashMap::default(),

            default_texture: DefaultTexture::new(state.renderer.default_texture.get()),
        }
    }

    pub fn spawn(&mut self, name: &str) -> CharacterId {
        let id = self.current_id;
        self.current_id.0 += 1;

        let character = Character {
            name: name.into(),
            player_controlled: true,
            stats: CharacterStats { speed: 5 },
            actions: Vec::new(),
            transform: Transform::default(),
            front_facing: true,
            texture: self.default_texture.get(),
        };

        self.characters.insert(id, character);

        id
    }

    #[inline]
    pub fn character(&self, id: CharacterId) -> Option<&Character> {
        self.characters.get(&id)
    }

    #[inline]
    pub fn character_mut(&mut self, id: CharacterId) -> Option<&mut Character> {
        self.characters.get_mut(&id)
    }

    #[inline]
    pub fn update(&mut self, state: &mut StateInner) {
        self.characters
            .values_mut()
            .into_iter()
            .for_each(|character| character.update(state));
    }

    #[inline]
    pub fn render(&self, state: &mut StateInner) {
        self.characters
            .values()
            .into_iter()
            .for_each(|character| character.render(state));
    }
}

//====================================================================

#[derive(Debug)]
pub struct Character {
    pub name: String,
    pub player_controlled: bool,
    pub stats: CharacterStats,
    pub actions: Vec<Action>,

    pub transform: Transform,
    pub front_facing: bool,
    pub texture: Arc<LoadedTexture>,
}

#[derive(Debug)]
pub struct CharacterStats {
    pub speed: u32,
}

impl Character {
    pub fn update(&mut self, state: &mut StateInner) {
        let camera = &state.renderer.camera.camera;

        let sprite_rot = self.transform.forward().xz().to_angle();

        let z = self.transform.translation.z - camera.translation.z;
        let x = self.transform.translation.x - camera.translation.x;

        let angle = f32::atan2(z, x) + sprite_rot;
        let angle = angle % TAU;
        let angle = match angle > PI {
            true => angle - TAU,
            false => angle,
        };

        let front_facing = angle > -FRAC_PI_2 && angle < FRAC_PI_2;
        self.front_facing = front_facing;
    }

    pub fn render(&self, state: &mut StateInner) {
        state.renderer.texture_pipeline.draw_texture(
            &self.texture,
            glam::vec2(50., 50.),
            match self.front_facing {
                true => [1., 1., 1., 1.],
                false => [1., 0., 0., 1.],
            },
            &self.transform,
        );
    }
}

//====================================================================
