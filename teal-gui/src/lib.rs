use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use gtk4::subclass::widget::WidgetClassExt;
use gtk4::{Application, ApplicationWindow, GLArea, DrawingArea, Widget, GestureDrag, GestureClick, EventControllerKey};
use gtk4::cairo;
use gtk4::ShortcutsShortcut;
use glib::signal;
use teal_base::{Key, Event};

/// Set up the drawing area.
///
/// This is how the image is currently displayed on the screen.
fn create_drawing_area<F>(f: Rc<F>, ctx: Rc<RefCell<Context>>) -> Rc<DrawingArea>
where
    F: Fn(&mut Context, Event) + 'static,
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
    F: Fn(&mut Context, Event) + 'static,
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

/// Create a gesture click handler
fn create_gesture_click_handler<F>(
    f: Rc<F>,
    ctx: Rc<RefCell<Context>>,
    drawing_area: Rc<DrawingArea>,
) -> GestureClick {
    let gesture_click = GestureClick::new();
    gesture_click.connect_pressed(|_, i, a, b| {
        println!("click pressed: {}, {}, {}", i, a, b);
    });
    gesture_click.connect_released(|_, i, a, b| {
        println!("click released: {}, {}, {}", i, a, b);
    });
    gesture_click
}

fn parse_key(key: gdk4::Key, modifier: gdk4::ModifierType) -> Option<Key> {
    if let Some(value) = key.to_unicode() {
        let control = modifier.contains(gdk4::ModifierType::CONTROL_MASK);
        let alt = modifier.contains(gdk4::ModifierType::ALT_MASK);
        Some(Key::Sequence {
            value,
            control,
            alt,
        })
    } else {
        if key == gdk4::Key::Control_L || key == gdk4::Key::Control_R {
            Some(Key::PlainControl)
        } else if key == gdk4::Key::Alt_L || key == gdk4::Key::Alt_R {
            Some(Key::PlainAlt)
        } else {
            None
        }
    }
}

/// Create the key handler
fn create_key_handler<F>(f: Rc<F>, ctx: Rc<RefCell<Context>>) -> EventControllerKey
where
    F: Fn(&mut Context, Event) + 'static,
{
    let key_handler = EventControllerKey::new();

    // Set up short cuts
    key_handler.connect_key_pressed({
        let f = Rc::clone(&f);
        let ctx = Rc::clone(&ctx);
        move |_, key, _, modifier| {
            if let Some(key) = parse_key(key, modifier) {
                f(&mut *ctx.borrow_mut(), Event::KeyPress(key));
            }
            signal::Propagation::Proceed
        }
    });

    key_handler.connect_key_released({
        let f = Rc::clone(&f);
        let ctx = Rc::clone(&ctx);
        move |_, key, _, modifier| {
            if let Some(key) = parse_key(key, modifier) {
                f(&mut *ctx.borrow_mut(), Event::KeyRelease(key));
            }
        }
    });

    key_handler
}

pub struct GtkGUI;

impl GtkGUI {
    pub fn new() -> GtkGUI {
        GtkGUI
    }
}

impl teal_base::GUI for GtkGUI {
    type Context<'a> = &'a mut Context;

    fn run<F>(&mut self, f: F)
    where
        F: Fn(Self::Context<'_>, Event) + 'static,
    {
        let app = Application::builder()
            .application_id("org.teal.Teal")
            .build();
        let ctx = Rc::new(RefCell::new(Context {
            surface: None,
        }));
        let f = Rc::new(f);

        app.connect_activate(move |app| {
            // Set up the drawing area and attach controllers
            let drawing_area = create_drawing_area(Rc::clone(&f), Rc::clone(&ctx));
            let gesture_drag = create_gesture_drag_handler(
                Rc::clone(&f),
                Rc::clone(&ctx),
                Rc::clone(&drawing_area),
            );
            drawing_area.add_controller(gesture_drag);
            let gesture_click = create_gesture_click_handler(
                Rc::clone(&f),
                Rc::clone(&ctx),
                Rc::clone(&drawing_area),
            );
            drawing_area.add_controller(gesture_click);

            let window = ApplicationWindow::builder()
                .application(app)
                .title("Teal")
                .build();
            let window = Rc::new(window);
            let key_handler = create_key_handler(Rc::clone(&f), Rc::clone(&ctx));
            window.add_controller(key_handler);
            window.set_child(Some(&*drawing_area));
            window.present();
        });

        app.run();
    }
}

pub struct Context {
    surface: Option<cairo::ImageSurface>,
}

impl teal_base::GUIContext for &mut Context {
    /// Produce the screen type that can be used by the backend.
    fn screen(&mut self) -> impl teal_base::ScreenBuffer {
        let format = cairo::Format::Rgb24;
        let surface = self.surface.as_mut().unwrap();
        let stride: usize = format
            .stride_for_width(surface.width().try_into().unwrap())
            .unwrap()
            .try_into()
            .unwrap();
        Screen {
            width: surface.width().try_into().unwrap(),
            height: surface.height().try_into().unwrap(),
            stride,
            surface_data: surface.data().unwrap(),
        }
    }
}

/// Screen type that can be updated by the backend.
pub struct Screen<'a> {
    width: usize,
    height: usize,
    stride: usize,
    surface_data: cairo::ImageSurfaceData<'a>,
}

impl<'a> teal_base::ScreenBuffer for Screen<'a> {
    #[inline]
    fn width(&self) -> usize {
        self.width
    }

    #[inline]
    fn height(&self) -> usize {
        self.height
    }

    #[inline]
    fn set(&mut self, x: usize, y: usize, pixel: teal_base::DisplayPixel) {
        let pos = y * self.stride;
        self.surface_data[pos + x * 4] = pixel.b;
        self.surface_data[pos + x * 4 + 1] = pixel.g;
        self.surface_data[pos + x * 4 + 2] = pixel.r;
    }
}
