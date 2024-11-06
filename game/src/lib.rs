//====================================================================

use engine::window::Runner;
use scenes::BattleScene;

pub(crate) mod camera;
pub(crate) mod characters;
pub(crate) mod scenery;
pub(crate) mod scenes;

//====================================================================

pub fn run() {
    Runner::<BattleScene>::run();
}

//====================================================================
