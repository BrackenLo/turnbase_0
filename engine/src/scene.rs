//====================================================================

use common::Size;

use crate::StateInner;

//====================================================================

pub trait Scene: 'static {
    fn new(state: &mut StateInner) -> Self
    where
        Self: Sized;

    fn resize(&mut self, state: &mut StateInner, new_size: Size<u32>);
    fn update(&mut self, state: &mut StateInner);
}

//====================================================================
