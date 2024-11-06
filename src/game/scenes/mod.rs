//====================================================================

use crate::engine::{scene::Scene, StateInner};

//====================================================================

pub struct EmptyScene;
impl Scene for EmptyScene {
    fn new() -> Self {
        Self
    }

    fn tick(&mut self, _state: &mut StateInner) {}
}

//====================================================================
