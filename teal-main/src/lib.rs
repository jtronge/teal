//! Teal paint
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use teal_base::{GUIContext, Image, ImageView, GUI, Pixel, Event};

fn fill(image: &mut Image, image_view: &ImageView, screen_x: usize, screen_y: usize) {
    if let Some((img_x, img_y)) = image_view.get_image_coords(
        image,
        screen_x,
        screen_y,
    ) {
        image.set(
            img_x,
            img_y,
            Pixel {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            },
        );
    }
}

// NOTE: I don't want anything too fancy here; I want something that works and
// that can slowly be refactored to perfection.
pub fn run<G: GUI>(mut gui: G) {
    let drag_x = Cell::new(0.0);
    let drag_y = Cell::new(0.0);
    let image_view = ImageView::new();
    let mut image: Rc<RefCell<Image>> = Rc::new(RefCell::new(Image::new(
        1024,
        1024,
        Pixel {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
    )));

    // TODO: Simply update the screen with changes to an image made from here
    gui.run(move |mut display, ev| {
        let mut image = image.borrow_mut();
        match ev {
            Event::KeyPress(key) => {
                println!("key press: {:?}", key);
            }
            Event::KeyRelease(key) => {
                println!("key release: {:?}", key);
            }
            Event::DragBegin(x, y) => {
                drag_x.set(x);
                drag_y.set(y);
                fill(&mut *image, &image_view, x as _, y as _);
                image_view.update_screen(&*image, display.screen());
            }
            Event::DragUpdate(x, y) => {
                let x = drag_x.get() + x;
                let y = drag_y.get() + y;
                fill(&mut *image, &image_view, x as _, y as _);
                image_view.update_screen(&*image, display.screen());
            }
            Event::DragEnd(x, y) => {
                let x = drag_x.get() + x;
                let y = drag_y.get() + y;
                fill(&mut *image, &image_view, x as _, y as _);
                image_view.update_screen(&*image, display.screen());
            }
            Event::Resize => {
                image_view.update_screen(&*image, display.screen());
            }
        }
    });
}
