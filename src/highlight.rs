use ratatui::style::{Color, Modifier, Style};
use syntect::highlighting::{
    FontStyle, Style as SyntectStyle, Theme, ThemeSet,
};
use syntect::parsing::SyntaxSet;

/// A segment of text with an associated ratatui style.
#[derive(Clone, Debug)]
pub struct StyledSegment {
    pub text: String,
    pub style: Style,
}

/// Wrapper around syntect for syntax highlighting.
pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl Highlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set.themes["base16-ocean.dark"].clone();
        Self { syntax_set, theme }
    }

    /// Highlight the given source code, returning styled segments per line.
    /// `extension` is used to determine the syntax (e.g. "rs", "py").
    pub fn highlight(&self, source: &str, extension: &str) -> Vec<Vec<StyledSegment>> {
        use syntect::easy::HighlightLines;

        let syntax = self
            .syntax_set
            .find_syntax_by_extension(extension)
            .or_else(|| self.syntax_set.find_syntax_by_extension("txt"))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut h = HighlightLines::new(syntax, &self.theme);
        let mut result = Vec::new();

        for line in source.lines() {
            let ranges = h
                .highlight_line(line, &self.syntax_set)
                .unwrap_or_default();
            let segments: Vec<StyledSegment> = ranges
                .into_iter()
                .map(|(style, text)| StyledSegment {
                    text: text.to_string(),
                    style: convert_style(style),
                })
                .collect();
            result.push(segments);
        }

        result
    }
}

/// Convert a syntect style to a ratatui style.
fn convert_style(syntect_style: SyntectStyle) -> Style {
    let fg = Color::Rgb(
        syntect_style.foreground.r,
        syntect_style.foreground.g,
        syntect_style.foreground.b,
    );

    let mut style = Style::default().fg(fg);

    if syntect_style.font_style.contains(FontStyle::BOLD) {
        style = style.add_modifier(Modifier::BOLD);
    }
    if syntect_style.font_style.contains(FontStyle::ITALIC) {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if syntect_style.font_style.contains(FontStyle::UNDERLINE) {
        style = style.add_modifier(Modifier::UNDERLINED);
    }

    style
}
