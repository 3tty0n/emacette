//! Optional user configuration.
//!
//! A simple `key = value` file is read from
//! `$EMACETTE_CONFIG`, `$XDG_CONFIG_HOME/emacette/config`, or
//! `~/.config/emacette/config`. The legacy `NANO_EMACS_CONFIG` and
//! `~/.config/nano-emacs/config` are read as fallbacks. Unknown keys are
//! ignored, so the file can grow without breaking older builds.
//!
//! Values outside a section are the defaults for every language. A section
//! such as `[rust]` overrides only that language.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use crate::syntax::Lang;
use crate::theme::ThemeName;

/// Per-language overrides. `None` means "inherit the global value".
#[derive(Clone, Debug, Default)]
pub struct LanguageConfig {
    pub tab_width: Option<usize>,
    pub line_numbers: Option<bool>,
    pub highlight_trailing_ws: Option<bool>,
    pub auto_indent: Option<bool>,
    pub auto_fill: Option<bool>,
    pub fill_column: Option<usize>,
    pub truncate_lines: Option<bool>,
    pub editor_completion: Option<bool>,
    pub spell_check: Option<bool>,
    pub spell_dictionary: Option<String>,
    pub comment_start: Option<String>,
    pub diagnostics_inline: Option<bool>,
    pub diagnostics_popup: Option<bool>,
    pub lsp_command: Option<String>,
    pub lsp_language_id: Option<String>,
}

/// Fully resolved settings for the current language.
#[derive(Clone, Debug)]
pub struct Settings {
    pub tab_width: usize,
    pub line_numbers: bool,
    pub highlight_trailing_ws: bool,
    pub auto_indent: bool,
    pub auto_fill: bool,
    pub fill_column: usize,
    pub truncate_lines: bool,
    pub editor_completion: bool,
    pub spell_check: bool,
    pub spell_dictionary: Option<String>,
    pub comment_start: String,
    pub diagnostics_inline: bool,
    pub diagnostics_popup: bool,
    pub lsp_command: Option<String>,
    pub lsp_language_id: Option<String>,
}

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
    /// Global line-comment prefix. When unset, use the language default.
    pub comment_start: Option<String>,
    /// Highlight LSP diagnostics inline.
    pub diagnostics_inline: bool,
    /// Show LSP diagnostics for the current line in a popup.
    pub diagnostics_popup: bool,
    /// Default LSP command line; sections can override or disable it.
    pub lsp_command: Option<String>,
    /// Default LSP `languageId`; sections can override it.
    pub lsp_language_id: Option<String>,
    pub languages: HashMap<String, LanguageConfig>,
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
            comment_start: None,
            diagnostics_inline: true,
            diagnostics_popup: true,
            lsp_command: None,
            lsp_language_id: None,
            languages: HashMap::new(),
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
        let mut section: Option<String> = None;
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') {
                if let Some(end) = line.find(']') {
                    let name = normalize_section(&line[1..end]);
                    section = if name.is_empty() { None } else { Some(name) };
                    continue;
                }
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim().to_ascii_lowercase();
            let value = unquote(strip_comment(value));
            match section.as_deref() {
                Some(name) => {
                    let language = self.languages.entry(name.to_string()).or_default();
                    apply_language_key(language, &key, value);
                }
                None => apply_global_key(self, &key, value),
            }
        }
    }

    pub fn config_path() -> Option<PathBuf> {
        config_path()
    }

    /// Merge global defaults with the section for `lang`.
    pub fn settings_for(&self, lang: Lang) -> Settings {
        let overrides = self.language_config(lang);
        let pick_bool = |get: fn(&LanguageConfig) -> Option<bool>, default: bool| {
            overrides.and_then(get).unwrap_or(default)
        };
        let pick_usize = |get: fn(&LanguageConfig) -> Option<usize>, default: usize| {
            overrides.and_then(get).unwrap_or(default)
        };
        let comment_start = overrides
            .and_then(|config| config.comment_start.clone())
            .or_else(|| self.comment_start.clone())
            .unwrap_or_else(|| match lang {
                Lang::GitCommit => self.comment_char.to_string(),
                _ => lang.default_comment_start().to_string(),
            });
        Settings {
            tab_width: pick_usize(|config| config.tab_width, self.tab_width),
            line_numbers: pick_bool(|config| config.line_numbers, self.line_numbers),
            highlight_trailing_ws: pick_bool(
                |config| config.highlight_trailing_ws,
                self.highlight_trailing_ws,
            ),
            auto_indent: pick_bool(|config| config.auto_indent, self.auto_indent),
            auto_fill: pick_bool(|config| config.auto_fill, self.auto_fill),
            fill_column: pick_usize(|config| config.fill_column, self.fill_column),
            truncate_lines: pick_bool(|config| config.truncate_lines, self.truncate_lines),
            editor_completion: pick_bool(|config| config.editor_completion, self.editor_completion),
            spell_check: pick_bool(|config| config.spell_check, self.spell_check),
            spell_dictionary: overrides
                .and_then(|config| config.spell_dictionary.clone())
                .or_else(|| self.spell_dictionary.clone()),
            comment_start,
            diagnostics_inline: pick_bool(
                |config| config.diagnostics_inline,
                self.diagnostics_inline,
            ),
            diagnostics_popup: pick_bool(|config| config.diagnostics_popup, self.diagnostics_popup),
            lsp_command: overrides
                .and_then(|config| config.lsp_command.clone())
                .or_else(|| self.lsp_command.clone()),
            lsp_language_id: overrides
                .and_then(|config| config.lsp_language_id.clone())
                .or_else(|| self.lsp_language_id.clone()),
        }
    }

    fn language_config(&self, lang: Lang) -> Option<&LanguageConfig> {
        let keys = [lang.config_key(), lang.name()];
        keys.iter()
            .map(|key| normalize_section(key))
            .chain(
                lang.config_aliases()
                    .iter()
                    .map(|key| normalize_section(key)),
            )
            .find_map(|key| self.languages.get(&key))
    }
}

