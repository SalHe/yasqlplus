use rustyline::{hint::HistoryHinter, Completer, Helper, Highlighter, Hinter, Validator};

use super::validate::YspValidator;

#[derive(Completer, Helper, Highlighter, Hinter, Validator)]
pub struct YspHelper {
    #[rustyline(Validator)]
    validator: YspValidator,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
}

impl YspHelper {
    pub fn new() -> Self {
        YspHelper {
            validator: YspValidator::new(),
            hinter: HistoryHinter::new(),
        }
    }
}
