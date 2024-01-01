use std::cell::Cell;

use crate::command::parse_command;

use super::{Input, InputError};

pub struct SingleCommand {
    command: String,
    gotten: Cell<bool>,
}

impl SingleCommand {
    pub fn new(command: String) -> Self {
        Self {
            command,
            gotten: Cell::new(false),
        }
    }
}

impl Input for SingleCommand {
    fn get_command(&self) -> Result<Option<(crate::command::Command, String)>, InputError> {
        if !self.gotten.get() {
            self.gotten.set(true);
            parse_command(&self.command)
                .map(|x| Some((x, self.command.clone())))
                .map_err(Into::into)
        } else {
            Err(InputError::Eof)
        }
    }

    fn line(&self, _prompt: &str) -> Result<String, InputError> {
        unimplemented!("Unsupported!")
    }
}
