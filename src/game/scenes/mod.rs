//====================================================================

use crate::engine::{scene::Scene, tools::Transform, StateInner};

//====================================================================

pub struct EmptyScene;
impl Scene for EmptyScene {
    fn new() -> Self {
        Self
    }

    fn tick(&mut self, state: &mut StateInner) {
        state.renderer.texture_pipeline.draw_texture(
            state.renderer.default_texture.texture(),
            glam::vec2(50., 50.),
            [1., 0., 0., 1.],
            &Transform::default(),
        );
    }
}

//====================================================================
