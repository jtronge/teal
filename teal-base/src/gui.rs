//! Generic GUI traits and data structures.
use crate::DisplayPixel;

/// Main GUI abstraction.
///
/// See https://stackoverflow.com/questions/50090578/how-to-write-a-trait-bound-for-a-reference-to-an-associated-type-on-the-trait-it
/// for more info on the trait bound.
pub trait GUI {
    type Context<'a>: GUIContext;

    fn run<F: Fn(Self::Context<'_>, Event) + 'static>(&mut self, f: F);
}

/// GUI context for interacting with the GUI front end.
///
/// This is where most communication occurs in the run/event handling closure.
pub trait GUIContext {
    fn screen(&mut self) -> impl crate::ScreenBuffer;
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

    /// A key release event
    KeyRelease(Key),

    /// Start of a drag gesture
    DragBegin(f64, f64),

    /// Update drag gesture
    DragUpdate(f64, f64),

    /// Finish drag gesture
    DragEnd(f64, f64),

    /// Window resize
    Resize,
}
