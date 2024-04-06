//! Module for handling key input state and commands.
use teal_base::Key;

pub struct CommandState;

impl CommandState {
    pub fn new() -> CommandState {
        CommandState
    }

    /// Update the state with a new key press.
    pub fn handle(&mut self, key: Key) {
        // TODO
    }
}

/// Commands to be executed by the application.
pub enum Command {
    ChooseBrush { quickid: char },
}
