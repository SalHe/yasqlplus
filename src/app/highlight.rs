use colorize::AnsiColor;
use rustyline::highlight::Highlighter;

pub struct YspHightligter {}

impl YspHightligter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Highlighter for YspHightligter {
    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        std::borrow::Cow::Owned(hint.to_owned().italic())
    }
}
