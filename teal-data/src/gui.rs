//! Generic GUI traits and data structures.
use crate::DisplayPixel;

/// Image display buffer
pub trait DisplayBuffer {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn set(&mut self, x: usize, y: usize, pixel: DisplayPixel);
}
