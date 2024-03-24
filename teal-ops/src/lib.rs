use teal_base::{Image, ImageView, Operation};

/// Stored representation of a drag gesture
pub struct DragOp {
    /// Saved image view
    image_view: ImageView,

    /// Input drag points (stored as screen points)
    points: Vec<(f64, f64)>,
}

impl DragOp {
    /// Create a new drag operation for the current image view.
    pub fn new(image_view: ImageView) -> DragOp {
        DragOp {
            image_view,
            points: vec![],
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
            image.set(
                img_x,
                img_y,
                teal_base::Pixel {
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                    a: 1.0,
                },
            );
        }

        self.points.push((off_x, off_y));
    }
}

impl Operation for DragOp {
    fn execute(&self, image: &mut Image) {
        // TODO
    }

    fn unexecute(&self, image: &mut Image) {
        // TODO
    }
}
