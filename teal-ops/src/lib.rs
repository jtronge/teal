use teal_base::image::Pixel;
use teal_base::{Brush, Image, ImagePixel, ImageView};
use std::collections::HashMap;

/// An operation to be applied to an image.
pub trait Operation {
    /// Redo the operation.
    fn redo(&mut self, image: &mut Image);

    /// Undo the operation.
    fn undo(&mut self, image: &mut Image);
}

/// Trait to abstract application of some operation that can be applied on a
/// series of lines on a canvas.
///
/// This could be for lines of paint or for more special operations like smudge
/// and blur.
pub trait Liner {
    /// Brush a line from point a to point b.
    fn line(
        &mut self,
        image: &mut Image,
        a: (f64, f64),
        b: (f64, f64),
        undo_pixels: &mut HashMap<(u32, u32), ImagePixel>,
    );
}

/// Stored representation of a drag gesture.
pub struct BrushOp {
    /// Saved image view.
    image_view: ImageView,

    /// First screen point.
    start: Option<(f64, f64)>,

    /// Input drag points (stored as offsets from the first screen point).
    points: Vec<(f64, f64)>,

    /// Original pixels that have been overwritten, for undo operation.
    undo_pixels: HashMap<(u32, u32), ImagePixel>,

    /// Liner operation.
    line_op: Box<dyn Liner>,
}

impl BrushOp {
    /// Create a new drag operation for the current image view.
    pub fn new<L: Liner + 'static>(image_view: ImageView, liner: L) -> BrushOp {
        BrushOp {
            image_view,
            start: None,
            points: vec![],
            undo_pixels: HashMap::new(),
            line_op: Box::new(liner),
        }
    }

    /// Add the first point of a drag operation and update the image.
    pub fn start(&mut self, image: &mut Image, start_x: f64, start_y: f64) {
        let _ = self.start.insert((start_x, start_y));
        self.points.push((0.0, 0.0));
    }

    /// Add the next point to the drag operation, updating the image.
    pub fn update(&mut self, image: &mut Image, off_x: f64, off_y: f64) {
        if self.points.len() == 0 {
            panic!("invalid use of BrushOp: start() was not called");
        }

        let (last_off_x, last_off_y) = self.points[self.points.len()-1];
        let a = self.get_image_coords(image, last_off_x, last_off_y);
        let b = self.get_image_coords(image, off_x, off_y);
        self.line_op.line(image, a, b, &mut self.undo_pixels);
        self.points.push((off_x, off_y));
    }

    /// Add the final point to the drag operation and update the image.
    pub fn finish(&mut self, image: &mut Image, off_x: f64, off_y: f64) {
        self.update(image, off_x, off_y);
    }

    /// Get image coordinates for the given offsets.
    ///
    /// NOTE: These could potentially be outside the bounds of the actual image.
    fn get_image_coords(&self, image: &Image, off_x: f64, off_y: f64) -> (f64, f64) {
        let (start_x, start_y) = self.start
            .as_ref()
            .expect("missing start point");
        let screen_x = start_x + off_x;
        let screen_y = start_y + off_y;
        self.image_view.get_image_coords_f(image, screen_x, screen_y)
    }
}

impl Operation for BrushOp {
    fn redo(&mut self, image: &mut Image) {
        // Works since self.undo_pixels will contain the redo pixels.
        self.undo(image);
    }

    fn undo(&mut self, image: &mut Image) {
        println!("undoing operation");
        let mut redo_pixels = HashMap::new();
        for ((x, y), pixel) in self.undo_pixels.iter() {
            redo_pixels.insert((*x, *y), *image.get_pixel(*x, *y));
            image.put_pixel(*x, *y, *pixel);
        }
        // Instead of having separate buffers for undo and redo pixels, just
        // use one.
        self.undo_pixels = redo_pixels;
    }
}

const ROUND_BRUSH: [[f32; 4]; 4] = [
    [0.0, 0.4, 0.4, 0.0],
    [0.4, 1.0, 1.0, 0.4],
    [0.4, 1.0, 1.0, 0.4],
    [0.0, 0.4, 0.4, 0.0],
];

