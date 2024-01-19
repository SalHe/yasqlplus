use colored::Colorize;
use reedline::StyledText;
use rustyline::highlight::Highlighter;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style, Theme, ThemeSet};
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
        let mut h = HighlightLines::new(syntax, &self.theme);
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &self.syntax_set).unwrap();
        let mut escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        escaped.push_str("\x1b[0m");
        std::borrow::Cow::Owned(escaped)
    }
}

impl reedline::Highlighter for YspHightligter {
    fn highlight(&self, line: &str, _cursor: usize) -> reedline::StyledText {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let theme = ts.themes["base16-ocean.dark"].clone();

        let syntax: &syntect::parsing::SyntaxReference =
            syntax_set.find_syntax_by_extension("sql").unwrap();
        let mut h = HighlightLines::new(syntax, &theme);
        let ranges = h.highlight_line(line, &syntax_set).unwrap();
        StyledText {
            buffer: ranges
                .iter()
                .map(|(st, str)| {
                    let mut style = nu_ansi_term::Style::new()
                        .fg(hl_color_to_nu_color(st.foreground))
                        .on(hl_color_to_nu_color(st.background));
                    if st.font_style.contains(FontStyle::BOLD) {
                        style = style.bold();
                    }
                    if st.font_style.contains(FontStyle::ITALIC) {
                        style = style.italic();
                    }
                    if st.font_style.contains(FontStyle::UNDERLINE) {
                        style = style.underline();
                    }
                    (style, str.to_string())
                })
                .collect::<Vec<_>>(),
        }
    }
}

fn hl_color_to_nu_color(color: syntect::highlighting::Color) -> nu_ansi_term::Color {
    nu_ansi_term::Color::Rgb(color.r, color.g, color.b)
}
