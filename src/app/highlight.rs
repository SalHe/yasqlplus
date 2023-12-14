use colored::Colorize;
use rustyline::highlight::Highlighter;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

pub struct YspHightligter {
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl YspHightligter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();

        let theme = ts.themes["base16-ocean.dark"].clone();
        Self { theme, syntax_set }
    }
}

impl Highlighter for YspHightligter {
    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        std::borrow::Cow::Owned(hint.to_owned().bold().to_string())
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        let syntax = self.syntax_set.find_syntax_by_extension("sql").unwrap();
        let mut h = HighlightLines::new(&syntax, &self.theme);
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &self.syntax_set).unwrap();
        let mut escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        escaped.push_str("\x1b[0m");
        std::borrow::Cow::Owned(escaped)
    }
}
