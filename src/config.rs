//! Optional user configuration.
//!
//! A simple `key = value` file is read from
//! `$NANO_EMACS_CONFIG`, `$XDG_CONFIG_HOME/nano-emacs/config`, or
//! `~/.config/nano-emacs/config`. Unknown keys are ignored, so the file can
//! grow without breaking older builds.

use std::env;
use std::fs;
use std::path::PathBuf;

use crate::theme::ThemeName;

#[derive(Clone, Debug)]
pub struct Config {
    pub tab_width: usize,
    pub line_numbers: bool,
    pub highlight_trailing_ws: bool,
    pub auto_indent: bool,
    /// Break typed text at `fill_column` (Emacs `auto-fill-mode`).
    pub auto_fill: bool,
    /// Column at which auto-fill breaks lines.
    pub fill_column: usize,
    /// Truncate long lines and scroll horizontally instead of wrapping them.
    pub truncate_lines: bool,
    /// Show word-completion candidates while typing in the editor.
    pub editor_completion: bool,
    /// Highlight misspelled words in prose modes.
    pub spell_check: bool,
    /// Optional path to a word list or hunspell `.dic` file.
    pub spell_dictionary: Option<String>,
    pub theme: ThemeName,
    /// Comment character used by git's commit message template.
    pub comment_char: char,
    pub confirm_on_quit: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tab_width: 4,
            line_numbers: true,
            highlight_trailing_ws: true,
            auto_indent: true,
            auto_fill: false,
            fill_column: 70,
            truncate_lines: true,
            editor_completion: false,
            spell_check: false,
            spell_dictionary: None,
            theme: ThemeName::Dark,
            comment_char: '#',
            confirm_on_quit: true,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let mut config = Config::default();
        if let Some(path) = config_path() {
            if let Ok(text) = fs::read_to_string(path) {
                config.parse(&text);
            }
        }
        config
    }

    pub fn parse(&mut self, text: &str) {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim().to_ascii_lowercase();
            let value = strip_comment(value).trim_matches('"').trim_matches('\'');
            match key.as_str() {
                "tab_width" | "tab-width" => {
                    if let Ok(n) = value.parse::<usize>() {
                        self.tab_width = n.clamp(1, 16);
                    }
                }
                "line_numbers" | "line-numbers" => {
                    if let Some(b) = parse_bool(value) {
                        self.line_numbers = b;
                    }
                }
                "highlight_trailing_whitespace" | "highlight-trailing-whitespace" => {
                    if let Some(b) = parse_bool(value) {
                        self.highlight_trailing_ws = b;
                    }
                }
                "auto_indent" | "auto-indent" => {
                    if let Some(b) = parse_bool(value) {
                        self.auto_indent = b;
                    }
                }
                "auto_fill" | "auto-fill" | "auto_fill_mode" | "auto-fill-mode" => {
                    if let Some(b) = parse_bool(value) {
                        self.auto_fill = b;
                    }
                }
                "fill_column" | "fill-column" => {
                    if let Ok(n) = value.parse::<usize>() {
                        self.fill_column = n.clamp(1, 1000);
                    }
                }
                "truncate_lines" | "truncate-lines" | "truncate" => {
                    if let Some(b) = parse_bool(value) {
                        self.truncate_lines = b;
                    }
                }
                "editor_completion" | "editor-completion" | "completion_in_editor" => {
                    if let Some(b) = parse_bool(value) {
                        self.editor_completion = b;
                    }
                }
                "spell_check" | "spell-check" | "spellcheck" => {
                    if let Some(b) = parse_bool(value) {
                        self.spell_check = b;
                    }
                }
                "spell_dictionary" | "spell-dictionary" => {
                    if value.is_empty() {
                        self.spell_dictionary = None;
                    } else {
                        self.spell_dictionary = Some(value.to_string());
                    }
                }
                "confirm_on_quit" | "confirm-on-quit" => {
                    if let Some(b) = parse_bool(value) {
                        self.confirm_on_quit = b;
                    }
                }
                "theme" => {
                    if let Some(theme) = ThemeName::parse(value) {
                        self.theme = theme;
                    }
                }
                "comment_char" | "comment-char" => {
                    if let Some(c) = value.chars().next() {
                        self.comment_char = c;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn config_path() -> Option<PathBuf> {
        config_path()
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// Remove a `#` comment from a value, but keep `#` inside quotes (so
/// `comment_char = "#"` still works).
fn strip_comment(value: &str) -> &str {
    let mut quote = None;
    let mut previous = ' ';
    for (index, c) in value.char_indices() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None => {
                if matches!(c, '"' | '\'') {
                    quote = Some(c);
                } else if c == '#' && previous.is_whitespace() {
                    return value[..index].trim();
                }
            }
        }
        previous = c;
    }
    value.trim()
}

fn config_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("NANO_EMACS_CONFIG") {
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    if let Ok(dir) = env::var("XDG_CONFIG_HOME") {
        if !dir.is_empty() {
            return Some(PathBuf::from(dir).join("nano-emacs").join("config"));
        }
    }
    let home = env::var("HOME").ok()?;
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("nano-emacs")
            .join("config"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_keys_and_ignores_junk() {
        let mut config = Config::default();
        config.parse(
            r#"
            # comment
            tab_width = 8
            line_numbers = no
            theme = light
            auto-indent = true
            auto_fill = yes
            fill_column = 42
            truncate_lines = false
            editor_completion = true
            spell_check = yes
            spell_dictionary = /tmp/words
            comment_char = ;
            nonsense = 42
            "#,
        );
        assert_eq!(config.tab_width, 8);
        assert!(!config.line_numbers);
        assert_eq!(config.theme, ThemeName::Light);
        assert!(config.auto_indent);
        assert!(config.auto_fill);
        assert_eq!(config.fill_column, 42);
        assert!(!config.truncate_lines);
        assert!(config.editor_completion);
        assert!(config.spell_check);
        assert_eq!(config.spell_dictionary.as_deref(), Some("/tmp/words"));
        assert_eq!(config.comment_char, ';');
    }

    #[test]
    fn clamps_tab_width() {
        let mut config = Config::default();
        config.parse("tab_width = 999");
        assert_eq!(config.tab_width, 16);
    }

    #[test]
    fn strips_inline_comments_but_keeps_quoted_hashes() {
        let mut config = Config::default();
        config.parse(
            "theme = light # comment\ncomment_char = \"#\" # hash\ntruncate_lines = false # wrap\n",
        );
        assert_eq!(config.theme, ThemeName::Light);
        assert_eq!(config.comment_char, '#');
        assert!(!config.truncate_lines);
    }
}
