//! Pixel data for both internal computations and display.

/// An RGBA Pixel for internal computations.
#[derive(Clone, Debug)]
pub struct Pixel {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Pixel {
    /// Convert to a display pixel.
    #[inline]
    pub fn display_pixel(&self) -> DisplayPixel {
        DisplayPixel {
            r: (self.r * (u8::MAX as f64)) as _,
            g: (self.g * (u8::MAX as f64)) as _,
            b: (self.b * (u8::MAX as f64)) as _,
            a: (self.a * (u8::MAX as f64)) as _,
        }
    }
}

/// Pixel to be used for display.
#[derive(Clone, Debug)]
pub struct DisplayPixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
