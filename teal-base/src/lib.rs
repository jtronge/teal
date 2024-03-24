//! Main data structures and traits for teal.
//!
//! This is designed to store shared backend data structures and traits that
//! are needed by the application. This also includes those items that are used
//! for communication between the backend application and the GUI and are
//! designed primarily to keep the GUI and the backend separated for easy future
//! updates.

mod pixel;
pub use pixel::{Pixel, DisplayPixel};
mod gui;
pub use gui::{GUI, GUIContext, Event, DragEvent, KeyEvent, Key};
mod image;
pub use image::{Image, ImageView, ScreenBuffer};

/// An operation to be applied to an image.
pub trait Operation {
    /// Run the operation
    fn execute(&self, image: &mut Image);

    /// Undo the operation
    fn unexecute(&self, image: &mut Image);
}
