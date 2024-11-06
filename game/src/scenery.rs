//====================================================================

use std::sync::Arc;

use engine::{renderer::texture_storage::LoadedTexture, tools::Transform, StateInner};

//====================================================================

pub struct Scenery {
    floor_pos: Transform,
    floor_tex: Arc<LoadedTexture>,
}

impl Scenery {
    pub fn new(state: &mut StateInner) -> Self {
        Self {
            floor_pos: Transform::from_rotation_translation(
                glam::Quat::from_rotation_x(90_f32.to_radians()),
                glam::vec3(0., -20., 0.),
            ),
            floor_tex: state.renderer.default_texture.get(),
        }
    }

    pub fn render(&self, state: &mut StateInner) {
        state.renderer.texture_pipeline.draw_texture(
            &self.floor_tex,
            glam::vec2(500., 500.),
            [0.3, 0.3, 0.3, 1.],
            &self.floor_pos,
        );
    }
}

//====================================================================
