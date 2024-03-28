//! Main data structures and traits for teal.
//!
//! This is designed to store shared backend data structures and traits that
//! are needed by the application. This also includes those items that are used
//! for communication between the backend application and the GUI and are
//! designed primarily to keep the GUI and the backend separated for easy future
//! updates.

/// Rexport the image crate
pub use image;

mod gui;
pub use gui::{DragEvent, Event, GUIContext, GUIOptions, Key, KeyEvent, GUI};

/// Image pixel type
pub type ImagePixel = image::Rgba<f32>;

/// Main image type
pub type Image = image::ImageBuffer<ImagePixel, Vec<<ImagePixel as image::Pixel>::Subpixel>>;

/// Pixel to be used for display.
#[derive(Clone, Debug)]
pub struct DisplayPixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl DisplayPixel {
    /// Do a lossy conversion from the image pixel type.
    fn from_image_pixel(pixel: &ImagePixel) -> DisplayPixel {
        DisplayPixel {
            r: (pixel.0[0] * (u8::MAX as f32)) as u8,
            g: (pixel.0[1] * (u8::MAX as f32)) as u8,
            b: (pixel.0[2] * (u8::MAX as f32)) as u8,
            // The alpha channel is ignored
        }
    }
}

/// ImageView handles coordinate-conversion between a front-end screen
/// buffer and backend image data.
#[derive(Clone, Debug)]
pub struct ImageView {
    /// X-position of upper left corner of image in view
    disp_corner_x: f64,

    /// Y-position of upper left corner of image in view
    disp_corner_y: f64,

    /// Conversion factor from display coordinates to image coordinates
    conversion_factor: f64,
}

/// Checkerboard pattern square dimension
pub const CHECKERBOARD_DIM: u32 = 20;

/// Produce a procedural checkerboard, used for empty parts of the screen.
fn checkerboard(screen_x: u32, screen_y: u32) -> DisplayPixel {
    let x = screen_x / CHECKERBOARD_DIM;
    let y = screen_y / CHECKERBOARD_DIM;

    if ((x + y) % 2) == 0 {
        DisplayPixel {
            r: 0,
            g: 128,
            b: 128,
        }
    } else {
        DisplayPixel {
            r: 0,
            g: 255,
            b: 255,
        }
    }
}

impl ImageView {
    pub fn new() -> ImageView {
        ImageView {
            disp_corner_x: 0.0,
            disp_corner_y: 0.0,
            conversion_factor: 0.5,
        }
    }

    /// Get the image coordinates. Return None on out of bounds.
    pub fn get_image_coords(
        &self,
        image: &Image,
        screen_x: u32,
        screen_y: u32,
    ) -> Option<(u32, u32)> {
        let x = screen_x as f64;
        let y = screen_y as f64;

        // Check if coordinate is outside image
        if x < self.disp_corner_x || y < self.disp_corner_y {
            return None;
        }

        let img_x = x * self.conversion_factor;
        let img_y = y * self.conversion_factor;
        let img_x = img_x as u32;
        let img_y = img_y as u32;

        // Check if this coordinate is outside the image
        if img_x >= image.width() || img_y >= image.height() {
            return None;
        }

        Some((img_x, img_y))
    }

    /// Get a display pixel for the screen coordinates.
    pub fn get_display_pixel(&self, image: &Image, screen_x: u32, screen_y: u32) -> DisplayPixel {
        if let Some((img_x, img_y)) = self.get_image_coords(image, screen_x, screen_y) {
            DisplayPixel::from_image_pixel(image.get_pixel(img_x, img_y))
        } else {
            checkerboard(screen_x, screen_y)
        }
    }

    pub fn update_screen(&self, image: &Image, mut screen: impl ScreenBuffer) {
        let width = screen.width();
        let height = screen.height();
        for x in 0..width {
            for y in 0..height {
                let pixel = self.get_display_pixel(image, x, y);
                screen.set(x, y, pixel);
            }
        }
    }

    pub fn image_coord(
        &self,
        img_width: f64,
        img_height: f64,
        disp_x: f64,
        disp_y: f64,
    ) -> (f64, f64) {
        let x = disp_x * self.conversion_factor;
        let x = if x >= img_width { img_width - 1.0 } else { x };
        let y = disp_y * self.conversion_factor;
        let y = if y >= img_height { img_height - 1.0 } else { y };
        (x, y)
    }
}

/// A trait for GUI/front end screen buffers.
///
/// This trait is used to abstract different types of screen buffers that may
/// be provided by different GUIs.
pub trait ScreenBuffer {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn set(&mut self, x: u32, y: u32, pixel: DisplayPixel);
}
