use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use gtk4::subclass::widget::WidgetClassExt;
use gtk4::{Application, ApplicationWindow, GLArea, DrawingArea, Widget, GestureDrag, EventControllerKey};
use gtk4::cairo;
use gtk4::ShortcutsShortcut;
use glib::signal;

/// Set up the drawing area.
///
/// This is how the image is currently displayed on the screen.
fn create_drawing_area<F>(f: Rc<F>, ctx: Rc<RefCell<Context>>) -> Rc<DrawingArea>
where
    F: Fn(&mut Context, Event) -> Option<()> + 'static,
{
    let drawing_area = Rc::new(DrawingArea::new());

    // Handle a resize
    drawing_area.connect_resize({
        let ctx = Rc::clone(&ctx);
        let f = Rc::clone(&f);
        move |area, width, height| {
            let surface = cairo::ImageSurface::create(
                cairo::Format::Rgb24,
                width,
                height,
            ).unwrap();
            ctx.borrow_mut().surface.insert(surface);
            f(&mut *ctx.borrow_mut(), Event::Resize);
        }
    });

    // Do actual drawing from the surface stored in the context
    drawing_area.set_draw_func({
        let ctx = Rc::clone(&ctx);
        move |area, cairo_ctx, a, b| {
            let mut ctx_ref = ctx.borrow();
            cairo_ctx.set_source_surface(
                &ctx_ref.surface.as_ref().unwrap(),
                0.0,
                0.0,
            );
            cairo_ctx.paint().unwrap();
        }
    });

    drawing_area
}


/// Setup controller for gesture dragging.
fn create_gesture_drag_handler<F>(
    f: Rc<F>,
    ctx: Rc<RefCell<Context>>,
    drawing_area: Rc<DrawingArea>,
) -> GestureDrag
where
    F: Fn(&mut Context, Event) -> Option<()> + 'static,
{
    let gesture_drag = GestureDrag::new();

    gesture_drag.connect_drag_begin({
        let f = Rc::clone(&f);
        let drawing_area = Rc::clone(&drawing_area);
        let ctx = Rc::clone(&ctx);
        move |gesture_drag, x, y| {
            f(&mut *ctx.borrow_mut(), Event::DragBegin(x, y));
            drawing_area.queue_draw();
        }
    });
    gesture_drag.connect_drag_update({
        let f = Rc::clone(&f);
        let drawing_area = Rc::clone(&drawing_area);
        let ctx = Rc::clone(&ctx);
        move |gesture_drag, x, y| {
            f(&mut *ctx.borrow_mut(), Event::DragUpdate(x, y));
            drawing_area.queue_draw();
        }
    });
    gesture_drag.connect_drag_end({
        let f = Rc::clone(&f);
        let drawing_area = Rc::clone(&drawing_area);
        let ctx = Rc::clone(&ctx);
        move |gesture_drag, x, y| {
            f(&mut *ctx.borrow_mut(), Event::DragEnd(x, y));
            drawing_area.queue_draw();
        }
    });

    gesture_drag
}

/// Create the key handler
fn create_key_handler<F>(f: Rc<F>, ctx: Rc<RefCell<Context>>) -> EventControllerKey
where
    F: Fn(&mut Context, Event) -> Option<()> + 'static,
{
    let key_handler = EventControllerKey::new();

    // Set up short cuts
    key_handler.connect_key_pressed(move |_, key, x, modifier| {
        println!("key pressed...: {:?}, {:?}, {:?}", key, x, modifier);
        if let Some(value) = key.to_unicode() {
            println!("value: {}", value);
            let control = modifier.contains(gdk4::ModifierType::CONTROL_MASK);
            let alt = modifier.contains(gdk4::ModifierType::ALT_MASK);
            f(&mut *ctx.borrow_mut(), Event::KeyPress(Key::Sequence {
                value,
                control,
                alt,
            }));
        } else {
            if key == gdk4::Key::Control_L || key == gdk4::Key::Control_R {
                f(&mut *ctx.borrow_mut(), Event::KeyPress(Key::PlainControl));
            }
            if key == gdk4::Key::Alt_L || key == gdk4::Key::Alt_R {
                f(&mut *ctx.borrow_mut(), Event::KeyPress(Key::PlainAlt));
            }
        }
        signal::Propagation::Proceed
    });

    key_handler
}

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
        let drawing_area = create_drawing_area(Rc::clone(&f), Rc::clone(&ctx));
        let gesture_drag = create_gesture_drag_handler(
            Rc::clone(&f),
            Rc::clone(&ctx),
            Rc::clone(&drawing_area),
        );
        drawing_area.add_controller(gesture_drag);
        let key_handler = create_key_handler(Rc::clone(&f), Rc::clone(&ctx));
        window.add_controller(key_handler);
        window.set_child(Some(&*drawing_area));
        window.present();
    });

    app.run();
}

#[derive(Debug)]
pub enum Key {
    /// An entered key sequence with possible modifiers
    Sequence {
        value: char,
        control: bool,
        alt: bool,
    },

    /// Control key pressed by itself
    PlainControl,

    /// Alt key pressed by itself
    PlainAlt,
}

#[derive(Debug)]
pub enum Event {
    /// A key press event
    KeyPress(Key),

    /// Start of a drag gesture
    DragBegin(f64, f64),

    /// Update drag gesture
    DragUpdate(f64, f64),

    /// Finish drag gesture
    DragEnd(f64, f64),

    /// Window resize
    Resize,
}

pub struct Context {
    surface: Option<cairo::ImageSurface>,
}

impl teal_data::DisplayBuffer for &mut Context {
    fn width(&self) -> usize {
        self.surface.as_ref().unwrap().width() as _
    }

    fn height(&self) -> usize {
        self.surface.as_ref().unwrap().height() as _
    }

    fn set(&mut self, x: usize, y: usize, pixel: teal_data::DisplayPixel) {
        let format = cairo::Format::Rgb24;
        let stride: usize = format
            .stride_for_width(self.width().try_into().unwrap())
            .unwrap()
            .try_into()
            .unwrap();
        let surface = self.surface.as_mut().unwrap();
        let mut data = surface.data().unwrap();
        let pos = y * stride;
        data[pos + x * 4 + 1] = pixel.r;
        data[pos + x * 4 + 2] = pixel.g;
        data[pos + x * 4 + 3] = pixel.b;
    }
}
