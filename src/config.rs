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
            let value = value.trim().trim_matches('"').trim_matches('\'');
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
            comment_char = ;
            nonsense = 42
            "#,
        );
        assert_eq!(config.tab_width, 8);
        assert!(!config.line_numbers);
        assert_eq!(config.theme, ThemeName::Light);
        assert!(config.auto_indent);
        assert_eq!(config.comment_char, ';');
    }

    #[test]
    fn clamps_tab_width() {
        let mut config = Config::default();
        config.parse("tab_width = 999");
        assert_eq!(config.tab_width, 16);
    }
}
