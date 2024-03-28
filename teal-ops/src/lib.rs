use teal_base::image::Pixel;
use teal_base::{Image, ImagePixel, ImageView};

/// An operation to be applied to an image.
pub trait Operation {
    /// Run the operation
    fn execute(&self, image: &mut Image);

    /// Undo the operation
    fn unexecute(&self, image: &mut Image);
}

/// Trait to abstract application of some operation that can be applied on a
/// series of lines on a canvas.
///
/// This could be for lines of paint or for more special operations like smudge
/// and blur.
pub trait Brush {
    /// Brush a line from point a to point b.
    fn line(
        &mut self,
        image: &mut Image,
        a: (f64, f64),
        b: (f64, f64),
        undo_pixels: &mut Vec<ImagePixel>,
    );
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
    undo_pixels: &mut Vec<(u32, u32, ImagePixel)>,
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
                undo_pixels.push((x, y, undo_pixel));
            }
        }
    }
}

/// Stored representation of a drag gesture
pub struct DragOp {
    /// Saved image view
    image_view: ImageView,

    /// First screen point
    start: Option<(f64, f64)>,

    /// Input drag points (stored as offsets from the first screen point)
    points: Vec<(f64, f64)>,

    /// Operation color
    color: ImagePixel,

    /// Pixels for undo operation
    undo_pixels: Vec<(u32, u32, ImagePixel)>,
}

impl DragOp {
    /// Create a new drag operation for the current image view.
    pub fn new(image_view: ImageView, color: ImagePixel) -> DragOp {
        DragOp {
            image_view,
            start: None,
            points: vec![],
            color,
            undo_pixels: vec![],
        }
    }

    /// Add the first point of a drag operation and update the image.
    pub fn start(&mut self, image: &mut Image, start_x: f64, start_y: f64) {
        if let Some((img_x, img_y)) =
            self.image_view
                .get_image_coords(image, start_x as u32, start_y as u32)
        {
            fill_dot(image, &self.color, img_x, img_y, &mut self.undo_pixels);
        }

        let _ = self.start.insert((start_x, start_y));
        self.points.push((0.0, 0.0));
    }

    /// Add the next point to the drag operation, updating the image.
    pub fn update(&mut self, image: &mut Image, off_x: f64, off_y: f64) {
        let (start_x, start_y) = self.start
            .expect("missing start point in drag operations");
        let last_i = self.points.len() - 1;
        let (last_x_off, last_y_off) = self.points[last_i];
        let last_x = start_x + last_x_off;
        let last_y = start_y + last_y_off;
        let cur_x = start_x + off_x;
        let cur_y = start_y + off_y;

        if let Some((img_x, img_y)) =
            self.image_view.get_image_coords(image, cur_x as u32, cur_y as u32)
        {
            fill_dot(image, &self.color, img_x, img_y, &mut self.undo_pixels);
        }

        self.points.push((off_x, off_y));
    }

    /// Add the final point to the drag operation and update the image.
    pub fn finish(&mut self, image: &mut Image, off_x: f64, off_y: f64) {
        self.update(image, off_x, off_y);
    }
}

impl Operation for DragOp {
    fn execute(&self, _image: &mut Image) {
        // TODO
    }

    fn unexecute(&self, _image: &mut Image) {
        // TODO
    }
}
