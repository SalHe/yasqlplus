mod error;
mod reader;
mod shell;

pub use error::*;
pub use reader::*;
pub use shell::*;

use crate::command::Command;

pub trait InputSettings {
    fn need_echo(&self) -> bool;
}

pub trait Input: InputSettings {
    fn get_command(&self) -> Result<Option<(Command, String)>, InputError>;
    fn line(&self, prompt: &str) -> Result<String, InputError>;
}
