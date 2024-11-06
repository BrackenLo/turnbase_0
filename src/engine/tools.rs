//====================================================================

use std::fmt::Display;

//====================================================================

#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

#[allow(dead_code)]
impl<T> Size<T> {
    #[inline]
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

impl<T> From<(T, T)> for Size<T> {
    #[inline]
    fn from(value: (T, T)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}

impl<T: Display> Display for Size<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.width, self.height)
    }
}

impl<T> From<winit::dpi::PhysicalSize<T>> for Size<T> {
    #[inline]
    fn from(value: winit::dpi::PhysicalSize<T>) -> Self {
        Size {
            width: value.width,
            height: value.height,
        }
    }
}

//====================================================================
