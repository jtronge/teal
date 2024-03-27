use teal_base::{Operation, Image, ImageView, ImagePixel};
use teal_base::image::Pixel;

const ROUND_BRUSH: [[f32; 4]; 4] = [
    [0.0, 0.4, 0.4, 0.0],
    [0.4, 1.0, 1.0, 0.4],
    [0.4, 1.0, 1.0, 0.4],
    [0.0, 0.4, 0.4, 0.0],
];

/// Stored representation of a drag gesture
pub struct DragOp {
    /// Saved image view
    image_view: ImageView,

    /// Input drag points (stored as screen points)
    points: Vec<(f64, f64)>,

    /// Pixels for undo operation
    undo_pixels: Vec<(u32, u32, ImagePixel)>,
}

/// Fill in a dot around the point from the
fn fill_dot(image: &mut Image, img_x: u32, img_y: u32, undo_pixels: &mut Vec<(u32, u32, ImagePixel)>) {
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
                pixel.blend(&ImagePixel::from([0.0, 0.0, 1.0, *value]));
                undo_pixels.push((x, y, undo_pixel));
            }
        }
    }
}

impl DragOp {
    /// Create a new drag operation for the current image view.
    pub fn new(image_view: ImageView) -> DragOp {
        DragOp {
            image_view,
            points: vec![],
            undo_pixels: vec![],
        }
    }

    /// Add the next point to the drag operation, updating the image.
    pub fn update(&mut self, image: &mut Image, off_x: f64, off_y: f64) {
        let (screen_x, screen_y) = if let Some((start_x, start_y)) = self.points.get(0) {
            (start_x + off_x, start_y + off_y)
        } else {
            // These are the start points
            (off_x, off_y)
        };

        if let Some((img_x, img_y)) = self.image_view.get_image_coords(
            image,
            screen_x as _,
            screen_y as _,
        ) {
            fill_dot(image, img_x, img_y, &mut self.undo_pixels);
        }

        self.points.push((off_x, off_y));
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
