use serde::Deserialize;

/// Brush configuration.
#[derive(Deserialize)]
pub struct Brush {
    /// Name of the brush.
    pub name: String,

    /// Path to brush image file.
    pub file: String,

    /// Quick ID (to be used to set the current brush).
    pub quickid: char,
}

/// Color setting.
#[derive(Deserialize)]
pub struct Color {
    /// Red channel.
    r: f32,

    /// Green channel.
    g: f32,

    /// Blue channel.
    b: f32,

    /// Alpha channel.
    a: f32,
}

/// Main application config.
#[derive(Deserialize)]
pub struct Config {
    /// Maximum number of operations in the undo buffer.
    pub max_undo: usize,

    /// Maximum number of operations in the redo buffer.
    pub max_redo: usize,

    /// List of available brushes.
    pub brushes: Vec<Brush>,

    /// Default color.
    pub default_color: Color,
}
