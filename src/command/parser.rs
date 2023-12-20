use thiserror::Error;

use super::{parse_connection_string, Command, ConnParsingError, InternalCommand};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    // Empty command. Do nothing.
    #[error("No command.")]
    Empty,

    // Incomplete command.
    #[error("Incomple command: {0}")]
    Incomplete(String),

    // Parsing error.
    #[error("Failed to parse command: {command}. \nError{err}")]
    ParsingError {
        err: ConnParsingError,
        command: String,
    },
}

pub fn parse_internal_command(command: &str) -> Option<Result<Command, ParseError>> {
    if let Some(table_or_view) = command.strip_prefix("desc ") {
        Some(if table_or_view.ends_with(';') {
            Ok(Command::Internal(InternalCommand::Describe(
                table_or_view.trim_end_matches(';').to_string(), // Consistent with yasql
            )))
        } else {
            Err(ParseError::Incomplete(command.to_string()))
        })
    } else {
        command.strip_prefix("conn ").map(|conn_str| {
            parse_connection_string(conn_str)
                .map(|x| Command::Internal(InternalCommand::Connect(x)))
                .map_err(|err| ParseError::ParsingError {
                    err,
                    command: command.to_string(),
                })
        })
    }
}

pub fn parse_command(command: &str) -> Result<Command, ParseError> {
    if command.is_empty() {
        Err(ParseError::Empty)
    } else if let Some(shell) = command.strip_prefix('!') {
        Ok(Command::Shell(shell.to_string()))
    } else if let Some(table_or_view) = parse_internal_command(command) {
        table_or_view
    } else {
        // fallback everything else to sql (including comment)
        let (single_line, last_line_is_comment, last_line) = command
            .lines()
            .enumerate()
            .last()
            .map_or((true, false, ""), |(y, last_line)| {
                (y == 0, last_line.starts_with("--"), last_line)
            });
        if (single_line && (last_line_is_comment || command.ends_with(';')))
            || (!single_line && last_line == "/")
        {
            Ok(Command::SQL(command.to_string()))
        } else {
            Err(ParseError::Incomplete(command.to_string()))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::command::{parser::ParseError, Command, InternalCommand};

    use super::parse_command;

    #[test]
    fn parse() {
        // empty command
        assert!(matches!(parse_command(""), Err(ParseError::Empty)));

        // incomplete
        assert!(matches!(
            parse_command("desc table"),
            Err(ParseError::Incomplete(_))
        )); // missing `;`
        assert!(matches!(
            parse_command("select * from dba_views"),
            Err(ParseError::Incomplete(_))
        )); // missing `;`
        assert!(matches!(
            parse_command(
                r"select * from dba_views
;



"
            ),
            Err(ParseError::Incomplete(_))
        )); // missing `/`

        // shell
        assert!(matches!(
            parse_command("!vim ~/.bashrc"),
            Ok(Command::Shell(shell)) if shell == "vim ~/.bashrc"
        ));

        // desc
        assert!(matches!(
            parse_command("desc table;"),
            Ok(Command::Internal(InternalCommand::Describe(t))) if t == "table"
        ));

        // conn
        assert!(matches!(
            parse_command("conn sys/pwd@host:9999"),
            Ok(Command::Internal(InternalCommand::Connect(_)))
        ));
        assert!(matches!(
            parse_command("conn sys/pwd@host:9999;"),
            Err(ParseError::ParsingError { .. })
        ));
    }
}
