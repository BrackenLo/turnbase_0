//====================================================================

use engine::window::Runner;
use scenes::battle_scene::BattleScene;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub(crate) mod camera;
pub(crate) mod characters;
pub(crate) mod scenery;
pub(crate) mod scenes;

//====================================================================

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Debug).expect("Couldn't initialize logger");
    }
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::Builder::new()
        .filter_module(env!("CARGO_PKG_NAME"), log::LevelFilter::Trace)
        .filter_module("engine", log::LevelFilter::Trace)
        .filter_module("wgpu", log::LevelFilter::Warn)
        .init();

    Runner::<BattleScene>::run();
}

//====================================================================
