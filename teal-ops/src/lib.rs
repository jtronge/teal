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

/// Trait designed to handle a drag operation consisting of a set of lines
/// passed one after another.
pub trait DragHandler {
    /// Handle a drag line.
    fn handle_line(
        &mut self,
        image: &mut Image,
        image_view: &mut ImageView,
        a: (f64, f64),
        b: (f64, f64),
        // undo_pixels: &mut HashMap<(u32, u32), ImagePixel>,
    );

    /// Convert to an undoable/redoable operation.
    fn to_op(&self) -> Option<PixelOp>;
}

/// Stored representation of a drag input gesture.
pub struct DragInput {
    /// First screen point.
    start: Option<(f64, f64)>,

    /// Input drag points (stored as offsets from the first screen point).
    points: Vec<(f64, f64)>,

    /// Original pixels that have been overwritten, for undo operation.
    undo_pixels: HashMap<(u32, u32), ImagePixel>,

    /// Drag handler.
    drag_handler: Box<dyn DragHandler>,
}

impl DragInput {
    /// Create a new drag operation for the current image view.
    pub fn new<D: DragHandler + 'static>(drag_handler: D) -> DragInput {
        DragInput {
            start: None,
            points: vec![],
            undo_pixels: HashMap::new(),
            drag_handler: Box::new(drag_handler),
        }
    }

    /// Add the first point of a drag operation and update the image.
    pub fn start(&mut self, image: &mut Image, start_x: f64, start_y: f64) {
        let _ = self.start.insert((start_x, start_y));
        self.points.push((0.0, 0.0));
    }

    /// Add the next point to the drag operation, updating the image.
    pub fn update(&mut self, image: &mut Image, image_view: &mut ImageView, off_x: f64, off_y: f64) {
        if self.points.len() == 0 {
            panic!("invalid use of BrushOp: start() was not called");
        }

        let (last_off_x, last_off_y) = self.points[self.points.len()-1];
        let a = self.get_image_coords(image, image_view, last_off_x, last_off_y);
        let b = self.get_image_coords(image, image_view, off_x, off_y);
        self.drag_handler.handle_line(image, image_view, a, b);
        self.points.push((off_x, off_y));
    }

    /// Add the final point to the drag operation and update the image.
    pub fn finish(&mut self, image: &mut Image, image_view: &mut ImageView, off_x: f64, off_y: f64) {
        self.update(image, image_view, off_x, off_y);
    }

    /// Get image coordinates for the given offsets.
    ///
    /// NOTE: These could potentially be outside the bounds of the actual image.
    fn get_image_coords(&self, image: &Image, image_view: &ImageView, off_x: f64, off_y: f64) -> (f64, f64) {
        let (start_x, start_y) = self.start
            .as_ref()
            .expect("missing start point");
        let screen_x = start_x + off_x;
        let screen_y = start_y + off_y;
        image_view.get_image_coords_f(image, screen_x, screen_y)
    }

    pub fn to_op(self) -> Option<PixelOp> {
        self.drag_handler.to_op()
    }
}

/// An operation based on updating pixels in the image.
pub struct PixelOp {
    undo_pixels: HashMap<(u32, u32), ImagePixel>,
}

impl Operation for PixelOp {
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

/// A simple paint brush operation.
pub struct PaintBrush {
    brush: Brush,
    color: ImagePixel,
    undo_pixels: HashMap<(u32, u32), ImagePixel>,
}

impl PaintBrush {
    /// Create a new simple brush from a pixel color.
    pub fn new(brush: Brush, color: ImagePixel) -> PaintBrush {
        PaintBrush {
            brush,
            color,
            undo_pixels: HashMap::new(),
        }
    }

    /// Fill the brush around the coordinates (x, y).
    fn fill(
        &mut self,
        image: &mut Image,
        x: i32,
        y: i32,
    ) {
        for (dx, dy, value) in self.brush.iter_values() {
            let img_x = x + dx;
            let img_y = y + dy;

            if img_x < 0 || img_y < 0 {
                continue;
            }

            let img_x: u32 = img_x.try_into().unwrap();
            let img_y: u32 = img_y.try_into().unwrap();
            if let Some(pixel) = image.get_pixel_mut_checked(img_x, img_y) {
                let undo_pixel = pixel.clone();
                pixel.blend(&ImagePixel::from([
                    self.color.0[0], self.color.0[1], self.color.0[2], value,
                ]));
                self.undo_pixels.entry((img_x, img_y)).or_insert(undo_pixel);
            }
        }
    }
}

/// Increment factor for the paint brush operation.
const PAINT_BRUSH_INCR_FACTOR: f64 = 0.4;

impl DragHandler for PaintBrush {
    fn handle_line(
        &mut self,
        image: &mut Image,
        _image_view: &mut ImageView,
        a: (f64, f64),
        b: (f64, f64),
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
            // fill_dot(image, &self.color, x, y, undo_pixels);
            self.fill(image, x as i32, y as i32);
            t += incr;
        }
    }

    fn to_op(&self) -> Option<PixelOp> {
        Some(PixelOp {
            undo_pixels: self.undo_pixels.clone(),
        })
    }
}
