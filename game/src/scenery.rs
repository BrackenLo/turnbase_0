//====================================================================

use common::Transform;
use engine::StateInner;
use renderer::pipelines::texture_pipeline::Sprite;

//====================================================================

pub struct Scenery;

pub fn spawn_scenery(state: &mut StateInner) {
    state.world.spawn((
        Scenery,
        Transform::from_rotation_translation(
            glam::Quat::from_rotation_x(90_f32.to_radians()),
            glam::vec3(0., -20., 0.),
        ),
        Sprite {
            texture: state.renderer.default_texture.get(),
            size: glam::vec2(500., 500.),
            color: [0.3, 0.3, 0.3, 1.],
        },
    ));
}

//====================================================================
