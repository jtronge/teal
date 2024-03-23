//! The main data structures for teal
//!
//! This is designed to store all data that is needed by the application, which
//! can be useful to both the backend algorithms and the GUI frontend. It's also
//! important that this help maintain strict separation between the GUI and the
//! backend code as make chaning GUI components and backend components as simple
//! as possible.

mod pixel;
pub use pixel::{Pixel, DisplayPixel};
mod gui;
pub use gui::{GUI, GUIContext, Key, Event};
mod image;
pub use image::{Image, ImageView, ScreenBuffer};

/*
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

/// The ImageDisplay handles coordinate-conversion between a front-end display
/// and a backend image.
pub struct ImageDisplay {
    /// Width of the view
    disp_width: f64,

    /// Height of the view
    disp_height: f64,

    /// X-position of upper left corner of image in view
    disp_corner_x: f64,

    /// Y-position of upper left corner of image in view
    disp_corner_y: f64,

    /// Conversion factor from display coordinates to image coordinates
    conversion_factor: f64,
}

impl ImageDisplay {
    pub fn new(disp_width: f64, disp_height: f64) -> ImageDisplay {
        ImageDisplay {
            disp_width,
            disp_height,
            disp_corner_x: 0.0,
            disp_corner_y: 0.0,
            conversion_factor: 1.0,
        }
    }

    /// Return the image coordinates for the display coordinates.
    pub fn image_coord(&self, img_width: f64, img_height: f64, disp_x: f64, disp_y: f64) -> (f64, f64) {
        let x = disp_x * self.conversion_factor;
        let x = if x >= img_width { img_width - 1.0 } else { x };
        let y = disp_y * self.conversion_factor;
        let y = if y >= img_height { img_height - 1.0 } else { y };
        (x, y)
    }
}

pub struct Image {
    pub data: ImageData,
    pub display: ImageDisplay,
}

impl Image {
    pub fn new(data: ImageData, display: ImageDisplay) -> Image {
        Image {
            data,
            display,
        }
    }

    pub fn image_coord(&self, disp_x: f64, disp_y: f64) -> (f64, f64) {
        self.display.image_coord(self.data.width as _, self.data.height as _, disp_x, disp_y)
    }

    /// Get the value of the pixel for the given display corrdinates
    pub fn get_display_pixel(&self, disp_x: f64, disp_y: f64) -> DisplayPixel {
        let (img_x, img_y) = self.image_coord(disp_x, disp_y);
        self.data.get(img_x as _, img_y as _).display_pixel()
    }
}
*/
