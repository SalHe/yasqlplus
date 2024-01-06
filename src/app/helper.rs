use std::sync::{Arc, RwLock};

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
    pub fn new(context: Arc<RwLock<Context>>) -> Self {
        YspHelper {
            validator: YspValidator,
            hinter: HistoryHinter::new(),
            hightligter: YspHightligter::new(),
            completer: YspCompleter::new(context),
        }
    }
}
