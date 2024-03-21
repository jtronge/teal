//! The main data structures for teal
//!
//! This is designed to store all data that is needed by the application, which
//! can be useful to both the backend algorithms and the GUI frontend. It's also
//! important that this help maintain strict separation between the GUI and the
//! backend code as make chaning GUI components and backend components as simple
//! as possible.

/// An RGBA Pixel
#[derive(Clone, Debug)]
pub struct Pixel {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

pub struct ImageData {
    /// Width of the image
    pub width: usize,

    /// Height of the image
    pub height: usize,

    /// Pixel data in [r, g, b, a] form
    pixels: Vec<Pixel>,
}

impl ImageData {
    pub fn new(width: usize, height: usize, pixel: Pixel) -> ImageData {
        ImageData {
            width,
            height,
            pixels: vec![pixel; width * height],
        }
    }

    /// Set a pixel value at (x, y).
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, pixel: Pixel) {
        self.pixels[y * self.width + x] = pixel;
    }

    /// Get a pixel value at (x, y).
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> &Pixel {
        &self.pixels[y * self.width + x]
    }
}
