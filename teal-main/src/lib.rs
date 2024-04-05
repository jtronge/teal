//! Teal paint
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::process::ExitCode;
use std::rc::Rc;
use teal_base::{
    Brush, DragEvent, Event, GUIContext, GUIOptions, Image, ImagePixel, ImageView, Key, KeyEvent,
    ScreenBuffer, GUI,
};
use teal_ops::{DragInput, Operation, PaintBrush, ViewDragHandler};

mod config;
pub use config::Config;
mod command;

/// CLI arguments.
pub struct Args {
    pub fname: String,
    pub dims: Option<(u32, u32)>,
}

/// Current input state.
///
/// This is used to control and store state info about events that are
/// currently being handled.
pub struct InputState {
    /// Holds in-progress drag operation.
    drag: Option<DragInput>,

    /// Holds current key press, removed when released.
    key: Option<Key>,

    /// Command state handling incoming key presses.
    command: command::CommandState,

    /// Current color.
    color: Option<ImagePixel>,

    /// Currently selected brush (by quickid).
    selected_brush: Option<char>,
}

/// Application data
pub struct Application {
    /// Image path.
    image_path: PathBuf,

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
    fn new(args: Args, config: Config) -> Application {
        let image_path = PathBuf::from(args.fname);
        let image = if let Some(image) = teal_base::load_image(&image_path) {
            if config.backup {
                // Make a backup of the old image.
                let ext = image_path
                    .extension()
                    .expect("image is missing an extension")
                    .to_str()
                    .expect("failed to decode extension into unicode string")
                    .to_string();
                let mut backup_path = image_path.clone();
                backup_path.set_extension(format!("teal_backup.{ext}"));
                image
                    .save(&backup_path)
                    .expect("failed to save backup image");
            }
            image
        } else {
            if let Some((width, height)) = args.dims {
                Image::new(width, height)
            } else {
                panic!("missing width and height dimensions for creating new image");
            }
        };

        // Load brushes.
        let mut brushes = HashMap::new();
        for brush_opt in &config.brushes {
            let brush = Brush::new(&brush_opt.name, &brush_opt.file)
                .expect(&format!("failed to load brush: {}", brush_opt.name));
            brushes.insert(brush_opt.quickid, brush);
        }

        Application {
            image_path,
            image,
            image_view: ImageView::new(),
            input_state: InputState {
                drag: None,
                command: command::CommandState::new(),
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
                self.input_state.key_state.handle(key.clone());
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
        if let Key::Sequence {
            value,
            control: _,
            alt,
        } = key
        {
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
                'r' => {
                    if let Some(mut last_op) = self.redo_buffer.pop_back() {
                        last_op.redo(&mut self.image);
                        self.undo_buffer.push_back(last_op);
                        self.image_view.update_screen(&self.image, screen);
                    } else {
                        println!("no more operations to redo");
                    }
                }
                // Save the image.
                's' => {
                    self.image
                        .save(&self.image_path)
                        .expect("failed to save image");
                }
                // Zoom in some.
                'z' => {
                    self.image_view.zoom_in(screen.width(), screen.height());
                    self.image_view.update_screen(&self.image, screen);
                }
                // Zoom out some.
                'x' => {
                    self.image_view.zoom_out(screen.width(), screen.height());
                    self.image_view.update_screen(&self.image, screen);
                }
                _ => (),
                _ => (),
            }
        }
    }

    /// Create the drag input handler.
    fn create_drag_input(&self) -> Option<DragInput> {
        if let Some(Key::PlainControl) = self.input_state.key {
            // This needs a drag handler that will translate the view.
            let view_handler = ViewDragHandler::new();
            Some(DragInput::new(view_handler))
        } else {
            // Create an image operation drag handler.
            if self.input_state.selected_brush.is_none() {
                eprintln!("No selected brush found; use 'ALT+<quickid>' to select a brush.");
                return None;
            }
            let selected_brush = self.input_state.selected_brush.unwrap();
            let brush = self
                .brushes
                .get(&selected_brush)
                .expect("failed to find brush");
            let color = if let Some(color) = self.input_state.color.as_ref() {
                color.clone()
            } else {
                ImagePixel::from([1.0, 1.0, 1.0, 1.0])
            };
            let paint_brush = PaintBrush::new(brush.clone(), color);
            Some(DragInput::new(paint_brush))
        }
    }

    /// Handle a drag event.
    fn handle_drag_event(&mut self, drag_event: DragEvent, screen: impl ScreenBuffer) {
        match drag_event {
            DragEvent::Begin(start_x, start_y) => {
                // First create drag input and handler.
                if let Some(mut drag) = self.create_drag_input() {
                    drag.start(&mut self.image, start_x, start_y);
                    self.image_view.update_screen(&self.image, screen);
                    let _ = self.input_state.drag.insert(drag);
                }
            }
            DragEvent::Update(x, y) => {
                if let Some(drag) = self.input_state.drag.as_mut() {
                    drag.update(&mut self.image, &mut self.image_view, x, y);
                    self.image_view.update_screen(&self.image, screen);
                }
            }
            DragEvent::End(x, y) => {
                if let Some(mut drag) = self.input_state.drag.take() {
                    drag.finish(&mut self.image, &mut self.image_view, x, y);
                    // Drag input complete, save it for undo later, if necessary.
                    if let Some(drag_op) = drag.to_op() {
                        self.undo_buffer.push_back(Box::new(drag_op));
                    }
                    self.image_view.update_screen(&self.image, screen);
                }
            }
        }
    }
}

// NOTE: I don't want anything too fancy here; I want something that works and
// that can slowly be refactored to perfection.
pub fn run<G: GUI>(args: Args, config: Config, mut gui: G) -> ExitCode {
    let app = Rc::new(RefCell::new(Application::new(args, config)));

    let options = GUIOptions {};
    // TODO: Simply update the screen with changes to an image made from here
    gui.run(options, move |ctx, event| {
        app.borrow_mut().handle_event(ctx, event);
    })
}
