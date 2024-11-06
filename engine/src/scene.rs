//====================================================================

use crate::{tools::Size, StateInner};

//====================================================================

pub trait Scene: 'static {
    fn new(state: &mut StateInner) -> Self
    where
        Self: Sized;

    fn tick(&mut self, state: &mut StateInner);
    fn resize(&mut self, state: &mut StateInner, new_size: Size<u32>);
}

//====================================================================