fn apply_global_key(config: &mut Config, key: &str, value: &str) {
    match key {
        "tab_width" | "tab-width" => {
            if let Ok(n) = value.parse::<usize>() {
                config.tab_width = n.clamp(1, 16);
            }
        }
        "line_numbers" | "line-numbers" => {
            if let Some(b) = parse_bool(value) {
                config.line_numbers = b;
            }
        }
        "highlight_trailing_whitespace" | "highlight-trailing-whitespace" => {
            if let Some(b) = parse_bool(value) {
                config.highlight_trailing_ws = b;
            }
        }
        "auto_indent" | "auto-indent" => {
            if let Some(b) = parse_bool(value) {
                config.auto_indent = b;
            }
        }
        "auto_fill" | "auto-fill" | "auto_fill_mode" | "auto-fill-mode" => {
            if let Some(b) = parse_bool(value) {
                config.auto_fill = b;
            }
        }
        "fill_column" | "fill-column" => {
            if let Ok(n) = value.parse::<usize>() {
                config.fill_column = n.clamp(1, 1000);
            }
        }
        "truncate_lines" | "truncate-lines" | "truncate" => {
            if let Some(b) = parse_bool(value) {
                config.truncate_lines = b;
            }
        }
        "editor_completion" | "editor-completion" | "completion_in_editor" => {
            if let Some(b) = parse_bool(value) {
                config.editor_completion = b;
            }
        }
        "spell_check" | "spell-check" | "spellcheck" => {
            if let Some(b) = parse_bool(value) {
                config.spell_check = b;
            }
        }
        "spell_dictionary" | "spell-dictionary" => {
            config.spell_dictionary = nonempty(value);
        }
        "comment_start" | "comment-start" => {
            config.comment_start = Some(value.to_string());
        }
        "comment_char" | "comment-char" => {
            if let Some(c) = value.chars().next() {
                config.comment_char = c;
            }
        }
        "diagnostics_inline" | "diagnostics-inline" => {
            if let Some(b) = parse_bool(value) {
                config.diagnostics_inline = b;
            }
        }
        "diagnostics_popup" | "diagnostics-popup" => {
            if let Some(b) = parse_bool(value) {
                config.diagnostics_popup = b;
            }
        }
        "lsp_command" | "lsp-command" => {
            config.lsp_command = command(value);
        }
        "lsp_language_id" | "lsp-language-id" => {
            config.lsp_language_id = nonempty(value);
        }
        "confirm_on_quit" | "confirm-on-quit" => {
            if let Some(b) = parse_bool(value) {
                config.confirm_on_quit = b;
            }
        }
        "theme" => {
            if let Some(theme) = ThemeName::parse(value) {
                config.theme = theme;
            }
        }
        _ => {}
    }
}

