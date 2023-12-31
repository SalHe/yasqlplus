use std::{rc::Rc, sync::RwLock};

use rustyline::{hint::HistoryHinter, Completer, Helper, Highlighter, Hinter, Validator};

use super::{
    completer::YspCompleter, context::Context, highlight::YspHightligter, validate::YspValidator,
};

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
    pub fn new(context: Rc<RwLock<Context>>) -> Self {
        YspHelper {
            validator: YspValidator::new(),
            hinter: HistoryHinter::new(),
            hightligter: YspHightligter::new(),
            completer: YspCompleter::new(context),
        }
    }

    pub fn disable_validation(&mut self) {
        self.validator.enabled = false;
    }

    pub fn enable_validation(&mut self) {
        self.validator.enabled = true;
    }
}
