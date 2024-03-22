//! Generic GUI traits and data structures.
use crate::DisplayPixel;

/// Image display buffer
pub trait DisplayBuffer {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn set(&mut self, x: usize, y: usize, pixel: DisplayPixel);
}

#[derive(Debug)]
pub enum Key {
    /// An entered key sequence with possible modifiers
    Sequence {
        value: char,
        control: bool,
        alt: bool,
    },

    /// Control key pressed by itself
    PlainControl,

    /// Alt key pressed by itself
    PlainAlt,
}

#[derive(Debug)]
pub enum Event {
    /// A key press event
    KeyPress(Key),

    /// Start of a drag gesture
    DragBegin(f64, f64),

    /// Update drag gesture
    DragUpdate(f64, f64),

    /// Finish drag gesture
    DragEnd(f64, f64),

    /// Window resize
    Resize,
}
