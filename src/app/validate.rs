use rustyline::validate::Validator;

pub struct YspValidator {}

impl Validator for YspValidator {}

impl YspValidator {
    pub fn new() -> Self {
        YspValidator {}
    }
}
