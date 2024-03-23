//! Base image data structures and related traits.
use crate::{Pixel, DisplayPixel};

/// Backend image representation.
pub struct Image {
    /// Width of the image
    pub width: usize,

    /// Height of the image
    pub height: usize,

    /// Pixel data in [r, g, b, a] form
    pixels: Vec<Pixel>,
}

impl Image {
    pub fn new(width: usize, height: usize, pixel: Pixel) -> Image {
        Image {
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

/// ImageView handles coordinate-conversion between a front-end screen
/// buffer and backend image data.
pub struct ImageView {
    /// X-position of upper left corner of image in view
    disp_corner_x: f64,

    /// Y-position of upper left corner of image in view
    disp_corner_y: f64,

    /// Conversion factor from display coordinates to image coordinates
    conversion_factor: f64,
}

/// Checkerboard pattern square dimension
pub const CHECKERBOARD_DIM: usize = 20;

/// Produce a procedural checkerboard, used for empty parts of the screen.
fn checkerboard(screen_x: usize, screen_y: usize) -> DisplayPixel {
    let x = screen_x / CHECKERBOARD_DIM;
    let y = screen_y / CHECKERBOARD_DIM;

    if ((x + y) % 2) == 0 {
        DisplayPixel {
            r: 0,
            g: 128,
            b: 128,
            a: 255,
        }
    } else {
        DisplayPixel {
            r: 0,
            g: 255,
            b: 255,
            a: 255,
        }
    }
}

impl ImageView {
    pub fn new() -> ImageView {
        ImageView {
            disp_corner_x: 0.0,
            disp_corner_y: 0.0,
            conversion_factor: 1.0,
        }
    }

    /// Get the image coordinates. Return None on out of bounds.
    pub fn get_image_coords(
        &self,
        image: &Image,
        screen_x: usize,
        screen_y: usize,
    ) -> Option<(usize, usize)> {
        let x = screen_x as f64;
        let y = screen_y as f64;

        // Check if coordinate is outside image
        if x < self.disp_corner_x || y < self.disp_corner_y {
            return None;
        }

        let img_x = x * self.conversion_factor;
        let img_y = y * self.conversion_factor;
        let img_x = img_x as usize;
        let img_y = img_y as usize;

        // Check if this coordinate is outside the image
        if img_x >= image.width || img_y >= image.height {
            return None;
        }

        Some((img_x, img_y))
    }

    /// Get a display pixel for the screen coordinates.
    pub fn get_display_pixel(
        &self,
        image: &Image,
        screen_x: usize,
        screen_y: usize,
    ) -> DisplayPixel {
        if let Some((img_x, img_y)) = self.get_image_coords(
            image,
            screen_x,
            screen_y,
        ) {
            image.get(img_x, img_y).display_pixel()
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

    pub fn image_coord(&self, img_width: f64, img_height: f64, disp_x: f64, disp_y: f64) -> (f64, f64) {
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
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn set(&mut self, x: usize, y: usize, pixel: DisplayPixel);
}
