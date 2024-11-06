//====================================================================

use engine::{scene::Scene, window::Runner};
use scenes::EmptyScene;

pub mod camera;
pub mod characters;
pub mod scenes;

//====================================================================

pub fn run() {
    Runner::run(Box::new(EmptyScene::new()));
}

//====================================================================
