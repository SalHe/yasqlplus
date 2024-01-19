mod error;
mod reader;
mod reed;
mod shell;
mod single;

pub use error::*;
pub use reader::*;
pub use reed::*;
pub use shell::*;
pub use single::*;

use crate::command::Command;

pub const INDICATOR: &str = " > ";
pub const INDICATOR_NORMAL: &str = " : ";

pub trait Input {
    fn get_command(&self) -> Result<Option<(Command, String)>, InputError>;
    fn line(&self, prompt: &str) -> Result<String, InputError>;
}
