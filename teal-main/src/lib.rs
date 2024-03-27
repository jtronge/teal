//! Teal paint
use std::cell::RefCell;
use std::rc::Rc;
use teal_base::{GUIContext, GUIOptions, Image, ImageView, ImagePixel, GUI, Event, DragEvent, KeyEvent, Key, ScreenBuffer, Operation};
use teal_ops::DragOp;

/// Handle a key event.
fn handle_key_event(key_event: KeyEvent) {
    match key_event {
        KeyEvent::Press(key) => {
            println!("key press: {:?}", key);
        }
        KeyEvent::Release(key) => {
            println!("key release: {:?}", key);
        }
    }
}

/// Current input state.
///
/// This is used to control and store state info about events that are
/// currently being handled.
pub struct InputState {
    /// Holds drag operation, if in progress
    drag: Option<DragOp>,

    /// Holds current key press, removed when released
    key: Option<Key>,

    /// Current color
    color: Option<ImagePixel>
}

/// Application data
pub struct Application {
    /// Acatual image data being operated on
    image: Image,

    /// Image view, tranforming the image for view on the screen
    image_view: ImageView,

    /// Current input state of the system
    input_state: InputState,

    /// Completed operations
    operations: Vec<Box<dyn Operation>>,
}

impl Application {
    fn new() -> Application {
        let image = Image::new(1024, 1024);
        Application {
            image,
            image_view: ImageView::new(),
            input_state: InputState {
                drag: None,
                key: None,
                color: None,
            },
            operations: vec![],
        }
    }

    /// Main event handling function.
    ///
    /// Handles all events coming from the GUI.
    fn handle_event(&mut self, mut ctx: impl GUIContext, event: Event) {
        match event {
            Event::Key(key_event) => {
                handle_key_event(key_event);
            }
            Event::Drag(drag_event) => {
                self.handle_drag_event(drag_event, ctx.screen());
            }
            Event::ColorUpdate { r, g, b, a } => {
                println!("color update: {} {} {} {}", r, g, b, a);
                let _ = self.input_state.color.insert(ImagePixel::from([r, g, b, a]));
            }
            Event::Resize => {
                self.image_view.update_screen(&self.image, ctx.screen());
            }
        }
    }

    /// Handle a drag event.
    fn handle_drag_event(
        &mut self,
        drag_event: DragEvent,
        screen: impl ScreenBuffer,
    ) {
        println!("number of completed operations: {}", self.operations.len());
        match drag_event {
            DragEvent::Begin(start_x, start_y) => {
                let mut drag_op = DragOp::new(self.image_view.clone());
                drag_op.update(&mut self.image, start_x, start_y);
                self.image_view.update_screen(&self.image, screen);
                let _ = self.input_state.drag.insert(drag_op);
            }
            DragEvent::Update(x, y) => {
                let drag_op = self
                    .input_state
                    .drag
                    .as_mut()
                    .expect("encountered unexpected drag update");
                drag_op.update(&mut self.image, x, y);
                self.image_view.update_screen(&self.image, screen);
            }
            DragEvent::End(x, y) => {
                let mut drag_op = self
                    .input_state
                    .drag
                    .take()
                    .expect("encountered unexpected drag end");
                drag_op.update(&mut self.image, x, y);
                // Drag operation completed, save it for undo later
                self.operations.push(Box::new(drag_op));
                self.image_view.update_screen(&self.image, screen);
            }
        }
    }
}

// NOTE: I don't want anything too fancy here; I want something that works and
// that can slowly be refactored to perfection.
pub fn run<G: GUI>(mut gui: G) {
    let app = Rc::new(RefCell::new(Application::new()));

    let options = GUIOptions {};
    // TODO: Simply update the screen with changes to an image made from here
    gui.run(options, move |ctx, event| {
        app.borrow_mut().handle_event(ctx, event);
    });
}
