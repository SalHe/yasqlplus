use colorize::AnsiColor;
use rustyline::highlight::Highlighter;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

pub struct YspHightligter {}

impl YspHightligter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Highlighter for YspHightligter {
    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        std::borrow::Cow::Owned(hint.to_owned().bold())
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        // TODO optimize

        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();

        let syntax = ps.find_syntax_by_extension("sql").unwrap();
        let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
        let mut escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        escaped.push_str("\x1b[0m");
        std::borrow::Cow::Owned(escaped)
    }
}
