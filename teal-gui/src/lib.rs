use glib::signal;
use gtk4::cairo;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, DrawingArea, EventControllerKey, GestureClick, GestureDrag,
};
use std::cell::RefCell;
use std::rc::Rc;
use teal_base::{DragEvent, Event, Key, KeyEvent};

/// Set up the drawing area.
///
/// This is how the image is currently displayed on the screen.
fn create_drawing_area<F>(f: Rc<F>, ctx: Rc<RefCell<Context>>) -> Rc<DrawingArea>
where
    F: Fn(&mut Context, Event) + 'static,
{
    let drawing_area = Rc::new(DrawingArea::new());
    // Save the drawing area for queue_draw() calls later.
    let _ = ctx.borrow_mut().drawing_area.insert(Rc::clone(&drawing_area));

    // Handle a resize.
    drawing_area.connect_resize({
        let ctx = Rc::clone(&ctx);
        let f = Rc::clone(&f);
        move |_, width, height| {
            let surface = cairo::ImageSurface::create(cairo::Format::Rgb24, width, height).unwrap();
            let _ = ctx.borrow_mut().surface.insert(surface);
            f(&mut *ctx.borrow_mut(), Event::Resize);
        }
    });

    // Do actual drawing from the surface stored in the context.
    drawing_area.set_draw_func({
        let ctx = Rc::clone(&ctx);
        move |_, cairo_ctx, _, _| {
            let ctx_ref = ctx.borrow();
            let _ = cairo_ctx.set_source_surface(&ctx_ref.surface.as_ref().unwrap(), 0.0, 0.0);
            let _ = cairo_ctx.paint().unwrap();
        }
    });

    // Handle gestures.
    let gesture_drag =
        create_gesture_drag_handler(Rc::clone(&f), Rc::clone(&ctx), Rc::clone(&drawing_area));
    drawing_area.add_controller(gesture_drag);
    let gesture_click =
        create_gesture_click_handler(Rc::clone(&f), Rc::clone(&ctx), Rc::clone(&drawing_area));
    drawing_area.add_controller(gesture_click);

    // IMPORTANT: hexpand and vexpand are needed to show up in the grid layout
    // later.
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);

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
        move |_gesture_drag, x, y| {
            f(&mut *ctx.borrow_mut(), Event::Drag(DragEvent::Begin(x, y)));
        }
    });
    gesture_drag.connect_drag_update({
        let f = Rc::clone(&f);
        let drawing_area = Rc::clone(&drawing_area);
        let ctx = Rc::clone(&ctx);
        move |_gesture_drag, x, y| {
            f(&mut *ctx.borrow_mut(), Event::Drag(DragEvent::Update(x, y)));
        }
    });
    gesture_drag.connect_drag_end({
        let f = Rc::clone(&f);
        let drawing_area = Rc::clone(&drawing_area);
        let ctx = Rc::clone(&ctx);
        move |_gesture_drag, x, y| {
            f(&mut *ctx.borrow_mut(), Event::Drag(DragEvent::End(x, y)));
        }
    });

    gesture_drag
}

