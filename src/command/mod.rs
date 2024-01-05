mod conn;
mod parser;

pub use conn::*;
pub use parser::*;

#[allow(clippy::upper_case_acronyms)]
pub enum Command {
    /// To execute SQL.
    SQL(String),

    /// To execute a shell command like in bash.
    Shell(String),

    /// To execute yasqlplus command such as `conn`, `desc`.
    Internal(InternalCommand),
}

pub enum InternalCommand {
    Describe(String),
    Connect(Connection),
    Exit,
}

impl Command {
    pub fn need_connection(&self) -> bool {
        match self {
            Command::SQL(_) => true,
            Command::Shell(_) => false,
            Command::Internal(internal) => match internal {
                InternalCommand::Describe(_) => true,
                InternalCommand::Connect(_) => false,
                InternalCommand::Exit => false,
            },
        }
    }
}