fn apply_language_key(config: &mut LanguageConfig, key: &str, value: &str) {
    match key {
        "tab_width" | "tab-width" => {
            if let Ok(n) = value.parse::<usize>() {
                config.tab_width = Some(n.clamp(1, 16));
            }
        }
        "line_numbers" | "line-numbers" => {
            if let Some(b) = parse_bool(value) {
                config.line_numbers = Some(b);
            }
        }
        "highlight_trailing_whitespace" | "highlight-trailing-whitespace" => {
            if let Some(b) = parse_bool(value) {
                config.highlight_trailing_ws = Some(b);
            }
        }
        "auto_indent" | "auto-indent" => {
            if let Some(b) = parse_bool(value) {
                config.auto_indent = Some(b);
            }
        }
        "auto_fill" | "auto-fill" | "auto_fill_mode" | "auto-fill-mode" => {
            if let Some(b) = parse_bool(value) {
                config.auto_fill = Some(b);
            }
        }
        "fill_column" | "fill-column" => {
            if let Ok(n) = value.parse::<usize>() {
                config.fill_column = Some(n.clamp(1, 1000));
            }
        }
        "truncate_lines" | "truncate-lines" | "truncate" => {
            if let Some(b) = parse_bool(value) {
                config.truncate_lines = Some(b);
            }
        }
        "editor_completion" | "editor-completion" | "completion_in_editor" => {
            if let Some(b) = parse_bool(value) {
                config.editor_completion = Some(b);
            }
        }
        "spell_check" | "spell-check" | "spellcheck" => {
            if let Some(b) = parse_bool(value) {
                config.spell_check = Some(b);
            }
        }
        "spell_dictionary" | "spell-dictionary" => {
            config.spell_dictionary = Some(value.to_string());
        }
        "comment_start" | "comment-start" | "comment_char" | "comment-char" => {
            config.comment_start = Some(value.to_string());
        }
        "diagnostics_inline" | "diagnostics-inline" => {
            if let Some(b) = parse_bool(value) {
                config.diagnostics_inline = Some(b);
            }
        }
        "diagnostics_popup" | "diagnostics-popup" => {
            if let Some(b) = parse_bool(value) {
                config.diagnostics_popup = Some(b);
            }
        }
        "lsp_command" | "lsp-command" => {
            config.lsp_command = Some(command(value).unwrap_or_default());
        }
        "lsp_language_id" | "lsp-language-id" => {
            config.lsp_language_id = Some(value.to_string());
        }
        _ => {}
    }
}

fn unquote(value: &str) -> &str {
    let value = value.trim();
    if let Some(inner) = value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        return inner;
    }
    if let Some(inner) = value
        .strip_prefix('\'')
        .and_then(|rest| rest.strip_suffix('\''))
    {
        return inner;
    }
    value
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn nonempty(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn command(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("off") || value.eq_ignore_ascii_case("none") {
        None
    } else {
        Some(value.to_string())
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

fn normalize_section(name: &str) -> String {
    name.trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != '_')
        .collect()
}

fn config_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("EMACETTE_CONFIG") {
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    let (new_path, legacy_path) = match env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|dir| !dir.is_empty())
    {
        Some(dir) => (
            PathBuf::from(&dir).join("emacette").join("config"),
            PathBuf::from(dir).join("nano-emacs").join("config"),
        ),
        None => {
            let home = env::var("HOME").ok()?;
            (
                PathBuf::from(&home)
                    .join(".config")
                    .join("emacette")
                    .join("config"),
                PathBuf::from(home)
                    .join(".config")
                    .join("nano-emacs")
                    .join("config"),
            )
        }
    };
    if new_path.is_file() {
        return Some(new_path);
    }
    // Migration fallback: the old environment variable is still honored.
    if let Ok(path) = env::var("NANO_EMACS_CONFIG") {
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    if legacy_path.is_file() {
        Some(legacy_path)
    } else {
        Some(new_path)
    }
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
    fn parses_per_language_sections() {
        let mut config = Config::default();
        config.parse(
            r##"
            tab_width = 8
            diagnostics_popup = false
            [rust]
            tab_width = 2
            comment_start = //
            lsp_command = rust-analyzer
            [markdown]
            spell_check = true
            diagnostics_inline = false
            [git-commit]
            comment_start = "#"
            "##,
        );
        let rust = config.settings_for(Lang::Rust);
        assert_eq!(rust.tab_width, 2);
        assert_eq!(rust.comment_start, "//");
        assert_eq!(rust.lsp_command.as_deref(), Some("rust-analyzer"));
        assert!(!rust.diagnostics_popup);

        let markdown = config.settings_for(Lang::Markdown);
        assert_eq!(markdown.tab_width, 8);
        assert!(markdown.spell_check);
        assert!(!markdown.diagnostics_inline);

        let commit = config.settings_for(Lang::GitCommit);
        assert_eq!(commit.comment_start, "#");
        assert_eq!(config.settings_for(Lang::Latex).comment_start, "%");
        assert_eq!(config.settings_for(Lang::Rust).comment_start, "//");
    }

    #[test]
    fn language_sections_can_disable_an_inherited_lsp_command() {
        let mut config = Config::default();
        config.parse("lsp_command = global-server\n[python]\nlsp_command = off\n");
        let python = config.settings_for(Lang::Python);
        assert_eq!(python.lsp_command.as_deref(), Some(""));
        assert_eq!(
            config.settings_for(Lang::Rust).lsp_command.as_deref(),
            Some("global-server")
        );
    }

    #[test]
    fn keeps_quotes_inside_a_command_line() {
        let mut config = Config::default();
        config.parse("[rust]\nlsp_command = python3 \"/tmp/a b/server.py\" --stdio\n");
        let settings = config.settings_for(Lang::Rust);
        assert_eq!(
            settings.lsp_command.as_deref(),
            Some("python3 \"/tmp/a b/server.py\" --stdio")
        );
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
