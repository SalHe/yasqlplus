mod error;
mod shell;
pub use error::*;
pub use shell::*;

use crate::command::Command;

pub trait InputSource {
    fn get_command(&self) -> Result<Option<Command>, InputError>;
    fn line(&self, prompt: &str) -> Result<String, InputError>;
}