/// Create a gesture click handler
fn create_gesture_click_handler<F>(
    _f: Rc<F>,
    _ctx: Rc<RefCell<Context>>,
    _drawing_area: Rc<DrawingArea>,
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

/// Create the key handler.
fn create_key_handler<F>(f: Rc<F>, ctx: Rc<RefCell<Context>>) -> EventControllerKey
where
    F: Fn(&mut Context, Event) + 'static,
{
    let key_handler = EventControllerKey::new();

    // Set up short cuts.
    key_handler.connect_key_pressed({
        let f = Rc::clone(&f);
        let ctx = Rc::clone(&ctx);
        move |_, key, _, modifier| {
            if let Some(key) = parse_key(key, modifier) {
                f(&mut *ctx.borrow_mut(), Event::Key(KeyEvent::Press(key)));
            }
            signal::Propagation::Proceed
        }
    });

    key_handler.connect_key_released({
        let f = Rc::clone(&f);
        let ctx = Rc::clone(&ctx);
        move |_, key, _, modifier| {
            if let Some(key) = parse_key(key, modifier) {
                f(&mut *ctx.borrow_mut(), Event::Key(KeyEvent::Release(key)));
            }
        }
    });

    key_handler
}

/// Create the main color picker.
fn create_color_picker<F>(f: Rc<F>, ctx: Rc<RefCell<Context>>) -> gtk4::ColorButton
where
    F: Fn(&mut Context, Event) + 'static,
{
    // NOTE: ColorButton is deprecated, need to use ColorDialogButton,
    // but only available for newer versions of gtk 4.x.
    let color_button = gtk4::ColorButton::new();

    color_button.connect_color_set({
        let ctx = Rc::clone(&ctx);
        let f = Rc::clone(&f);
        move |color_button| {
            let rgba = color_button.rgba();
            let event = Event::ColorUpdate {
                r: rgba.red(),
                g: rgba.green(),
                b: rgba.blue(),
                a: rgba.alpha(),
            };
            f(&mut *ctx.borrow_mut(), event);
        }
    });

    color_button
}

pub struct GtkGUI;

impl GtkGUI {
    pub fn new() -> GtkGUI {
        GtkGUI
    }
}

impl teal_base::GUI for GtkGUI {
    type Context<'a> = &'a mut Context;

    fn run<F>(&mut self, _options: teal_base::GUIOptions, f: F)
    where
        F: Fn(Self::Context<'_>, Event) + 'static,
    {
        let app = Application::builder()
            .application_id("org.teal.Teal")
            .build();
        let ctx = Rc::new(RefCell::new(Context { drawing_area: None, surface: None }));
        let f = Rc::new(f);

        app.connect_activate(move |app| {
            let grid = gtk4::Grid::new();
            let drawing_area = create_drawing_area(Rc::clone(&f), Rc::clone(&ctx));
            grid.attach(&*drawing_area, 0, 0, 10, 10);
            let color_picker = create_color_picker(Rc::clone(&f), Rc::clone(&ctx));
            color_picker.set_valign(gtk4::Align::Start);
            let label = gtk4::Label::new(Some("testo"));
            let label2 = gtk4::Label::new(Some("testo2"));
            let box_layout = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
            box_layout.append(&color_picker);
            box_layout.append(&label);
            box_layout.append(&label2);
            grid.attach(&box_layout, 10, 0, 1, 1);

            let window = ApplicationWindow::builder()
                .application(app)
                .title("Teal")
                .build();
            let window = Rc::new(window);
            let key_handler = create_key_handler(Rc::clone(&f), Rc::clone(&ctx));
            window.add_controller(key_handler);
            window.set_child(Some(&grid));
            window.present();
        });

        app.run();
    }
}

pub struct Context {
    drawing_area: Option<Rc<DrawingArea>>,
    surface: Option<cairo::ImageSurface>,
}

impl teal_base::GUIContext for &mut Context {
    /// Produce the screen type that can be used by the backend.
    fn screen(&mut self) -> impl teal_base::ScreenBuffer {
        // Always queue a draw when a screen is returned.
        self.drawing_area.as_ref().unwrap().queue_draw();

        let format = cairo::Format::Rgb24;
        let surface = self.surface.as_mut().unwrap();
        let stride = format
            .stride_for_width(surface.width().try_into().unwrap())
            .unwrap();
        Screen {
            width: surface.width().try_into().unwrap(),
            height: surface.height().try_into().unwrap(),
            stride: stride.try_into().unwrap(),
            surface_data: surface.data().unwrap(),
        }
    }
}

/// Screen type that can be updated by the backend.
pub struct Screen<'a> {
    width: u32,
    height: u32,
    stride: u32,
    surface_data: cairo::ImageSurfaceData<'a>,
}

impl<'a> teal_base::ScreenBuffer for Screen<'a> {
    #[inline]
    fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    fn set(&mut self, x: u32, y: u32, pixel: teal_base::DisplayPixel) {
        let pos: usize = (y * self.stride).try_into().unwrap();
        let x: usize = x.try_into().unwrap();
        self.surface_data[pos + x * 4] = pixel.b;
        self.surface_data[pos + x * 4 + 1] = pixel.g;
        self.surface_data[pos + x * 4 + 2] = pixel.r;
    }
}
