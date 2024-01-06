use rustyline::validate::{ValidationContext, ValidationResult};

use crate::command::parse_command;

pub struct YspValidator;

impl rustyline::validate::Validator for YspValidator {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();

        // validate sql mainly
        match parse_command(input) {
            Err(crate::command::ParseError::Incomplete(_)) => Ok(ValidationResult::Incomplete),
            _ => Ok(ValidationResult::Valid(None)),
        }
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}

impl reedline::Validator for YspValidator {
    fn validate(&self, line: &str) -> reedline::ValidationResult {
        let input = line;
        match parse_command(input) {
            Err(crate::command::ParseError::Incomplete(_)) => {
                reedline::ValidationResult::Incomplete
            }
            _ => reedline::ValidationResult::Complete,
        }
    }
}
