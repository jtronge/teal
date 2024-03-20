use teal_window::Event;

fn main() {
    teal_window::gui(|ctx, ev| {
        match ev {
            Event::KeyPress(key) => {
                println!("key press: {:?}", key);
            }
        }
        true
    });
}
