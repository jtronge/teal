use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use gtk4::subclass::widget::WidgetClassExt;
use gtk4::{Application, ApplicationWindow, GLArea, DrawingArea, Widget, GestureDrag};
use gtk4::cairo;
use gtk4::ShortcutsShortcut;
use glib::signal;

pub fn gui<F>(f: F)
where
    F: Fn(&mut Context, Event) -> Option<()> + 'static,
{
    let app = Application::builder()
        .application_id("org.teal.Teal")
        .build();
    let ctx = Rc::new(RefCell::new(Context {
        surface: None,
    }));
    let f = Rc::new(f);
    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Teal")
            .build();
        let window = Rc::new(window);
        let drawing_area = Rc::new(DrawingArea::new());
        drawing_area.connect_resize({
            let ctx = Rc::clone(&ctx);
            move |area, width, height| {
                ctx.borrow_mut().surface.insert(cairo::ImageSurface::create(cairo::Format::Rgb24, width, height).unwrap());
                println!("resize: width = {}, height = {}", width, height);
            }
        });

        drawing_area.set_draw_func({
            let ctx = Rc::clone(&ctx);
            move |area, cairo_ctx, a, b| {
                println!("drawing...");
                let (x0, y0, x1, y1) = cairo_ctx.clip_extents().unwrap();
                let format = cairo::Format::Rgb24;
                let mut ctx_ref = ctx.borrow_mut();
                {
                    let surface = ctx_ref.surface.as_mut().unwrap();
                    let width = surface.width() as usize;
                    let height = surface.height() as usize;
                    let stride: usize = format.stride_for_width(width.try_into().unwrap()).unwrap().try_into().unwrap();
                    println!("stride: {}", stride);
                    let mut data = surface.data().unwrap();
                    for y in 0..height {
                        let pos = y * stride;
                        for x in 0..width {
                            data[pos + x * 4 + 1] = 255;
                            data[pos + x * 4 + 2] = 0;
                            data[pos + x * 4 + 3] = 255;
                        }
                    }
                }
                cairo_ctx.set_source_surface(&ctx_ref.surface.as_ref().unwrap(), 0.0, 0.0);
                cairo_ctx.paint().unwrap();
            }
        });
        // Handle dragging gestures
        let gesture_drag = GestureDrag::new();
        gesture_drag.connect_drag_begin({
            let f = Rc::clone(&f);
            let drawing_area = Rc::clone(&drawing_area);
            let ctx = Rc::clone(&ctx);
            move |gesture_drag, x, y| {
                drawing_area.queue_draw();
                f(&mut *ctx.borrow_mut(), Event::DragBegin(x, y));
            }
        });
        gesture_drag.connect_drag_update({
            let f = Rc::clone(&f);
            let drawing_area = Rc::clone(&drawing_area);
            let ctx = Rc::clone(&ctx);
            move |gesture_drag, x, y| {
                drawing_area.queue_draw();
                f(&mut *ctx.borrow_mut(), Event::DragUpdate(x, y));
            }
        });
        gesture_drag.connect_drag_end({
            let f = Rc::clone(&f);
            let drawing_area = Rc::clone(&drawing_area);
            let ctx = Rc::clone(&ctx);
            move |gesture_drag, x, y| {
                drawing_area.queue_draw();
                f(&mut *ctx.borrow_mut(), Event::DragEnd(x, y));
            }
        });
        drawing_area.add_controller(gesture_drag);

        window.set_child(Some(&*drawing_area));
        window.present();
    });
    app.run();
}

#[derive(Debug)]
pub enum Key {
    ASCII(char),
}

#[derive(Debug)]
pub enum Event {
    KeyPress(Key),
    DragBegin(f64, f64),
    DragUpdate(f64, f64),
    DragEnd(f64, f64),
}

pub struct Context {
    surface: Option<cairo::ImageSurface>,
}

impl Context {
    pub fn draw(&mut self, image: &teal_data::ImageData) {
    }
}
