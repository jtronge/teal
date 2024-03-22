//! Teal paint
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use teal_data::{DisplayBuffer, Image, ImageData, ImageDisplay, Pixel};
use teal_gui::Event;

fn fill(image: &mut Image, disp_x: usize, disp_y: usize) {
    let (img_x, img_y) = image.image_coord(disp_x as _, disp_y as _);
    image.data.set(
        img_x as _,
        img_y as _,
        Pixel {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        },
    );
    /*
        let (img_x, img_y) = image.display_to_image(
            disp_width,
            disp_height,
            disp_x,
            disp_y,
        );
        image.set(img_x, img_y, Pixel { r: 0.0, g: 0.0, b: 1.0, a: 1.0});
    */
}

fn update_display(mut display: impl DisplayBuffer, image: &Image) {
    // TODO
    let width = display.width();
    let height = display.height();
    for x in 0..width {
        for y in 0..height {
            let disp_pixel = image.get_display_pixel(x as _, y as _);
            display.set(x, y, disp_pixel);
        }
    }
}

// NOTE: I don't want anything too fancy here; I want something that works and
// that can slowly be refactored to perfection.
fn main() {
    let drag_x = Cell::new(0.0);
    let drag_y = Cell::new(0.0);
    let mut image: Rc<RefCell<Option<Image>>> = Rc::new(RefCell::new(None));

    // TODO: Simply update the screen with changes to an image made from here
    teal_gui::gui(move |display, ev| {
        let disp_width = display.width();
        let disp_height = display.height();
        let mut image = image.borrow_mut();
        if image.is_none() {
            let image_data = ImageData::new(
                1024,
                1024,
                Pixel {
                    r: 1.0,
                    g: 0.0,
                    b: 0.3,
                    a: 1.0,
                },
            );
            let image_disp = ImageDisplay::new(disp_width as _, disp_height as _);
            image.insert(Image::new(image_data, image_disp));
        }
        let image_ref = image.as_mut().unwrap();
        match ev {
            Event::KeyPress(key) => {
                println!("key press: {:?}", key);
                None
            }
            Event::DragBegin(x, y) => {
                drag_x.set(x);
                drag_y.set(y);
                fill(&mut *image_ref, x as _, y as _);
                update_display(display, &*image_ref);
                None
            }
            Event::DragUpdate(x, y) => {
                let x = drag_x.get() + x;
                let y = drag_y.get() + y;
                fill(&mut *image_ref, x as _, y as _);
                update_display(display, &*image_ref);
                None
            }
            Event::DragEnd(x, y) => {
                let x = drag_x.get() + x;
                let y = drag_y.get() + y;
                fill(&mut *image_ref, x as _, y as _);
                update_display(display, &*image_ref);
                None
            }
            Event::Resize => {
                update_display(display, &*image_ref);
                None
            }
        }
    });
}
