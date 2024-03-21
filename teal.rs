use std::cell::Cell;
use teal_gui::Event;

fn main() {
    let drag_x = Cell::new(0.0);
    let drag_y = Cell::new(0.0);
    teal_gui::gui(move |ctx, ev| {
        match ev {
            Event::KeyPress(key) => {
                println!("key press: {:?}", key);
                None
            }
            Event::DragBegin(x, y) => {
                println!("drag begin: ({}, {})", x, y);
                drag_x.set(x);
                drag_y.set(y);
                None
            }
            Event::DragUpdate(x, y) => {
                println!("drag update: ({}, {})", x, y);
                None
            }
            Event::DragEnd(x, y) => {
                println!("drag end: ({}, {})", x, y);
                None
            }
        }
    });
}
