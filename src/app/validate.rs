use rustyline::validate::{ValidationContext, ValidationResult, Validator};

use crate::command::InternalCommand;

use super::{App, Command};

pub struct YspValidator {
    pub enabled: bool,
}

impl Validator for YspValidator {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();

        // validate sql mainly
        if self.enabled {
            if let Ok(Some(command)) = App::parse_command(input) {
                match command {
                    Command::SQL(sql) => return self.validate_sql(&sql),
                    Command::Internal(InternalCommand::Describe(table_or_view)) => {
                        if table_or_view.ends_with(';') {
                            return Ok(ValidationResult::Valid(None));
                        } else {
                            return Ok(ValidationResult::Incomplete);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(ValidationResult::Valid(None))
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}

impl YspValidator {
    fn validate_sql(&self, sql: &str) -> rustyline::Result<ValidationResult> {
        let (single_line, last_line_is_comment, last_line) = sql
            .lines()
            .enumerate()
            .last()
            .map_or((true, false, ""), |(y, last_line)| {
                (y == 0, last_line.starts_with("--"), last_line)
            });
        if (single_line && (last_line_is_comment || sql.ends_with(';')))
            || (!single_line && last_line == "/")
        {
            Ok(ValidationResult::Valid(None))
        } else {
            Ok(ValidationResult::Incomplete)
        }
    }
}

impl YspValidator {
    pub fn new() -> Self {
        YspValidator { enabled: true }
    }
}
