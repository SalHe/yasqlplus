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
}
