//! Teal paint
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use teal_base::{
    DragEvent, Event, GUIContext, GUIOptions, Image, ImagePixel, ImageView, Key, KeyEvent,
    ScreenBuffer, GUI, Brush,
};
use teal_ops::{Operation, BrushOp, PaintBrush};

mod config;
pub use config::Config;

/// Current input state.
///
/// This is used to control and store state info about events that are
/// currently being handled.
pub struct InputState {
    /// Holds brush operation, if in progress.
    brush: Option<BrushOp>,

    /// Holds current key press, removed when released.
    key: Option<Key>,

    /// Current color.
    color: Option<ImagePixel>,

    /// Currently selected brush (by quickid).
    selected_brush: Option<char>,
}

/// Application data
pub struct Application {
    /// Actual image data being operated on.
    image: Image,

    /// Image view, tranforming the image for view on the screen.
    image_view: ImageView,

    /// Current input state of the system.
    input_state: InputState,

    /// Completed operations.
    undo_buffer: VecDeque<Box<dyn Operation>>,

    /// Undone operations.
    redo_buffer: VecDeque<Box<dyn Operation>>,

    /// Loaded brushes (<quickid, Brush> pairs).
    brushes: HashMap<char, Brush>,

    /// Config file options.
    config: Config,
}

impl Application {
    /// Create a new application from a config.
    fn new(config: Config) -> Application {
        let image = Image::new(256, 256);

        // Load brushes.
        let mut brushes = HashMap::new();
        for brush_opt in &config.brushes {
            let brush = Brush::new(&brush_opt.name, &brush_opt.file)
                .expect(&format!("failed to load brush: {}", brush_opt.name));
            brushes.insert(brush_opt.quickid, brush);
        }

        Application {
            image,
            image_view: ImageView::new(),
            input_state: InputState {
                brush: None,
                key: None,
                color: None,
                selected_brush: None,
            },
            undo_buffer: VecDeque::new(),
            redo_buffer: VecDeque::new(),
            brushes,
            config,
        }
    }

    /// Main event handling function.
    ///
    /// Handles all events coming from the GUI.
    fn handle_event(&mut self, mut ctx: impl GUIContext, event: Event) {
        match event {
            Event::Key(key_event) => {
                self.handle_key_event(key_event, ctx.screen());
            }
            Event::Drag(drag_event) => {
                self.handle_drag_event(drag_event, ctx.screen());
            }
            Event::ColorUpdate { r, g, b, a } => {
                let _ = self
                    .input_state
                    .color
                    .insert(ImagePixel::from([r, g, b, a]));
            }
            Event::Resize => {
                self.image_view.update_screen(&self.image, ctx.screen());
            }
        }

        // After each event, check whether the undo/redo buffers are too big,
        // and if so drop some operations.
        if self.undo_buffer.len() > self.config.max_undo {
            let new_size = self.config.max_undo / 2;
            let _ = self.undo_buffer.drain(0..new_size);
        }

        if self.redo_buffer.len() > self.config.max_redo {
            let new_size = self.config.max_redo / 2;
            let _ = self.redo_buffer.drain(0..new_size);
        }
    }

    /// Handle a key event.
    fn handle_key_event(&mut self, key_event: KeyEvent, screen: impl ScreenBuffer) {
        match key_event {
            KeyEvent::Press(key) => {
                self.take_key_press_action(key.clone(), screen);
                let _ = self.input_state.key.insert(key);
            }
            KeyEvent::Release(_key) => {
                let _ = self.input_state.key.take();
            }
        }
    }

    /// Take action for various key press sequences.
    fn take_key_press_action(&mut self, key: Key, screen: impl ScreenBuffer) {
        if let Key::Sequence { value, control: _, alt } = key {
            // Check if it's a brush action.
            if alt {
                if self.brushes.get(&value).is_some() {
                    let _ = self.input_state.selected_brush.insert(value);
                } else {
                    eprintln!("no brush for quickid '{}' found", value);
                }
                return;
            }

            // Check for other commands.
            match value {
                // Undo an operation.
                'u' => {
                    if let Some(mut last_op) = self.undo_buffer.pop_back() {
                        last_op.undo(&mut self.image);
                        self.redo_buffer.push_back(last_op);
                        self.image_view.update_screen(&self.image, screen);
                    } else {
                        println!("no more operations to undo");
                    }
                }
                // Redo an operation.
                'U' => {
                    if let Some(mut last_op) = self.redo_buffer.pop_back() {
                        last_op.redo(&mut self.image);
                        self.undo_buffer.push_back(last_op);
                        self.image_view.update_screen(&self.image, screen);
                    } else {
                        println!("no more operations to redo");
                    }
                }
                _ => (),
            }
        }
    }

    /// Handle a drag event.
    fn handle_drag_event(&mut self, drag_event: DragEvent, screen: impl ScreenBuffer) {
        self.do_brush_op(drag_event, screen);
    }

    /// Do a brush operation.
    fn do_brush_op(&mut self, drag_event: DragEvent, screen: impl ScreenBuffer) {
        if self.input_state.selected_brush.is_none() {
            eprintln!("No selected brush found; use 'ALT+<quickid>' to select a brush.");
            return;
        }
        let selected_brush = self.input_state.selected_brush.unwrap();

        match drag_event {
            DragEvent::Begin(start_x, start_y) => {
                let color = if let Some(color) = self.input_state.color.as_ref() {
                    color.clone()
                } else {
                    ImagePixel::from([1.0, 1.0, 1.0, 1.0])
                };
                let brush = self.brushes
                    .get(&selected_brush)
                    .expect("failed to find brush");
                let paint_brush = PaintBrush::new(brush.clone(), color);
                let mut brush_op = BrushOp::new(self.image_view.clone(), paint_brush);
                brush_op.start(&mut self.image, start_x, start_y);
                self.image_view.update_screen(&self.image, screen);
                let _ = self.input_state.brush.insert(brush_op);
            }
            DragEvent::Update(x, y) => {
                let brush_op = self
                    .input_state
                    .brush
                    .as_mut()
                    .expect("encountered unexpected drag update");
                brush_op.update(&mut self.image, x, y);
                self.image_view.update_screen(&self.image, screen);
            }
            DragEvent::End(x, y) => {
                let mut brush_op = self
                    .input_state
                    .brush
                    .take()
                    .expect("encountered unexpected drag end");
                brush_op.finish(&mut self.image, x, y);
                // Drag operation completed, save it for undo later.
                self.undo_buffer.push_back(Box::new(brush_op));
                self.image_view.update_screen(&self.image, screen);
            }
        }
    }
}

// NOTE: I don't want anything too fancy here; I want something that works and
// that can slowly be refactored to perfection.
pub fn run<G: GUI>(mut gui: G, config: Config) {
    let app = Rc::new(RefCell::new(Application::new(config)));

    let options = GUIOptions {};
    // TODO: Simply update the screen with changes to an image made from here
    gui.run(options, move |ctx, event| {
        app.borrow_mut().handle_event(ctx, event);
    });
}
