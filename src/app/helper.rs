use rustyline::{hint::HistoryHinter, Completer, Helper, Highlighter, Hinter, Validator};

use super::{highlight::YspHightligter, validate::YspValidator};

#[derive(Completer, Helper, Highlighter, Hinter, Validator)]
pub struct YspHelper {
    #[rustyline(Validator)]
    validator: YspValidator,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
    #[rustyline(Highlighter)]
    hightligter: YspHightligter,
}

impl YspHelper {
    pub fn new() -> Self {
        YspHelper {
            validator: YspValidator::new(),
            hinter: HistoryHinter::new(),
            hightligter: YspHightligter::new(),
        }
    }

    pub fn disable_validation(&mut self) {
        self.validator.enabled = false;
    }

    pub fn enable_validation(&mut self) {
        self.validator.enabled = true;
    }
}
