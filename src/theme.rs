//! Colour schemes for the syntax highlighter and the UI chrome.

use crate::syntax::{Tok, TOK_COUNT};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub fg: Option<u8>,
    pub bg: Option<u8>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Style {
    pub const fn plain() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }

    pub const fn fg(color: u8) -> Self {
        Self {
            fg: Some(color),
            ..Self::plain()
        }
    }

    pub const fn bg(color: u8) -> Self {
        Self {
            bg: Some(color),
            ..Self::plain()
        }
    }

    pub const fn bold(self) -> Self {
        Self { bold: true, ..self }
    }

    pub const fn italic(self) -> Self {
        Self {
            italic: true,
            ..self
        }
    }

    pub const fn underline(self) -> Self {
        Self {
            underline: true,
            ..self
        }
    }

    pub const fn with_bg(self, color: u8) -> Self {
        Self {
            bg: Some(color),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeName {
    Dark,
    Light,
}

impl ThemeName {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "dark" | "default" => Some(ThemeName::Dark),
            "light" => Some(ThemeName::Light),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct Theme {
    pub syntax: [Style; TOK_COUNT],
    pub text: Style,
    pub gutter: Style,
    pub gutter_current: Style,
    pub modeline: Style,
    pub modeline_accent: Style,
    pub minibuffer: Style,
    pub prompt: Style,
    pub region: Style,
    pub search: Style,
    pub trailing_ws: Style,
    pub help_heading: Style,
    pub help_key: Style,
    pub help_dim: Style,
    pub fill: Style,
    pub completion: Style,
    pub completion_selected: Style,
    pub spell: Style,
}

impl Theme {
    pub fn by_name(name: ThemeName) -> Theme {
        match name {
            ThemeName::Dark => Theme::dark(),
            ThemeName::Light => Theme::light(),
        }
    }

    pub fn tok(&self, tok: Tok) -> Style {
        self.syntax[tok as usize]
    }

    pub fn dark() -> Theme {
        let syntax = [
            Style::fg(81).bold(),       // Keyword
            Style::fg(114),             // Type
            Style::fg(179),             // Constant
            Style::fg(222),             // Function
            Style::fg(150),             // String
            Style::fg(244),             // Comment
            Style::fg(180),             // Number
            Style::fg(210),             // Operator
            Style::fg(176),             // Preproc
            Style::fg(117).bold(),      // Heading
            Style::fg(223).italic(),    // Emphasis
            Style::fg(75).underline(),  // Link
            Style::fg(120).with_bg(22), // DiffAdd
            Style::fg(210).with_bg(52), // DiffDel
            Style::fg(81).with_bg(236), // DiffHunk
            Style::fg(246),             // Meta
        ];
        Theme {
            syntax,
            text: Style::plain(),
            gutter: Style::fg(240),
            gutter_current: Style::fg(252).bold(),
            modeline: Style::fg(252).with_bg(236),
            modeline_accent: Style::fg(81).with_bg(236).bold(),
            minibuffer: Style::fg(252).with_bg(234),
            prompt: Style::fg(81).with_bg(234).bold(),
            region: Style::plain().with_bg(237),
            search: Style::fg(231).with_bg(58),
            trailing_ws: Style::plain().with_bg(52),
            help_heading: Style::fg(81).bold(),
            help_key: Style::fg(222),
            help_dim: Style::fg(244),
            fill: Style::fg(238),
            completion: Style::fg(250).with_bg(236),
            completion_selected: Style::fg(231).with_bg(60).bold(),
            spell: Style::fg(203).underline(),
        }
    }

    pub fn light() -> Theme {
        let syntax = [
            Style::fg(25).bold(),        // Keyword
            Style::fg(28),               // Type
            Style::fg(130),              // Constant
            Style::fg(160),              // Function
            Style::fg(28),               // String
            Style::fg(244),              // Comment
            Style::fg(94),               // Number
            Style::fg(90),               // Operator
            Style::fg(55),               // Preproc
            Style::fg(25).bold(),        // Heading
            Style::fg(90).italic(),      // Emphasis
            Style::fg(26).underline(),   // Link
            Style::fg(22).with_bg(194),  // DiffAdd
            Style::fg(124).with_bg(224), // DiffDel
            Style::fg(25).with_bg(254),  // DiffHunk
            Style::fg(240),              // Meta
        ];
        Theme {
            syntax,
            text: Style::plain(),
            gutter: Style::fg(250),
            gutter_current: Style::fg(236).bold(),
            modeline: Style::fg(235).with_bg(254),
            modeline_accent: Style::fg(25).with_bg(254).bold(),
            minibuffer: Style::fg(235).with_bg(255),
            prompt: Style::fg(25).with_bg(255).bold(),
            region: Style::plain().with_bg(254),
            search: Style::fg(16).with_bg(226),
            trailing_ws: Style::plain().with_bg(217),
            help_heading: Style::fg(25).bold(),
            help_key: Style::fg(160),
            help_dim: Style::fg(244),
            fill: Style::fg(250),
            completion: Style::fg(235).with_bg(253),
            completion_selected: Style::fg(16).with_bg(153).bold(),
            spell: Style::fg(160).underline(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn themes_have_a_style_for_every_token() {
        let all = [
            Tok::Keyword,
            Tok::Type,
            Tok::Constant,
            Tok::Function,
            Tok::String,
            Tok::Comment,
            Tok::Number,
            Tok::Operator,
            Tok::Preproc,
            Tok::Heading,
            Tok::Emphasis,
            Tok::Link,
            Tok::DiffAdd,
            Tok::DiffDel,
            Tok::DiffHunk,
            Tok::Meta,
        ];
        assert_eq!(all.len(), TOK_COUNT);
        for theme in [Theme::dark(), Theme::light()] {
            for tok in all {
                assert!(theme.tok(tok).fg.is_some() || theme.tok(tok).bg.is_some());
            }
        }
    }

    #[test]
    fn theme_name_parsing() {
        assert_eq!(ThemeName::parse("Dark"), Some(ThemeName::Dark));
        assert_eq!(ThemeName::parse("light"), Some(ThemeName::Light));
        assert_eq!(ThemeName::parse("nope"), None);
    }
}
