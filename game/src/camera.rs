//====================================================================

use engine::{tools::KeyCode, StateInner};

//====================================================================

const CAMERA_MOVE_SPEED: f32 = 100.;

pub fn move_camera(state: &mut StateInner) {
    let left = state.keys.pressed(KeyCode::KeyA);
    let right = state.keys.pressed(KeyCode::KeyD);

    let up = state.keys.pressed(KeyCode::Space);
    let down = state.keys.pressed(KeyCode::ShiftLeft);

    let forwards = state.keys.pressed(KeyCode::KeyW);
    let backwards = state.keys.pressed(KeyCode::KeyS);

    let x_dir = (right as i8 - left as i8) as f32;
    let y_dir = (up as i8 - down as i8) as f32;
    let z_dir = (forwards as i8 - backwards as i8) as f32;

    //--------------------------------------------------

    let dir = glam::Vec3::new(x_dir, y_dir, z_dir);

    if dir != glam::Vec3::ZERO {
        let forward = state.renderer.camera.camera.forward() * dir.z;
        let right = state.renderer.camera.camera.right() * dir.x;
        let up = glam::Vec3::Y * dir.y;

        state.renderer.camera.camera.translation +=
            (forward + right + up) * CAMERA_MOVE_SPEED * state.time.delta_seconds();
    }

    //--------------------------------------------------

    let look_left = state.keys.pressed(KeyCode::KeyJ);
    let look_right = state.keys.pressed(KeyCode::KeyL);

    let look_up = state.keys.pressed(KeyCode::KeyI);
    let look_down = state.keys.pressed(KeyCode::KeyK);

    let yaw = (look_right as i8 - look_left as i8) as f32;
    let pitch = (look_down as i8 - look_up as i8) as f32;

    //--------------------------------------------------

    state.renderer.camera.camera.rotate_camera(
        yaw * state.time.delta_seconds(),
        pitch * state.time.delta_seconds(),
    );
}

//====================================================================
