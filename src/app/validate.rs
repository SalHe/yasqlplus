use rustyline::validate::{ValidationContext, ValidationResult, Validator};

use crate::command::parse_command;

pub struct YspValidator {
    pub enabled: bool,
}

impl Validator for YspValidator {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();

        // validate sql mainly
        if self.enabled {
            return match parse_command(input) {
                Err(crate::command::ParseError::Incomplete(_)) => Ok(ValidationResult::Incomplete),
                _ => Ok(ValidationResult::Valid(None)),
            };
        }
        Ok(ValidationResult::Valid(None))
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}

impl YspValidator {
    pub fn new() -> Self {
        YspValidator { enabled: true }
    }
}
