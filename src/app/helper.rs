use std::{cell::RefCell, rc::Rc};

use rustyline::{hint::HistoryHinter, Completer, Helper, Highlighter, Hinter, Validator};
use yasqlplus::wrapper::Connection;

use super::{completer::YspCompleter, highlight::YspHightligter, validate::YspValidator};

#[derive(Completer, Helper, Highlighter, Hinter, Validator)]
pub struct YspHelper {
    #[rustyline(Validator)]
    validator: YspValidator,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
    #[rustyline(Highlighter)]
    hightligter: YspHightligter,
    #[rustyline(Completer)]
    completer: YspCompleter,
}

impl YspHelper {
    pub fn new(connection: Rc<RefCell<Option<Connection>>>) -> Self {
        YspHelper {
            validator: YspValidator::new(),
            hinter: HistoryHinter::new(),
            hightligter: YspHightligter::new(),
            completer: YspCompleter::new(connection),
        }
    }

    pub fn disable_validation(&mut self) {
        self.validator.enabled = false;
    }

    pub fn enable_validation(&mut self) {
        self.validator.enabled = true;
    }
}