/// Fill in a dot around the point from the
fn fill_dot(
    image: &mut Image,
    color: &ImagePixel,
    img_x: u32,
    img_y: u32,
    undo_pixels: &mut HashMap<(u32, u32), ImagePixel>,
) {
    let img_x: isize = img_x.try_into().unwrap();
    let img_y: isize = img_y.try_into().unwrap();
    let half_dim: isize = (ROUND_BRUSH.len() / 2).try_into().unwrap();
    for (j, row) in ROUND_BRUSH.iter().enumerate() {
        for (i, value) in row.iter().enumerate() {
            let i: isize = i.try_into().unwrap();
            let j: isize = j.try_into().unwrap();

            let x = img_x + i - half_dim;
            let y = img_y + j - half_dim;
            if x < 0 || y < 0 {
                continue;
            }

            let x: u32 = x.try_into().unwrap();
            let y: u32 = y.try_into().unwrap();

            if let Some(pixel) = image.get_pixel_mut_checked(x, y) {
                let undo_pixel = pixel.clone();
                pixel.blend(&ImagePixel::from([
                    color.0[0], color.0[1], color.0[2], *value,
                ]));
                undo_pixels.entry((x, y)).or_insert(undo_pixel);
            }
        }
    }
}

/// A simple paint brush operation.
pub struct PaintBrush {
    brush: Brush,
    color: ImagePixel,
}

impl PaintBrush {
    /// Create a new simple brush from a pixel color.
    pub fn new(brush: Brush, color: ImagePixel) -> PaintBrush {
        PaintBrush {
            brush,
            color,
        }
    }

    /// Fill the brush around the coordinates (x, y).
    fn fill(
        &self,
        image: &mut Image,
        x: u32,
        y: u32,
        undo_pixels: &mut HashMap<(u32, u32), ImagePixel>,
    ) {
        let x = x as i32;
        let y = y as i32;
        for (dx, dy, value) in self.brush.iter_values() {
            let img_x = x + dx;
            let img_y = y + dy;

            if img_x < 0 || img_y < 0 {
                return;
            }

            let img_x: u32 = x.try_into().unwrap();
            let img_y: u32 = y.try_into().unwrap();
            if let Some(pixel) = image.get_pixel_mut_checked(img_x, img_y) {
                let undo_pixel = pixel.clone();
                pixel.blend(&ImagePixel::from([
                    self.color.0[0], self.color.0[1], self.color.0[2], value,
                ]));
                undo_pixels.entry((img_x, img_y)).or_insert(undo_pixel);
            }
        }
    }
}

/// Increment factor for the paint brush operation.
const PAINT_BRUSH_INCR_FACTOR: f64 = 0.4;

impl Liner for PaintBrush {
    fn line(
        &mut self,
        image: &mut Image,
        a: (f64, f64),
        b: (f64, f64),
        undo_pixels: &mut HashMap<(u32, u32), ImagePixel>,
    ) {
        let alpha = b.0 - a.0;
        let beta = b.1 - a.1;
        // Determine the length of the line.
        let line_len = (alpha * alpha + beta * beta).sqrt();
        // Based on PAINT_BRUSH_INCR_FACTOR, determine what number of times we
        // should fill the brush radius for a parametric representation of the
        // line.
        let count = line_len / PAINT_BRUSH_INCR_FACTOR;
        let incr = if count > 0.0 { 1.0 / count } else { 1.0 };

        // Now slide along the parametric version of the line.
        let mut t = 0.0;
        while t < 1.0 {
            let x = a.0 + t * alpha;
            let y = a.1 + t * beta;
            if x < 0.0 || y < 0.0 {
                continue;
            }
            let x = x as u32;
            let y = y as u32;
            // fill_dot(image, &self.color, x, y, undo_pixels);
            self.fill(image, x, y, undo_pixels);
            t += incr;
        }
    }
}
