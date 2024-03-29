//! Generic GUI traits and data structures.

/// Options for the GUI.
pub struct GUIOptions {}

/// Main GUI abstraction.
///
/// See <https://stackoverflow.com/questions/50090578/how-to-write-a-trait-bound-for-a-reference-to-an-associated-type-on-the-trait-it>
/// for more info on the trait bound.
pub trait GUI {
    type Context<'a>: GUIContext;

    fn run<F: Fn(Self::Context<'_>, Event) + 'static>(&mut self, options: GUIOptions, f: F);
}

/// GUI context for interacting with the GUI front end.
///
/// This is where most communication occurs in the run/event handling closure.
pub trait GUIContext {
    fn screen(&mut self) -> impl crate::ScreenBuffer;
}

/// Enum representing various types and sequences of key presses
#[derive(Clone, Debug)]
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

/// Key event
#[derive(Debug)]
pub enum KeyEvent {
    /// Key press
    Press(Key),

    /// Key release
    Release(Key),
}

/// Drag event
#[derive(Debug)]
pub enum DragEvent {
    /// Start of a drag gesture
    Begin(f64, f64),

    /// Update drag gesture
    Update(f64, f64),

    /// Finish drag gesture
    End(f64, f64),
}

#[derive(Debug)]
pub enum Event {
    /// A key event (press or release)
    Key(KeyEvent),

    /// Drag event motion
    Drag(DragEvent),

    /// A new color was chosen
    ColorUpdate { r: f32, g: f32, b: f32, a: f32 },

    /// Window resize
    Resize,
}
