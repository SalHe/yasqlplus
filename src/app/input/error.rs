use rustyline::error::ReadlineError;

use crate::command::ParseError;

#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("No more commands.")]
    Eof,

    #[error("Input cancelled.")]
    Cancelled,

    #[error("Error occurs when parsing command: {0}")]
    Parse(#[from] ParseError),

    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unknown error")]
    Unknown(#[from] Box<dyn std::error::Error>),
}

impl From<ReadlineError> for InputError {
    fn from(value: ReadlineError) -> Self {
        match value {
            ReadlineError::Io(err) => Self::Io(err),
            ReadlineError::Eof => Self::Eof,
            ReadlineError::Interrupted => Self::Cancelled,
            _ => Self::Unknown(Box::new(value)),
        }
    }
}
