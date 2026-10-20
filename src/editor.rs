//! Editor state and key dispatch. This module has no terminal dependency so it
//! can be driven headlessly from tests.

use std::fs;
use std::io;
use std::path::PathBuf;

use crate::buffer::{Buffer, CaseOp, Coalesce, Point};
use crate::completion::{self, EditorCompletion};
use crate::config::{Config, Settings};
use crate::help;
use crate::highlight::Highlighter;
use crate::key::{Key, KeyCode};
use crate::lsp::{Diagnostic, LspClient, LspEvent, Severity};
use crate::minibuffer::{CompletionState, Minibuffer, PromptKind};
use crate::search::{find_backward, find_forward, find_last, SearchState};
use crate::spell::{SpellChecker, SpellState};
use crate::syntax::{Lang, Span};
use crate::theme::Theme;
use crate::view::{char_width, display_col, View};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmd {
    FindFile,
    Save,
    WriteFile,
    Quit,
    Undo,
    Redo,
    GotoLine,
    ToggleLineNumbers,
    ToggleTrailingWs,
    ToggleAutoFill,
    ToggleTruncateLines,
    ToggleEditorCompletion,
    CompleteSymbol,
    ToggleSpellCheck,
    CommentDwm,
    NextError,
    PreviousError,
    TabNext,
    TabPrevious,
    TabNew,
    TabClose,
    SpellWord,
    NextSpellingError,
    PreviousSpellingError,
    SpellCheckBuffer,
    SetMark,
    KillRegion,
    CopyRegion,
    DeleteRegion,
    ExchangePointAndMark,
    MarkWholeBuffer,
    DeactivateMark,
    Revert,
    DeleteTrailingWs,
    Help,
    SearchForward,
    SearchBackward,
    Beginning,
    End,
}

pub const COMMANDS: &[(&str, Cmd)] = &[
    ("find-file", Cmd::FindFile),
    ("save-buffer", Cmd::Save),
    ("write-file", Cmd::WriteFile),
    ("kill-emacs", Cmd::Quit),
    ("save-buffers-kill-terminal", Cmd::Quit),
    ("undo", Cmd::Undo),
    ("redo", Cmd::Redo),
    ("goto-line", Cmd::GotoLine),
    ("toggle-line-numbers", Cmd::ToggleLineNumbers),
    ("toggle-trailing-whitespace", Cmd::ToggleTrailingWs),
    ("auto-fill-mode", Cmd::ToggleAutoFill),
    ("toggle-auto-fill", Cmd::ToggleAutoFill),
    ("toggle-truncate-lines", Cmd::ToggleTruncateLines),
    ("toggle-editor-completion", Cmd::ToggleEditorCompletion),
    ("complete-symbol", Cmd::CompleteSymbol),
    ("completion-at-point", Cmd::CompleteSymbol),
    ("comment-dwim", Cmd::CommentDwm),
    ("next-error", Cmd::NextError),
    ("previous-error", Cmd::PreviousError),
    ("tab-next", Cmd::TabNext),
    ("tab-previous", Cmd::TabPrevious),
    ("tab-new", Cmd::TabNew),
    ("tab-close", Cmd::TabClose),
    ("flyspell-mode", Cmd::ToggleSpellCheck),
    ("spell-check-mode", Cmd::ToggleSpellCheck),
    ("toggle-spell-check", Cmd::ToggleSpellCheck),
    ("ispell-word", Cmd::SpellWord),
    ("spell-word", Cmd::SpellWord),
    ("ispell-next", Cmd::NextSpellingError),
    ("next-spelling-error", Cmd::NextSpellingError),
    ("ispell-previous", Cmd::PreviousSpellingError),
    ("previous-spelling-error", Cmd::PreviousSpellingError),
    ("ispell-buffer", Cmd::SpellCheckBuffer),
    ("spell-check-buffer", Cmd::SpellCheckBuffer),
    ("set-mark-command", Cmd::SetMark),
    ("kill-region", Cmd::KillRegion),
    ("kill-ring-save", Cmd::CopyRegion),
    ("copy-region-as-kill", Cmd::CopyRegion),
    ("delete-region", Cmd::DeleteRegion),
    ("exchange-point-and-mark", Cmd::ExchangePointAndMark),
    ("mark-whole-buffer", Cmd::MarkWholeBuffer),
    ("deactivate-mark", Cmd::DeactivateMark),
    ("revert-buffer", Cmd::Revert),
    ("delete-trailing-whitespace", Cmd::DeleteTrailingWs),
    ("help", Cmd::Help),
    ("describe-bindings", Cmd::Help),
    ("search-forward", Cmd::SearchForward),
    ("search-backward", Cmd::SearchBackward),
    ("beginning-of-buffer", Cmd::Beginning),
    ("end-of-buffer", Cmd::End),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Prefix {
    Cx,
    Cc,
    CxT,
    MetaG,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConfirmKind {
    QuitSave,
    QuitSaveUnnamed,
    AbortCommit,
    EmptyCommit,
}

#[derive(Clone, Debug)]
struct Confirm {
    kind: ConfirmKind,
    question: String,
}

#[derive(Clone, Copy, Debug)]
pub struct Overlay {
    pub scroll: usize,
}

struct TabState {
    buffer: Option<Buffer>,
    top: usize,
    top_offset: usize,
    left: usize,
    lsp: Option<LspClient>,
    diagnostics: Vec<Diagnostic>,
    last_lsp_version: u64,
    settings: Settings,
    mode: Lang,
    spell: SpellState,
}

#[derive(Clone, Debug)]
struct LspCompletionContext {
    request_id: i64,
    start: Point,
    end: Point,
    prefix: String,
}

pub struct Editor {
    pub buffer: Buffer,
    pub mode: Lang,
    pub theme: Theme,
    pub view: View,
    pub config: Config,
    pub settings: Settings,
    pub highlighter: Highlighter,
    pub minibuffer: Option<Minibuffer>,
    pub search: Option<SearchState>,
    pub editor_completion: Option<EditorCompletion>,
    pub spell: SpellState,
    word_cache: Option<(u64, Vec<String>)>,
    pub lsp: Option<LspClient>,
    pub diagnostics: Vec<Diagnostic>,
    lsp_completion: Option<LspCompletionContext>,
    last_lsp_version: u64,
    tabs: Vec<TabState>,
    active_tab: usize,
    confirm: Option<Confirm>,
    pub overlay: Option<Overlay>,
    pub message: String,
    prefix: Option<Prefix>,
    meta_prefix: bool,
    pub should_quit: bool,
    pub exit_code: i32,
    pub history: Vec<String>,
    /// Set to request a full repaint on the next frame (e.g. C-l).
    pub needs_redraw: bool,
}

impl Editor {
    pub fn open(path: Option<PathBuf>, config: Config) -> io::Result<Self> {
        match path {
            Some(path) => {
                if path.is_dir() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("{} is a directory", path.display()),
                    ));
                }
                if path.exists() {
                    let buffer = Buffer::open(&path)?;
                    Ok(Self::from_buffer(buffer, config))
                } else {
                    let buffer = Buffer::from_text("", Some(path.clone()));
                    let mut editor = Self::from_buffer(buffer, config);
                    editor.message = format!("(New file) {}", path.display());
                    Ok(editor)
                }
            }
            None => Ok(Self::from_buffer(Buffer::new(), config)),
        }
    }

    pub fn from_text(text: &str, path: Option<PathBuf>, config: Config) -> Self {
        Self::from_buffer(Buffer::from_text(text, path), config)
    }

    pub fn from_buffer(buffer: Buffer, config: Config) -> Self {
        let first_line: String = buffer
            .lines
            .first()
            .map(|l| l.iter().copied().collect())
            .unwrap_or_default();
        let mode = Lang::detect(buffer.path.as_deref(), Some(&first_line));
        let theme = Theme::by_name(config.theme);
        let settings = config.settings_for(mode);
        let version = buffer.version;
        let spell_checker = if settings.spell_check {
            SpellChecker::load(settings.spell_dictionary.as_deref())
        } else {
            None
        };
        let spell_missing = settings.spell_check && spell_checker.is_none();
        let spell = SpellState::new(settings.spell_check, spell_checker);
        let view = View {
            line_numbers: settings.line_numbers,
            tab_width: settings.tab_width,
            highlight_trailing_ws: settings.highlight_trailing_ws,
            truncate_lines: settings.truncate_lines,
            ..View::default()
        };
        let mut editor = Self {
            buffer,
            mode,
            theme,
            view,
            config,
            settings: settings.clone(),
            highlighter: Highlighter::new(mode, version),
            minibuffer: None,
            search: None,
            editor_completion: None,
            spell,
            word_cache: None,
            lsp: None,
            diagnostics: Vec::new(),
            lsp_completion: None,
            last_lsp_version: version,
            tabs: vec![TabState {
                buffer: None,
                top: 0,
                top_offset: 0,
                left: 0,
                lsp: None,
                diagnostics: Vec::new(),
                last_lsp_version: version,
                settings: settings.clone(),
                mode,
                spell: SpellState::default(),
            }],
            active_tab: 0,
            confirm: None,
            overlay: None,
            message: String::new(),
            prefix: None,
            meta_prefix: false,
            should_quit: false,
            exit_code: 0,
            history: Vec::new(),
            needs_redraw: false,
        };
        if spell_missing {
            editor.message = "Spell check: no English dictionary found".into();
        }
        editor.start_lsp();
        editor
    }

    // ------------------------------------------------------------- helpers

    fn apply_settings(&mut self) {
        self.view.tab_width = self.settings.tab_width;
        self.view.line_numbers = self.settings.line_numbers;
        self.view.highlight_trailing_ws = self.settings.highlight_trailing_ws;
        self.view.truncate_lines = self.settings.truncate_lines;
    }

    pub fn sync_highlighter(&mut self) {
        if self.highlighter.lang != self.mode {
            self.highlighter = Highlighter::new(self.mode, self.buffer.version);
        } else if self.highlighter.version != self.buffer.version {
            self.highlighter.invalidate(self.buffer.dirty_from);
            self.highlighter.version = self.buffer.version;
            self.buffer.dirty_from = usize::MAX;
        }
    }

    pub fn buffer_text(&self) -> String {
        self.buffer.text()
    }

    pub fn hint(&self) -> String {
        if self.mode == Lang::GitCommit {
            "C-c C-c commit   C-c C-k abort   C-x C-s save   F1 help".to_string()
        } else {
            "C-x C-s save   C-x C-c quit   C-s search   M-x commands   F1 help".to_string()
        }
    }

    pub fn minibuffer_text(&self) -> Option<String> {
        self.minibuffer.as_ref().map(Minibuffer::text)
    }

    fn page(&mut self, direction: isize) {
        let step = self.view.body_height().saturating_sub(1).max(1) as isize;
        self.buffer.move_vertical(direction * step);
    }

    fn finish_quit(&mut self, code: i32) {
        self.should_quit = true;
        self.exit_code = code;
    }

    fn set_buffer(&mut self, buffer: Buffer) {
        let first_line: String = buffer
            .lines
            .first()
            .map(|l| l.iter().copied().collect())
            .unwrap_or_default();
        self.mode = Lang::detect(buffer.path.as_deref(), Some(&first_line));
        self.settings = self.config.settings_for(self.mode);
        self.highlighter = Highlighter::new(self.mode, buffer.version);
        self.buffer = buffer;
        self.apply_settings();
        self.view.top = 0;
        self.view.top_offset = 0;
        self.view.left = 0;
        self.search = None;
        self.editor_completion = None;
        self.word_cache = None;
        self.spell.clear();
        self.configure_spell();
        self.start_lsp();
        self.prefix = None;
        self.needs_redraw = true;
    }

    // --------------------------------------------------------------- tabs

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    fn first_unsaved_other_tab(&self) -> Option<usize> {
        self.tabs
            .iter()
            .enumerate()
            .find(|(index, tab)| {
                *index != self.active_tab
                    && tab.buffer.as_ref().is_some_and(Buffer::has_unsaved_changes)
            })
            .map(|(index, _)| index)
    }

    pub fn tab_labels(&self) -> Vec<(String, bool)> {
        (0..self.tabs.len())
            .map(|index| {
                let buffer = if index == self.active_tab {
                    Some(&self.buffer)
                } else {
                    self.tabs[index].buffer.as_ref()
                };
                let name = buffer
                    .map(Buffer::display_name)
                    .unwrap_or_else(|| "*scratch*".to_string());
                let star = if buffer.is_some_and(|buffer| buffer.modified) {
                    "*"
                } else {
                    ""
                };
                (format!("{name}{star}"), index == self.active_tab)
            })
            .collect()
    }

    fn stash_active_tab(&mut self) {
        let active = self.active_tab;
        let tab = &mut self.tabs[active];
        tab.buffer = Some(std::mem::take(&mut self.buffer));
        tab.top = self.view.top;
        tab.top_offset = self.view.top_offset;
        tab.left = self.view.left;
        tab.lsp = self.lsp.take();
        tab.diagnostics = std::mem::take(&mut self.diagnostics);
        tab.last_lsp_version = self.last_lsp_version;
        tab.settings = self.settings.clone();
        tab.mode = self.mode;
        tab.spell = std::mem::take(&mut self.spell);
    }

    /// Load `active_tab`, whose saved buffer must be present.
    fn load_active_tab(&mut self) {
        let active = self.active_tab;
        let (buffer, top, top_offset, left, lsp, diagnostics, last_version, settings, mode, spell) = {
            let tab = &mut self.tabs[active];
            (
                tab.buffer.take().expect("active tab has no buffer"),
                tab.top,
                tab.top_offset,
                tab.left,
                tab.lsp.take(),
                std::mem::take(&mut tab.diagnostics),
                tab.last_lsp_version,
                tab.settings.clone(),
                tab.mode,
                std::mem::take(&mut tab.spell),
            )
        };
        self.buffer = buffer;
        self.mode = mode;
        self.settings = settings;
        self.view.top = top;
        self.view.top_offset = top_offset;
        self.view.left = left;
        self.highlighter = Highlighter::new(mode, self.buffer.version);
        self.apply_settings();
        self.lsp = lsp;
        self.diagnostics = diagnostics;
        self.last_lsp_version = last_version;
        self.spell = spell;
        self.lsp_completion = None;
        self.editor_completion = None;
        self.word_cache = None;
        self.search = None;
        self.minibuffer = None;
        self.confirm = None;
        self.prefix = None;
        self.needs_redraw = true;
        if self.lsp.is_none() {
            self.start_lsp();
        }
        if self.settings.spell_check && !self.spell.enabled {
            self.configure_spell();
        }
    }

    fn switch_to_tab(&mut self, index: usize) {
        if index == self.active_tab || index >= self.tabs.len() {
            return;
        }
        self.stash_active_tab();
        self.active_tab = index;
        self.load_active_tab();
        self.message = format!("Tab: {}", self.buffer.display_name());
    }

    fn switch_tab(&mut self, delta: isize) {
        if self.tabs.len() <= 1 {
            self.message = "Only one tab".into();
            return;
        }
        let len = self.tabs.len() as isize;
        let index = (self.active_tab as isize + delta).rem_euclid(len) as usize;
        self.switch_to_tab(index);
    }

    fn new_tab(&mut self) {
        let buffer = Buffer::new();
        let version = buffer.version;
        let mode = Lang::Plain;
        let settings = self.config.settings_for(mode);
        self.stash_active_tab();
        self.tabs.push(TabState {
            buffer: Some(buffer),
            top: 0,
            top_offset: 0,
            left: 0,
            lsp: None,
            diagnostics: Vec::new(),
            last_lsp_version: version,
            settings,
            mode,
            spell: SpellState::default(),
        });
        self.active_tab = self.tabs.len() - 1;
        self.load_active_tab();
        self.message = "New tab".into();
    }

    fn close_tab(&mut self) {
        if self.buffer.has_unsaved_changes() {
            self.message = "Tab modified: save it first".into();
            return;
        }
        if self.tabs.len() == 1 {
            self.finish_quit(0);
            return;
        }
        let index = self.active_tab;
        self.lsp = None;
        self.diagnostics.clear();
        self.lsp_completion = None;
        self.spell = SpellState::default();
        self.tabs.remove(index);
        self.active_tab = index.min(self.tabs.len() - 1);
        self.buffer = Buffer::new();
        self.load_active_tab();
        self.message = "Closed tab".into();
    }

    fn open_in_tab(&mut self, buffer: Buffer, new_file: bool) {
        let name = buffer.display_name();
        let lines = buffer.line_count();
        if let Some(path) = buffer.path.as_deref() {
            if self.buffer.path.as_deref() == Some(path) {
                self.message = format!("Already open: {name}");
                return;
            }
            if let Some(index) = self.tabs.iter().position(|tab| {
                tab.buffer
                    .as_ref()
                    .and_then(|buffer| buffer.path.as_deref())
                    == Some(path)
            }) {
                self.switch_to_tab(index);
                self.message = format!("Switched to tab: {name}");
                return;
            }
        }
        let reuse_scratch = self.buffer.path.is_none() && !self.buffer.has_unsaved_changes();
        if reuse_scratch {
            self.set_buffer(buffer);
        } else {
            let version = buffer.version;
            let first_line: String = buffer
                .lines
                .first()
                .map(|line| line.iter().copied().collect())
                .unwrap_or_default();
            let mode = Lang::detect(buffer.path.as_deref(), Some(&first_line));
            let settings = self.config.settings_for(mode);
            self.stash_active_tab();
            self.tabs.push(TabState {
                buffer: Some(buffer),
                top: 0,
                top_offset: 0,
                left: 0,
                lsp: None,
                diagnostics: Vec::new(),
                last_lsp_version: version,
                settings,
                mode,
                spell: SpellState::default(),
            });
            self.active_tab = self.tabs.len() - 1;
            self.load_active_tab();
        }
        self.message = if new_file {
            format!("(New file) {name}")
        } else {
            format!("Opened {name} ({lines} lines)")
        };
    }

    // ---------------------------------------------------------- key input

    pub fn handle_key(&mut self, key: Key) {
        if self.overlay.is_some() {
            self.handle_overlay(key);
            return;
        }
        if self.confirm.is_some() {
            self.buffer.begin_command();
            self.handle_confirm(key);
            self.buffer.end_command(None);
            return;
        }
        if self.minibuffer.is_some() {
            self.buffer.begin_command();
            self.handle_minibuffer(key);
            self.buffer.end_command(None);
            return;
        }
        if self.search.is_some() {
            if self.is_search_key(&key) {
                self.buffer.begin_command();
                self.handle_search(key);
                self.buffer.end_command(None);
                return;
            }
            self.finish_search(true);
        }

        let self_insert = matches!(key.code, KeyCode::Char(_)) && !key.ctrl && !key.alt;
        let backspace = matches!(key.code, KeyCode::Backspace) && !key.alt;
        let coalesce = if self_insert {
            Some(Coalesce::Insert)
        } else if backspace {
            Some(Coalesce::Delete)
        } else {
            None
        };
        self.buffer.begin_command();
        self.handle_normal(key);
        if (self.settings.editor_completion || self.editor_completion.is_some())
            && (self_insert || backspace)
        {
            self.update_editor_completion(false);
        }
        self.buffer.end_command(coalesce);
    }

    pub fn handle_paste(&mut self, text: &str) {
        if self.overlay.is_some() || self.confirm.is_some() {
            return;
        }
        if let Some(mb) = self.minibuffer.as_mut() {
            mb.insert_str(text);
            self.prime_completion();
            return;
        }
        if self.search.is_some() {
            let filtered: String = text.chars().filter(|c| *c != '\n' && *c != '\r').collect();
            if let Some(search) = self.search.as_mut() {
                search.query.extend(filtered.chars());
            }
            self.update_search();
            return;
        }
        self.editor_completion = None;
        self.buffer.begin_command();
        self.buffer.insert_text(text);
        self.buffer.end_command(None);
    }

    fn handle_editor_completion_key(&mut self, key: Key) -> bool {
        let word_char = matches!(
            key.code,
            KeyCode::Char(c) if !key.ctrl && !key.alt && (completion::is_word_char(c) || c == '\\')
        );
        if word_char || (key.code == KeyCode::Backspace && !key.alt) {
            return false;
        }
        match key.code {
            KeyCode::Tab | KeyCode::Enter if !key.ctrl && !key.alt => {
                self.accept_editor_completion();
            }
            KeyCode::Esc => {
                self.editor_completion = None;
                self.lsp_completion = None;
                self.message = "Quit".into();
            }
            KeyCode::Char('g') if key.ctrl => {
                self.editor_completion = None;
                self.lsp_completion = None;
                self.message = "Quit".into();
            }
            KeyCode::Up => self.move_editor_completion(false),
            KeyCode::Down => self.move_editor_completion(true),
            KeyCode::Char('p') if key.ctrl => self.move_editor_completion(false),
            KeyCode::Char('n') if key.ctrl => self.move_editor_completion(true),
            _ => {
                self.editor_completion = None;
                self.lsp_completion = None;
                return false;
            }
        }
        true
    }

    fn move_editor_completion(&mut self, forward: bool) {
        if let Some(completion) = self.editor_completion.as_mut() {
            if forward {
                completion.next();
            } else {
                completion.previous();
            }
        }
    }

    fn accept_editor_completion(&mut self) {
        self.lsp_completion = None;
        let Some(completion) = self.editor_completion.take() else {
            return;
        };
        let Some(word) = completion.selected().map(str::to_string) else {
            return;
        };
        self.buffer
            .replace_range(completion.start, completion.end, &word);
        self.message = format!("Completed: {word}");
    }

    fn update_editor_completion(&mut self, force: bool) {
        if self.minibuffer.is_some() || self.overlay.is_some() {
            self.editor_completion = None;
            self.lsp_completion = None;
            return;
        }
        let row = self.buffer.point.row;
        let end = self.buffer.point;
        let (start, prefix) = {
            let line = &self.buffer.lines[row];
            completion::prefix_at(line, end.col)
        };
        let lsp_available = self.lsp.is_some();
        let minimum = if force {
            if lsp_available {
                0
            } else {
                1
            }
        } else {
            2
        };
        if prefix.chars().count() < minimum {
            self.editor_completion = None;
            self.lsp_completion = None;
            return;
        }
        self.sync_lsp();
        let request_id = self
            .lsp
            .as_mut()
            .and_then(|lsp| lsp.request_completion(row, end.col));
        self.lsp_completion = request_id.map(|request_id| LspCompletionContext {
            request_id,
            start: Point::new(row, start),
            end,
            prefix: prefix.clone(),
        });

        let candidates = if prefix.is_empty() {
            Vec::new()
        } else {
            let words = self.completion_words();
            completion::candidates(&prefix, &words, 100)
        };
        if candidates.is_empty() {
            self.editor_completion = None;
            if force && request_id.is_none() {
                self.message = "No completion at point".into();
            } else if force {
                self.message = "Waiting for LSP completion".into();
            }
            return;
        }
        self.editor_completion = Some(EditorCompletion {
            candidates,
            index: 0,
            start: Point::new(row, start),
            end,
            prefix,
        });
    }

    fn completion_words(&mut self) -> Vec<String> {
        if let Some((version, words)) = &self.word_cache {
            if *version == self.buffer.version {
                return words.clone();
            }
        }
        let words = completion::collect_words(&self.buffer.lines, self.mode);
        self.word_cache = Some((self.buffer.version, words.clone()));
        words
    }

    /// Candidates currently shown in the completion menu, if any.
    pub fn completion_menu(&self) -> Option<(&[String], Option<usize>)> {
        if let Some(minibuffer) = &self.minibuffer {
            if let Some(completion) = &minibuffer.completion {
                if !completion.candidates.is_empty() {
                    return Some((&completion.candidates, completion.index));
                }
            }
        }
        if let Some(completion) = &self.editor_completion {
            if !completion.candidates.is_empty() {
                return Some((&completion.candidates, Some(completion.index)));
            }
        }
        None
    }

    pub fn completion_menu_count(&self) -> usize {
        self.completion_menu()
            .map(|(candidates, _)| candidates.len())
            .unwrap_or(0)
    }

    pub fn spell_spans(&mut self, row: usize) -> Vec<(usize, usize)> {
        if !self.spell.enabled || !self.mode.is_prose() || row >= self.buffer.line_count() {
            return Vec::new();
        }
        self.sync_highlighter();
        let syntax: Vec<Span> = {
            let lines = &self.buffer.lines;
            self.highlighter.tokens(row, lines).to_vec()
        };
        let line = &self.buffer.lines[row];
        self.spell.spans(self.buffer.version, row, line, &syntax)
    }

    fn start_lsp(&mut self) {
        self.lsp = None;
        self.diagnostics.clear();
        self.lsp_completion = None;
        let command = self
            .settings
            .lsp_command
            .as_deref()
            .filter(|command| !command.is_empty());
        let Some(command) = command else {
            return;
        };
        let Some(path) = self.buffer.path.clone() else {
            return;
        };
        let language_id = self
            .settings
            .lsp_language_id
            .clone()
            .unwrap_or_else(|| self.mode.lsp_language_id().to_string());
        let text = self.buffer.text();
        match LspClient::start(command, &language_id, &path, &text) {
            Ok(client) => {
                self.lsp = Some(client);
                self.last_lsp_version = self.buffer.version;
            }
            Err(error) => {
                self.message = format!("LSP: {error}");
            }
        }
    }

    /// Send full-text changes to the language server once per buffer version.
    pub fn sync_lsp(&mut self) {
        let Some(lsp) = self.lsp.as_mut() else {
            return;
        };
        if self.last_lsp_version == self.buffer.version {
            return;
        }
        lsp.did_change(&self.buffer.text());
        self.last_lsp_version = self.buffer.version;
    }

    /// Drain language-server events. Returns true when the display changed.
    pub fn poll_lsp(&mut self) -> bool {
        self.sync_lsp();
        let events = match self.lsp.as_mut() {
            Some(lsp) => lsp.poll(),
            None => return false,
        };
        let mut changed = false;
        for event in events {
            match event {
                LspEvent::Diagnostics(diagnostics) => {
                    self.diagnostics = diagnostics;
                    changed = true;
                }
                LspEvent::Completion { request_id, items } => {
                    if let Some(context) = self.lsp_completion.take() {
                        if context.request_id == request_id {
                            changed |= self.apply_lsp_completion(context, items);
                        }
                    }
                }
                LspEvent::Error(error) => {
                    self.message = format!("LSP: {error}");
                    changed = true;
                }
            }
        }
        changed
    }

    fn apply_lsp_completion(&mut self, context: LspCompletionContext, items: Vec<String>) -> bool {
        if items.is_empty() || self.minibuffer.is_some() || self.overlay.is_some() {
            return false;
        }
        if self.buffer.point != context.end {
            return false;
        }
        let line = &self.buffer.lines[self.buffer.point.row];
        let (start, prefix) = completion::prefix_at(line, self.buffer.point.col);
        if start != context.start.col || prefix != context.prefix {
            return false;
        }
        self.editor_completion = Some(EditorCompletion {
            candidates: items,
            index: 0,
            start: context.start,
            end: context.end,
            prefix,
        });
        true
    }

    pub fn diagnostic_ranges(&self, row: usize) -> Vec<(usize, usize, Severity)> {
        if !self.settings.diagnostics_inline || row >= self.buffer.line_count() {
            return Vec::new();
        }
        let len = self.buffer.lines[row].len();
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.row == row)
            .filter_map(|diagnostic| {
                let start = diagnostic.start.min(len);
                let end = diagnostic.end.min(len);
                if end > start {
                    Some((start, end, diagnostic.severity))
                } else if start < len {
                    Some((start, start + 1, diagnostic.severity))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn diagnostic_popup(&self) -> Vec<&Diagnostic> {
        if !self.settings.diagnostics_popup {
            return Vec::new();
        }
        let row = self.buffer.point.row;
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.row == row)
            .take(4)
            .collect()
    }

    fn configure_spell(&mut self) {
        self.spell.clear();
        if !self.settings.spell_check || !self.mode.is_prose() {
            self.spell.enabled = false;
            return;
        }
        if self.spell.checker.is_none() {
            self.spell.checker = SpellChecker::load(self.settings.spell_dictionary.as_deref());
        }
        self.spell.enabled = self.spell.checker.is_some();
        if !self.spell.enabled {
            self.message = "Spell check: no English dictionary found (set spell_dictionary)".into();
        }
    }

    fn enable_spell_check(&mut self) -> bool {
        if self.spell.enabled {
            return true;
        }
        if !self.mode.is_prose() {
            self.message = "Spell check is available in text modes".into();
            return false;
        }
        if self.spell.checker.is_none() {
            self.spell.checker = SpellChecker::load(self.settings.spell_dictionary.as_deref());
        }
        if self.spell.checker.is_some() {
            self.spell.enabled = true;
            self.settings.spell_check = true;
            self.spell.clear();
            true
        } else {
            self.message =
                "No English dictionary found (install one or set spell_dictionary)".into();
            false
        }
    }

    fn toggle_spell_check(&mut self) {
        if self.spell.enabled {
            self.spell.enabled = false;
            self.settings.spell_check = false;
            self.spell.clear();
            self.message = "Spell check off".into();
        } else if self.enable_spell_check() {
            let source = self
                .spell
                .checker
                .as_ref()
                .map(|checker| checker.source().display().to_string())
                .unwrap_or_default();
            self.message = format!("Spell check on ({source})");
        }
    }

    fn spell_word(&mut self) {
        if !self.enable_spell_check() {
            return;
        }
        let row = self.buffer.point.row;
        let found = completion::word_at(&self.buffer.lines[row], self.buffer.point.col);
        let Some((start, end, word)) = found else {
            self.message = "No word at point".into();
            return;
        };
        let (correct, suggestions) = {
            let Some(checker) = self.spell.checker.as_ref() else {
                return;
            };
            (checker.is_correct(&word), checker.suggestions(&word, 8))
        };
        if correct {
            self.message = format!("Correct: {word}");
        } else if suggestions.is_empty() {
            self.message = format!("Misspelled: {word} (no suggestions)");
        } else {
            self.message.clear();
            let mut minibuffer = Minibuffer::new(
                format!("Replace '{word}' with: "),
                PromptKind::SpellReplace {
                    start: Point::new(row, start),
                    end: Point::new(row, end),
                },
                &suggestions[0],
            );
            minibuffer.hint = suggestions.join("  ");
            self.minibuffer = Some(minibuffer);
        }
    }

    pub fn spell_check_buffer(&mut self) -> usize {
        if !self.enable_spell_check() {
            return 0;
        }
        let mut count = 0;
        for row in 0..self.buffer.line_count() {
            count += self.spell_spans(row).len();
        }
        self.message = format!("{count} misspelled word(s)");
        count
    }

    fn next_spelling_error(&mut self, forward: bool) {
        if !self.enable_spell_check() {
            return;
        }
        let current = self.buffer.point;
        let row_count = self.buffer.line_count();
        let mut found = None;
        if forward {
            for row in current.row..row_count {
                let errors = self.spell_spans(row);
                for (start, _) in errors {
                    if row > current.row || start > current.col {
                        found = Some((row, start));
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }
        } else {
            for row in (0..=current.row.min(row_count.saturating_sub(1))).rev() {
                let errors = self.spell_spans(row);
                for (start, end) in errors.into_iter().rev() {
                    if row < current.row || end <= current.col {
                        found = Some((row, start));
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }
        }
        match found {
            Some((row, col)) => {
                self.buffer.set_point(Point::new(row, col));
                self.message = format!("Misspelled word at L{} C{}", row + 1, col + 1);
            }
            None => self.message = "No more misspelled words".into(),
        }
    }

    fn handle_normal(&mut self, mut key: Key) {
        if self.editor_completion.is_some() && self.handle_editor_completion_key(key) {
            return;
        }
        self.message.clear();
        if key.code == KeyCode::Esc || (key.ctrl && matches!(key.code, KeyCode::Char('g'))) {
            self.lsp_completion = None;
        }
        if self.meta_prefix {
            self.meta_prefix = false;
            let cancel =
                key.code == KeyCode::Esc || (key.ctrl && matches!(key.code, KeyCode::Char('g')));
            if cancel {
                self.message = "Quit".into();
                return;
            }
            key.alt = true;
        }
        if let Some(prefix) = self.prefix.take() {
            self.handle_prefix(prefix, key);
            return;
        }

        match key.code {
            KeyCode::Char(c) if key.ctrl => self.handle_ctrl(c),
            KeyCode::Char(c) if key.alt => self.handle_alt(c),
            KeyCode::Char(c) => {
                self.buffer.insert_char(c);
                if self.settings.auto_fill {
                    self.auto_fill();
                }
            }
            KeyCode::Enter => {
                self.buffer.newline(self.settings.auto_indent);
            }
            KeyCode::Tab => self.indent(),
            KeyCode::BackTab => self.unindent(),
            KeyCode::Backspace if key.alt => {
                self.buffer.kill_word_backward();
            }
            KeyCode::Backspace => {
                if !self.delete_active_region() {
                    self.buffer.delete_backward();
                }
            }
            KeyCode::Delete => {
                if !self.delete_active_region() {
                    self.buffer.delete_forward();
                }
            }
            KeyCode::Esc => {
                self.meta_prefix = true;
                self.message = "M-".into();
            }
            KeyCode::Left if key.alt => self.switch_tab(-1),
            KeyCode::Right if key.alt => self.switch_tab(1),
            KeyCode::Left => self.buffer.move_left(),
            KeyCode::Right => self.buffer.move_right(),
            KeyCode::Up => self.buffer.move_vertical(-1),
            KeyCode::Down => self.buffer.move_vertical(1),
            KeyCode::Home => self.buffer.move_line_start(),
            KeyCode::End => self.buffer.move_line_end(),
            KeyCode::PageUp => self.page(-1),
            KeyCode::PageDown => self.page(1),
            KeyCode::F(1) => self.open_help(),
            KeyCode::Insert => {}
            _ => {
                self.message = format!("{} is unbound", key.display());
            }
        }
    }

    fn handle_ctrl(&mut self, c: char) {
        let c = crate::key::normalize_control(c);
        match c.to_ascii_lowercase() {
            'a' => self.buffer.move_line_start(),
            'e' => self.buffer.move_line_end(),
            'f' => self.buffer.move_right(),
            'b' => self.buffer.move_left(),
            'n' => self.buffer.move_vertical(1),
            'p' => self.buffer.move_vertical(-1),
            'v' => self.page(1),
            'd' => {
                if !self.delete_active_region() {
                    self.buffer.delete_forward();
                }
            }
            'h' => {
                if !self.delete_active_region() {
                    self.buffer.delete_backward();
                }
            }
            'k' => {
                self.buffer.kill_line();
            }
            'y' => self.yank(),
            'w' => self.kill_region(),
            'g' => {
                self.buffer.deactivate_mark();
                self.message = "Quit".into();
            }
            's' => self.start_search(1),
            'r' => self.start_search(-1),
            'l' => {
                self.view.recenter(&self.buffer);
                self.needs_redraw = true;
            }
            't' => {
                self.buffer.transpose_chars();
            }
            'o' => {
                self.buffer.open_line();
            }
            'j' => {
                self.buffer.newline(self.settings.auto_indent);
            }
            'x' => {
                self.prefix = Some(Prefix::Cx);
                self.message = "C-x-".into();
            }
            'c' => {
                self.prefix = Some(Prefix::Cc);
                self.message = "C-c-".into();
            }
            // crossterm reports the raw 0x1f byte (C-_ and C-/) as C-7,
            // so accept that spelling too.
            '/' | '_' | '7' => self.undo(),
            ' ' | '@' => {
                self.buffer.set_mark();
                self.message = "Mark set".into();
            }
            _ => {
                self.message = format!("C-{} is unbound", c);
            }
        }
    }

    fn handle_alt(&mut self, c: char) {
        match c.to_ascii_lowercase() {
            'f' => self.buffer.forward_word(),
            'b' => self.buffer.backward_word(),
            'd' => {
                self.buffer.kill_word_forward();
            }
            'u' => {
                self.buffer.transform_case(CaseOp::Upcase);
            }
            'l' => {
                self.buffer.transform_case(CaseOp::Downcase);
            }
            'c' => {
                self.buffer.transform_case(CaseOp::Capitalize);
            }
            '<' => self.buffer.move_buffer_start(),
            '>' => self.buffer.move_buffer_end(),
            'v' => self.page(-1),
            'w' => self.copy_region(),
            'y' => {
                if self.buffer.yank_pop() {
                    self.message = "Yank-pop".into();
                } else {
                    self.message = "No previous yank".into();
                }
            }
            'g' => {
                self.prefix = Some(Prefix::MetaG);
                self.message = "M-g-".into();
            }
            'x' => self.start_mx(),
            'm' => self.buffer.back_to_indentation(),
            ';' => self.comment_dwim(),
            '$' => self.spell_word(),
            _ => {
                self.message = format!("M-{} is unbound", c);
            }
        }
    }

    fn handle_prefix(&mut self, prefix: Prefix, key: Key) {
        if key.ctrl && matches!(key.code, KeyCode::Char('g')) {
            self.message = "Quit".into();
            return;
        }
        match prefix {
            Prefix::Cx => match key.code {
                KeyCode::Char('s') if key.ctrl => self.save_buffer(),
                KeyCode::Char('c') if key.ctrl => self.quit_request(),
                KeyCode::Char('w') if key.ctrl => self.prompt_write_file(false),
                KeyCode::Char('f') if key.ctrl => self.prompt_find_file(),
                KeyCode::Char('x') if key.ctrl => {
                    if !self.buffer.exchange_point_and_mark() {
                        self.message = "No mark set".into();
                    }
                }
                KeyCode::Char('u') if key.ctrl => self.undo(),
                KeyCode::Char('h') if key.ctrl => {
                    self.buffer.mark_whole_buffer();
                    self.message = "Mark set".into();
                }
                KeyCode::Char('t') if !key.ctrl && !key.alt => {
                    self.prefix = Some(Prefix::CxT);
                    self.message = "C-x t-".into();
                }
                KeyCode::Char('s') => self.save_buffer(),
                _ => self.message = format!("C-x {} is unbound", key.display()),
            },
            Prefix::Cc => match key.code {
                KeyCode::Char('c') if key.ctrl => self.save_and_quit(),
                KeyCode::Char('k') if key.ctrl => self.abort_commit(),
                _ => self.message = format!("C-c {} is unbound", key.display()),
            },
            Prefix::CxT => match key.code {
                KeyCode::Char('n') if !key.alt || key.ctrl => self.switch_tab(1),
                KeyCode::Char('p') if !key.alt || key.ctrl => self.switch_tab(-1),
                KeyCode::Char('o') if !key.ctrl && !key.alt => self.switch_tab(1),
                KeyCode::Char('2') if !key.ctrl && !key.alt => self.new_tab(),
                KeyCode::Char('k') if !key.ctrl && !key.alt => self.close_tab(),
                KeyCode::Char('f') if !key.ctrl && !key.alt => self.prompt_find_file(),
                _ => self.message = format!("C-x t {} is unbound", key.display()),
            },
            Prefix::MetaG => match key.code {
                KeyCode::Char('g') if !key.ctrl && !key.alt => self.prompt_goto_line(),
                KeyCode::Char('n') if !key.ctrl && !key.alt => self.next_diagnostic(true),
                KeyCode::Char('p') if !key.ctrl && !key.alt => self.next_diagnostic(false),
                _ => self.message = format!("M-g {} is unbound", key.display()),
            },
        }
    }

    // ------------------------------------------------------------ commands

    fn undo(&mut self) {
        if self.buffer.undo() {
            self.message = "Undo".into();
        } else {
            self.message = "No further undo information".into();
        }
    }

    fn yank(&mut self) {
        if self.buffer.yank() {
            self.message = "Yanked".into();
        } else {
            self.message = "Kill ring is empty".into();
        }
    }

    fn delete_active_region(&mut self) -> bool {
        if self.buffer.region().is_none() {
            return false;
        }
        let removed = self.buffer.delete_region();
        self.message = format!("Deleted {} characters", removed.chars().count());
        true
    }

    fn kill_region(&mut self) {
        if self.buffer.kill_region().is_empty() {
            self.message = "No active region".into();
        } else {
            self.message = "Killed region".into();
        }
    }

    fn copy_region(&mut self) {
        if self.buffer.copy_region().is_empty() {
            self.message = "No active region".into();
        } else {
            self.message = "Copied region".into();
        }
    }

    fn comment_dwim(&mut self) {
        let prefix = self.settings.comment_start.clone();
        if prefix.is_empty() {
            self.message = "No line-comment syntax for this mode".into();
            return;
        }
        let (start, end) = match self.buffer.region() {
            Some((start, end)) => (start, end),
            None => {
                let row = self.buffer.point.row;
                (
                    Point::new(row, 0),
                    Point::new(row, self.buffer.lines[row].len()),
                )
            }
        };
        let rows: Vec<usize> = (start.row..=end.row.min(self.buffer.line_count() - 1)).collect();
        if rows.is_empty() {
            return;
        }
        let mut nonblank = 0usize;
        let mut commented = 0usize;
        for &row in &rows {
            let line = &self.buffer.lines[row];
            let indent = line.iter().take_while(|c| c.is_whitespace()).count();
            let text: String = line.iter().skip(indent).collect();
            if text.is_empty() {
                continue;
            }
            nonblank += 1;
            if text.starts_with(&prefix) {
                commented += 1;
            }
        }
        if nonblank == 0 {
            self.message = "Nothing to comment".into();
            return;
        }
        let remove = commented == nonblank;
        for &row in rows.iter().rev() {
            let line = self.buffer.lines[row].clone();
            let indent = line.iter().take_while(|c| c.is_whitespace()).count();
            if line.iter().skip(indent).all(|c| c.is_whitespace()) {
                continue;
            }
            self.buffer.set_point(Point::new(row, indent));
            if remove {
                for _ in 0..prefix.chars().count() {
                    self.buffer.delete_forward();
                }
                if self.buffer.lines[row].get(indent) == Some(&' ') {
                    self.buffer.delete_forward();
                }
            } else {
                self.buffer.insert_text(&format!("{prefix} "));
            }
        }
        self.message = if remove {
            "Uncommented".into()
        } else {
            "Commented".into()
        };
    }

    fn next_diagnostic(&mut self, forward: bool) {
        let mut diagnostics: Vec<&Diagnostic> = self
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.row < self.buffer.line_count())
            .collect();
        if diagnostics.is_empty() {
            self.message = "No LSP diagnostics".into();
            return;
        }
        diagnostics.sort_by_key(|diagnostic| (diagnostic.row, diagnostic.start));
        let point = (self.buffer.point.row, self.buffer.point.col);
        let selected = if forward {
            diagnostics
                .iter()
                .find(|diagnostic| (diagnostic.row, diagnostic.start) > point)
                .copied()
                .or_else(|| diagnostics.first().copied())
        } else {
            diagnostics
                .iter()
                .rev()
                .find(|diagnostic| (diagnostic.row, diagnostic.start) < point)
                .copied()
                .or_else(|| diagnostics.last().copied())
        };
        let Some(diagnostic) = selected else {
            return;
        };
        let row = diagnostic.row;
        let col = diagnostic.start.min(self.buffer.lines[row].len());
        self.buffer.set_point(Point::new(row, col));
        let severity = match diagnostic.severity {
            Severity::Error => "Error",
            Severity::Warning => "Warning",
            Severity::Information => "Info",
            Severity::Hint => "Hint",
        };
        self.message = format!("{severity}: {}", diagnostic.message);
    }

    fn indent(&mut self) {
        let row = self.buffer.point.row;
        let col = display_col(
            &self.buffer.lines[row],
            self.buffer.point.col,
            self.settings.tab_width,
        );
        let spaces = self.settings.tab_width - (col % self.settings.tab_width);
        let text = " ".repeat(spaces);
        self.buffer.insert_text(&text);
    }

    /// `auto-fill-mode`: break the current line at the last whitespace that
    /// fits within `fill_column` after a self-insert.
    fn auto_fill(&mut self) {
        let fill_column = self.settings.fill_column;
        if fill_column == 0 {
            return;
        }
        let row = self.buffer.point.row;
        let tab_width = self.settings.tab_width;
        let (point_col, break_col, indent_len) = {
            let line = &self.buffer.lines[row];
            let point_col = self.buffer.point.col.min(line.len());
            if display_col(line, point_col, tab_width) <= fill_column {
                return;
            }
            let indent_len = line
                .iter()
                .take_while(|c| **c == ' ' || **c == '\t')
                .count();
            let mut col = display_col(line, indent_len, tab_width);
            let mut break_col = None;
            for (idx, c) in line.iter().enumerate().take(point_col).skip(indent_len) {
                let c = *c;
                if (c == ' ' || c == '\t') && col <= fill_column {
                    break_col = Some(idx);
                }
                col += char_width(c, col, tab_width);
                if col > fill_column {
                    break;
                }
            }
            (point_col, break_col, indent_len)
        };
        let Some(break_col) = break_col else {
            return;
        };
        if break_col <= indent_len {
            return;
        }
        let new_col = indent_len + (point_col - break_col - 1);
        self.buffer.point = Point::new(row, break_col);
        self.buffer.delete_forward();
        self.buffer.newline(true);
        self.buffer.point = Point::new(row + 1, new_col);
    }

    fn unindent(&mut self) {
        let row = self.buffer.point.row;
        let width = self.settings.tab_width;
        let mut removed = 0;
        while removed < width && self.buffer.point.col > 0 {
            let col = self.buffer.point.col;
            if self.buffer.lines[row][col - 1] != ' ' {
                break;
            }
            self.buffer.delete_backward();
            removed += 1;
        }
    }

    fn save_buffer(&mut self) {
        if self.buffer.path.is_none() {
            self.prompt_write_file(false);
            return;
        }
        match self.buffer.save() {
            Ok(path) => {
                self.message = format!("Wrote {}", path.display());
                if self.mode == Lang::GitCommit
                    && self
                        .buffer
                        .is_effectively_empty(&self.settings.comment_start)
                {
                    self.message.push_str("   (warning: empty commit message)");
                }
            }
            Err(e) => self.message = format!("Save failed: {e}"),
        }
    }

    fn save_and_quit(&mut self) {
        if self.buffer.path.is_none() {
            self.prompt_write_file(true);
            return;
        }
        match self.buffer.save() {
            Ok(_) => {
                if self.mode == Lang::GitCommit
                    && self
                        .buffer
                        .is_effectively_empty(&self.settings.comment_start)
                {
                    self.confirm = Some(Confirm {
                        kind: ConfirmKind::EmptyCommit,
                        question: "Empty commit message; finish anyway? (y/n)".into(),
                    });
                } else {
                    self.finish_quit(0);
                }
            }
            Err(e) => self.message = format!("Save failed: {e}"),
        }
    }

    fn abort_commit(&mut self) {
        if self.buffer.has_unsaved_changes() {
            self.confirm = Some(Confirm {
                kind: ConfirmKind::AbortCommit,
                question: "Abort commit and discard changes? (y/n)".into(),
            });
        } else {
            self.finish_quit(1);
        }
    }

    fn quit_request(&mut self) {
        if !self.config.confirm_on_quit {
            self.finish_quit(0);
            return;
        }
        if let Some(index) = self.first_unsaved_other_tab() {
            self.switch_to_tab(index);
            self.message = "Other tabs have unsaved changes; save them first".into();
            return;
        }
        if !self.buffer.has_unsaved_changes() {
            self.finish_quit(0);
            return;
        }
        if self.buffer.path.is_none() {
            self.confirm = Some(Confirm {
                kind: ConfirmKind::QuitSaveUnnamed,
                question: "Save this buffer to a file? (y/n)".into(),
            });
        } else {
            self.confirm = Some(Confirm {
                kind: ConfirmKind::QuitSave,
                question: format!("Save changes to {}? (y/n)", self.buffer.display_name()),
            });
        }
    }

    fn handle_confirm(&mut self, key: Key) {
        let Some(confirm) = self.confirm.take() else {
            return;
        };
        let yes = matches!(key.code, KeyCode::Char('y' | 'Y') | KeyCode::Enter);
        let no = matches!(key.code, KeyCode::Char('n' | 'N'));
        let cancel = matches!(key.code, KeyCode::Esc)
            || (key.ctrl && matches!(key.code, KeyCode::Char('g')));
        if !yes && !no && !cancel {
            self.confirm = Some(confirm);
            return;
        }
        match confirm.kind {
            ConfirmKind::QuitSave => {
                if yes {
                    self.save_buffer();
                    if !self.buffer.modified {
                        self.finish_quit(0);
                    }
                } else if no {
                    self.finish_quit(0);
                }
            }
            ConfirmKind::QuitSaveUnnamed => {
                if yes {
                    self.prompt_write_file(true);
                } else if no {
                    self.finish_quit(0);
                }
            }
            ConfirmKind::AbortCommit => {
                if yes {
                    self.finish_quit(1);
                } else {
                    self.message = "Canceled".into();
                }
            }
            ConfirmKind::EmptyCommit => {
                if yes {
                    self.finish_quit(0);
                } else {
                    self.message = "Commit canceled".into();
                }
            }
        }
    }

    // -------------------------------------------------------------- search

    fn start_search(&mut self, dir: i32) {
        self.search = Some(SearchState {
            query: Vec::new(),
            origin: self.buffer.point,
            dir,
            current: None,
            wrapped: false,
        });
        self.message.clear();
    }

    fn is_search_key(&self, key: &Key) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                (!key.ctrl && !key.alt)
                    || (key.ctrl && matches!(c.to_ascii_lowercase(), 's' | 'r' | 'g'))
            }
            KeyCode::Backspace | KeyCode::Enter | KeyCode::Esc => true,
            _ => false,
        }
    }

    fn handle_search(&mut self, key: Key) {
        let Some(mut search) = self.search.take() else {
            return;
        };
        match key.code {
            KeyCode::Char(c) if key.ctrl => match c.to_ascii_lowercase() {
                's' => {
                    self.search = Some(search);
                    self.search_step(1);
                }
                'r' => {
                    self.search = Some(search);
                    self.search_step(-1);
                }
                'g' => {
                    self.search = Some(search);
                    self.abort_search();
                }
                _ => self.search = Some(search),
            },
            KeyCode::Char(c) if !key.alt => {
                search.query.push(c);
                self.search = Some(search);
                self.update_search();
            }
            KeyCode::Backspace => {
                search.query.pop();
                self.search = Some(search);
                self.update_search();
            }
            KeyCode::Enter => {
                self.search = Some(search);
                self.finish_search(true);
            }
            KeyCode::Esc => {
                self.search = Some(search);
                self.abort_search();
            }
            _ => self.search = Some(search),
        }
    }

    fn update_search(&mut self) {
        let Some(mut search) = self.search.take() else {
            return;
        };
        if search.query.is_empty() {
            self.buffer.set_point(search.origin);
            search.current = None;
            search.wrapped = false;
            self.search = Some(search);
            return;
        }
        let fold = !search.query.iter().any(|c| c.is_uppercase());
        let lines = &self.buffer.lines;
        let found = if search.dir > 0 {
            find_forward(lines, &search.query, search.origin, fold)
        } else {
            find_backward(lines, &search.query, search.origin, fold)
        };
        let (found, wrapped) = match found {
            Some(p) => (Some(p), false),
            None => {
                let p = if search.dir > 0 {
                    find_forward(lines, &search.query, Point::new(0, 0), fold)
                } else {
                    find_last(lines, &search.query, fold)
                };
                (p, true)
            }
        };
        if let Some(p) = found {
            self.buffer.set_point(p);
            search.current = Some(p);
        } else {
            search.current = None;
        }
        search.wrapped = wrapped && found.is_some();
        self.search = Some(search);
    }

    fn search_step(&mut self, dir: i32) {
        let Some(mut search) = self.search.take() else {
            return;
        };
        if search.query.is_empty() {
            self.search = Some(search);
            return;
        }
        search.dir = dir;
        let fold = !search.query.iter().any(|c| c.is_uppercase());
        let start = match (search.current, dir) {
            (Some(cur), 1) => self.buffer.next_pos(cur).unwrap_or(Point::new(0, 0)),
            (Some(cur), _) => cur,
            (None, _) => search.origin,
        };
        let lines = &self.buffer.lines;
        let found = if dir > 0 {
            find_forward(lines, &search.query, start, fold)
        } else {
            find_backward(lines, &search.query, start, fold)
        };
        let (found, wrapped) = match found {
            Some(p) => (Some(p), false),
            None => {
                let p = if dir > 0 {
                    find_forward(lines, &search.query, Point::new(0, 0), fold)
                } else {
                    find_last(lines, &search.query, fold)
                };
                (p, true)
            }
        };
        if let Some(p) = found {
            self.buffer.set_point(p);
            search.current = Some(p);
        }
        search.wrapped = wrapped && found.is_some();
        self.search = Some(search);
    }

    fn finish_search(&mut self, keep: bool) {
        let Some(search) = self.search.take() else {
            return;
        };
        if keep {
            if search.current.is_some() && self.buffer.point != search.origin {
                self.buffer.mark = Some(search.origin);
            }
        } else {
            self.buffer.set_point(search.origin);
        }
    }

    fn abort_search(&mut self) {
        let Some(search) = self.search.take() else {
            return;
        };
        self.buffer.set_point(search.origin);
        self.message = "Quit".into();
    }

    // ---------------------------------------------------------- minibuffer

    fn handle_minibuffer(&mut self, key: Key) {
        if self.minibuffer.is_none() {
            return;
        }
        let completion_active = self.minibuffer.as_ref().is_some_and(|mb| {
            mb.completion
                .as_ref()
                .is_some_and(|completion| !completion.candidates.is_empty())
        });
        let completion_key = matches!(key.code, KeyCode::Tab | KeyCode::BackTab)
            || (completion_active
                && (matches!(key.code, KeyCode::Up | KeyCode::Down)
                    || (key.ctrl && matches!(key.code, KeyCode::Char('n' | 'p')))));
        if !completion_key {
            if let Some(mb) = self.minibuffer.as_mut() {
                mb.reset_completion();
            }
        }
        let mut history_prev = false;
        let mut history_next = false;
        let mut completion_prev = false;
        let mut completion_next = false;
        let mut edited = false;
        let mut complete = None;
        let mut accept = false;
        let mut cancel = false;
        {
            let Some(mb) = self.minibuffer.as_mut() else {
                return;
            };
            match key.code {
                KeyCode::Char(c) if key.ctrl => match c.to_ascii_lowercase() {
                    'a' => mb.home(),
                    'e' => mb.end(),
                    'b' => mb.left(),
                    'f' => mb.right(),
                    'd' => {
                        mb.delete_forward();
                        edited = true;
                    }
                    'k' => {
                        mb.kill_to_end();
                        edited = true;
                    }
                    'w' => {
                        mb.delete_word_backward();
                        edited = true;
                    }
                    'g' => cancel = true,
                    'p' => {
                        if completion_active {
                            completion_prev = true;
                        } else {
                            history_prev = true;
                        }
                    }
                    'n' => {
                        if completion_active {
                            completion_next = true;
                        } else {
                            history_next = true;
                        }
                    }
                    _ => {}
                },
                KeyCode::Char('n') if key.alt => history_next = true,
                KeyCode::Char('p') if key.alt => history_prev = true,
                KeyCode::Char(c) if !key.alt => {
                    mb.insert(c);
                    edited = true;
                }
                KeyCode::Backspace => {
                    mb.backspace();
                    edited = true;
                }
                KeyCode::Delete => {
                    mb.delete_forward();
                    edited = true;
                }
                KeyCode::Left => mb.left(),
                KeyCode::Right => mb.right(),
                KeyCode::Home => mb.home(),
                KeyCode::End => mb.end(),
                KeyCode::Tab => complete = Some(true),
                KeyCode::BackTab => complete = Some(false),
                KeyCode::Enter => accept = true,
                KeyCode::Esc => cancel = true,
                KeyCode::Up => {
                    if completion_active {
                        completion_prev = true;
                    } else {
                        history_prev = true;
                    }
                }
                KeyCode::Down => {
                    if completion_active {
                        completion_next = true;
                    } else {
                        history_next = true;
                    }
                }
                _ => {}
            }
        }
        if completion_prev && !self.select_completion(false) {
            if let Some(mb) = self.minibuffer.as_mut() {
                mb.history_prev(&self.history);
            }
        }
        if completion_next && !self.select_completion(true) {
            if let Some(mb) = self.minibuffer.as_mut() {
                mb.history_next(&self.history);
            }
        }
        if history_prev {
            if let Some(mb) = self.minibuffer.as_mut() {
                mb.history_prev(&self.history);
            }
        }
        if history_next {
            if let Some(mb) = self.minibuffer.as_mut() {
                mb.history_next(&self.history);
            }
        }
        if edited {
            self.prime_completion();
        }
        if let Some(forward) = complete {
            self.complete(forward);
        }
        if accept {
            self.accept_minibuffer();
        }
        if cancel {
            self.minibuffer = None;
            self.message = "Quit".into();
        }
    }

    fn start_mx(&mut self) {
        self.minibuffer = Some(Minibuffer::new("M-x ", PromptKind::Mx, ""));
        self.prime_completion();
    }

    fn prompt_find_file(&mut self) {
        let mut initial = std::env::current_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| ".".to_string());
        if !initial.ends_with('/') {
            initial.push('/');
        }
        self.minibuffer = Some(Minibuffer::new(
            "Find file: ",
            PromptKind::FindFile,
            &initial,
        ));
        self.prime_completion();
    }

    fn prompt_write_file(&mut self, then_quit: bool) {
        let initial = match &self.buffer.path {
            Some(path) => path.to_string_lossy().into_owned(),
            None => {
                let mut dir = std::env::current_dir()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(|_| ".".to_string());
                if !dir.ends_with('/') {
                    dir.push('/');
                }
                dir
            }
        };
        self.minibuffer = Some(Minibuffer::new(
            "Write file: ",
            PromptKind::WriteFile { then_quit },
            &initial,
        ));
        self.prime_completion();
    }

    fn prompt_goto_line(&mut self) {
        self.minibuffer = Some(Minibuffer::new("Goto line: ", PromptKind::GotoLine, ""));
    }

    fn push_history(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        if self.history.last().map(|s| s == text).unwrap_or(false) {
            return;
        }
        self.history.push(text.to_string());
        if self.history.len() > 100 {
            self.history.remove(0);
        }
    }

    fn accept_minibuffer(&mut self) {
        let Some(mb) = self.minibuffer.take() else {
            return;
        };
        let text = mb.text();
        let trimmed = text.trim().to_string();
        match mb.kind {
            PromptKind::Mx => {
                if trimmed.is_empty() {
                    return;
                }
                self.push_history(&trimmed);
                self.run_mx(&trimmed);
            }
            PromptKind::GotoLine => match trimmed.parse::<usize>() {
                Ok(n) if n >= 1 => {
                    self.buffer.move_to_line(n);
                    self.message = format!("Line {n}");
                }
                _ => self.message = format!("Invalid line number: {trimmed}"),
            },
            PromptKind::FindFile => {
                if trimmed.is_empty() {
                    return;
                }
                self.push_history(&trimmed);
                let path = self.resolve_path(&trimmed);
                self.open_file(path);
            }
            PromptKind::WriteFile { then_quit } => {
                if trimmed.is_empty() {
                    return;
                }
                self.push_history(&trimmed);
                let path = self.resolve_path(&trimmed);
                match self.buffer.save_as(&path) {
                    Ok(()) => {
                        self.refresh_mode();
                        self.message = format!("Wrote {}", path.display());
                        if then_quit {
                            self.finish_quit(0);
                        }
                    }
                    Err(e) => self.message = format!("Write failed: {e}"),
                }
            }
            PromptKind::SpellReplace { start, end } => {
                if trimmed.is_empty() {
                    return;
                }
                self.buffer.replace_range(start, end, &trimmed);
                self.message = format!("Replaced with {trimmed}");
            }
        }
    }

    fn complete(&mut self, forward: bool) {
        let Some(kind) = self.minibuffer.as_ref().map(|m| m.kind.clone()) else {
            return;
        };
        let directory_input = matches!(kind, PromptKind::FindFile | PromptKind::WriteFile { .. })
            && self
                .minibuffer
                .as_ref()
                .is_some_and(|mb| mb.text().ends_with('/'));
        let listing = self
            .minibuffer
            .as_ref()
            .and_then(|mb| mb.completion.as_ref())
            .is_some_and(|completion| completion.directory_listing);
        if (!directory_input || listing) && self.cycle_completion(forward) {
            return;
        }
        match kind {
            PromptKind::Mx => self.complete_command(true),
            PromptKind::FindFile | PromptKind::WriteFile { .. } => self.complete_path(true),
            PromptKind::GotoLine | PromptKind::SpellReplace { .. } => {}
        }
    }

    /// Fill the candidate menu without changing the typed input.  Used when a
    /// prompt opens and after each edit, so the list is visible immediately.
    fn prime_completion(&mut self) {
        let Some(kind) = self.minibuffer.as_ref().map(|m| m.kind.clone()) else {
            return;
        };
        match kind {
            PromptKind::Mx => self.complete_command(false),
            PromptKind::FindFile | PromptKind::WriteFile { .. } => self.complete_path(false),
            PromptKind::GotoLine | PromptKind::SpellReplace { .. } => {}
        }
    }

    /// Handle repeated Tab presses: keep the original candidate set even when
    /// the inserted candidate no longer matches the shorter typed prefix.
    fn cycle_completion(&mut self, forward: bool) -> bool {
        let Some(mb) = self.minibuffer.as_mut() else {
            return false;
        };
        let Some(mut state) = mb.completion.take() else {
            return false;
        };
        let len = state.candidates.len();
        if len == 0 {
            return false;
        }
        let current = mb.text();
        let selected = match state.index {
            Some(index) if index < len && current == state.candidates[index] => {
                if forward {
                    (index + 1) % len
                } else {
                    (index + len - 1) % len
                }
            }
            None => match state.candidates.iter().position(|c| *c == current) {
                // The shared prefix is already a complete candidate; show
                // that candidate now and start cycling from it next time.
                Some(index) => index,
                None => {
                    if forward {
                        0
                    } else {
                        len - 1
                    }
                }
            },
            _ => {
                // Stale cycle (the text was changed outside the completion
                // UI); fall back to a fresh completion pass.
                return false;
            }
        };
        let candidate = state.candidates[selected].clone();
        mb.set_text(&candidate);
        state.index = Some(selected);
        state.directory_listing = false;
        let hint = format!(
            "[{}/{}] {}",
            selected + 1,
            len,
            completion_preview(&state.candidates)
        );
        mb.completion = Some(state);
        mb.hint = hint;
        true
    }

    /// Ctrl-n / Ctrl-p selection.  If the current text is already a candidate
    /// but no row is selected yet, move past it instead of no-op selecting it.
    fn select_completion(&mut self, forward: bool) -> bool {
        let Some(mb) = self.minibuffer.as_mut() else {
            return false;
        };
        let current = mb.text();
        let Some(mut state) = mb.completion.take() else {
            return false;
        };
        let len = state.candidates.len();
        if len == 0 {
            return false;
        }
        if state.index.is_none() {
            if let Some(index) = state.candidates.iter().position(|c| *c == current) {
                let selected = if forward {
                    (index + 1) % len
                } else {
                    (index + len - 1) % len
                };
                let candidate = state.candidates[selected].clone();
                mb.set_text(&candidate);
                state.index = Some(selected);
                state.directory_listing = false;
                let hint = format!(
                    "[{}/{}] {}",
                    selected + 1,
                    len,
                    completion_preview(&state.candidates)
                );
                mb.completion = Some(state);
                mb.hint = hint;
                return true;
            }
        }
        mb.completion = Some(state);
        self.cycle_completion(forward)
    }

    /// Insert the shared prefix of `candidates`.  `candidates` are the exact
    /// strings to put in the minibuffer (file names, complete paths, or
    /// command names).
    fn offer_completions(
        &mut self,
        candidates: Vec<String>,
        noun: &str,
        directory_listing: bool,
        advance: bool,
    ) {
        let Some(mb) = self.minibuffer.as_mut() else {
            return;
        };
        if candidates.is_empty() {
            mb.reset_completion();
            mb.hint = "No match".into();
            return;
        }
        if advance && candidates.len() == 1 {
            mb.set_text(&candidates[0]);
            mb.completion = None;
            mb.hint.clear();
            return;
        }
        if advance {
            let current = mb.text();
            let prefix = common_prefix(&candidates);
            if prefix.chars().count() > current.chars().count() {
                mb.set_text(&prefix);
            }
        }
        mb.completion = Some(CompletionState {
            candidates: candidates.clone(),
            index: None,
            directory_listing,
        });
        let label = if candidates.len() == 1 {
            noun.trim_end_matches('s')
        } else {
            noun
        };
        mb.hint = format!(
            "{} {}: {}",
            candidates.len(),
            label,
            completion_preview(&candidates)
        );
    }

    fn complete_command(&mut self, advance: bool) {
        let Some(mb) = self.minibuffer.as_ref() else {
            return;
        };
        let text = mb.text();
        let candidates: Vec<String> = COMMANDS
            .iter()
            .map(|(name, _)| (*name).to_string())
            .filter(|name| name.starts_with(&text))
            .collect();
        self.offer_completions(candidates, "commands", false, advance);
    }

    fn complete_path(&mut self, advance: bool) {
        let base_dir = self.buffer.dir();
        let raw = {
            let Some(mb) = self.minibuffer.as_mut() else {
                return;
            };
            let raw = mb.text();
            // A directory candidate ends in `/`; the next Tab should list
            // its contents rather than cycle through the parent's siblings.
            if raw.ends_with('/') {
                mb.reset_completion();
            }
            raw
        };
        let listing = raw.ends_with('/');
        let (dir_part, name_part) = match raw.rfind('/') {
            Some(i) => (&raw[..=i], &raw[i + 1..]),
            None => ("", raw.as_str()),
        };
        let dir = if dir_part.is_empty() {
            base_dir
        } else {
            let expanded = expand_tilde(dir_part);
            if expanded.is_absolute() {
                expanded
            } else {
                base_dir.join(expanded)
            }
        };

        let mut entries: Vec<(String, bool)> = Vec::new();
        let Ok(read_dir) = fs::read_dir(&dir) else {
            if let Some(mb) = self.minibuffer.as_mut() {
                mb.reset_completion();
                mb.hint = format!("No match in {}", dir.display());
            }
            return;
        };
        for entry in read_dir.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with(name_part) {
                continue;
            }
            if name.starts_with('.') && !name_part.starts_with('.') {
                continue;
            }
            entries.push((name, entry.path().is_dir()));
        }
        if name_part.starts_with('.') {
            if ".".starts_with(name_part) {
                entries.push((".".to_string(), true));
            }
            if "..".starts_with(name_part) {
                entries.push(("..".to_string(), true));
            }
        }
        entries.sort();
        let candidates: Vec<String> = entries
            .into_iter()
            .map(|(name, is_dir)| format!("{dir_part}{name}{}", if is_dir { "/" } else { "" }))
            .collect();
        self.offer_completions(candidates, "candidates", listing, advance);
    }

    fn resolve_path(&self, input: &str) -> PathBuf {
        let path = expand_tilde(input);
        if path.is_absolute() {
            path
        } else {
            self.buffer.dir().join(path)
        }
    }

    fn open_file(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.message = format!("{} is a directory", path.display());
            return;
        }
        match Buffer::open(&path) {
            Ok(buffer) => self.open_in_tab(buffer, false),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let buffer = Buffer::from_text("", Some(path.clone()));
                self.open_in_tab(buffer, true);
            }
            Err(e) => self.message = format!("Cannot open {}: {e}", path.display()),
        }
    }

    fn run_mx(&mut self, name: &str) {
        let exact = COMMANDS.iter().find(|(n, _)| *n == name).map(|(_, c)| *c);
        let command = match exact {
            Some(c) => Some(c),
            None => {
                let matches: Vec<Cmd> = COMMANDS
                    .iter()
                    .filter(|(n, _)| n.starts_with(name))
                    .map(|(_, c)| *c)
                    .collect();
                if matches.len() == 1 {
                    Some(matches[0])
                } else {
                    None
                }
            }
        };
        match command {
            Some(command) => self.run_command(command),
            None => self.message = format!("No matching command: {name}"),
        }
    }

    fn run_command(&mut self, command: Cmd) {
        match command {
            Cmd::FindFile => self.prompt_find_file(),
            Cmd::Save => self.save_buffer(),
            Cmd::WriteFile => self.prompt_write_file(false),
            Cmd::Quit => self.quit_request(),
            Cmd::Undo => self.undo(),
            Cmd::Redo => {
                if self.buffer.redo() {
                    self.message = "Redo".into();
                } else {
                    self.message = "Nothing to redo".into();
                }
            }
            Cmd::GotoLine => self.prompt_goto_line(),
            Cmd::ToggleLineNumbers => {
                self.settings.line_numbers = !self.settings.line_numbers;
                self.view.line_numbers = self.settings.line_numbers;
                self.message = format!(
                    "Line numbers {}",
                    if self.settings.line_numbers {
                        "on"
                    } else {
                        "off"
                    }
                );
            }
            Cmd::ToggleTrailingWs => {
                self.settings.highlight_trailing_ws = !self.settings.highlight_trailing_ws;
                self.view.highlight_trailing_ws = self.settings.highlight_trailing_ws;
                self.message = format!(
                    "Trailing whitespace highlight {}",
                    if self.settings.highlight_trailing_ws {
                        "on"
                    } else {
                        "off"
                    }
                );
            }
            Cmd::ToggleAutoFill => {
                self.settings.auto_fill = !self.settings.auto_fill;
                self.message = format!(
                    "Auto-fill mode {}",
                    if self.settings.auto_fill { "on" } else { "off" }
                );
            }
            Cmd::ToggleTruncateLines => {
                self.settings.truncate_lines = !self.settings.truncate_lines;
                self.view.truncate_lines = self.settings.truncate_lines;
                self.view.left = 0;
                self.view.top_offset = 0;
                self.needs_redraw = true;
                self.message = format!(
                    "Truncate lines {}",
                    if self.settings.truncate_lines {
                        "on"
                    } else {
                        "off (wrapping)"
                    }
                );
            }
            Cmd::ToggleEditorCompletion => {
                self.settings.editor_completion = !self.settings.editor_completion;
                if !self.settings.editor_completion {
                    self.editor_completion = None;
                }
                self.message = format!(
                    "Editor completion {}",
                    if self.settings.editor_completion {
                        "on"
                    } else {
                        "off"
                    }
                );
            }
            Cmd::CompleteSymbol => self.update_editor_completion(true),
            Cmd::ToggleSpellCheck => self.toggle_spell_check(),
            Cmd::CommentDwm => self.comment_dwim(),
            Cmd::NextError => self.next_diagnostic(true),
            Cmd::PreviousError => self.next_diagnostic(false),
            Cmd::TabNext => self.switch_tab(1),
            Cmd::TabPrevious => self.switch_tab(-1),
            Cmd::TabNew => self.new_tab(),
            Cmd::TabClose => self.close_tab(),
            Cmd::SpellWord => self.spell_word(),
            Cmd::NextSpellingError => self.next_spelling_error(true),
            Cmd::PreviousSpellingError => self.next_spelling_error(false),
            Cmd::SpellCheckBuffer => {
                self.spell_check_buffer();
            }
            Cmd::SetMark => {
                self.buffer.set_mark();
                self.message = "Mark set".into();
            }
            Cmd::KillRegion => self.kill_region(),
            Cmd::CopyRegion => self.copy_region(),
            Cmd::DeleteRegion => {
                if !self.delete_active_region() {
                    self.message = "No active region".into();
                }
            }
            Cmd::ExchangePointAndMark => {
                if !self.buffer.exchange_point_and_mark() {
                    self.message = "No mark set".into();
                }
            }
            Cmd::MarkWholeBuffer => {
                self.buffer.mark_whole_buffer();
                self.message = "Mark set".into();
            }
            Cmd::DeactivateMark => {
                self.buffer.deactivate_mark();
                self.message = "Mark deactivated".into();
            }
            Cmd::Revert => self.revert_buffer(),
            Cmd::DeleteTrailingWs => {
                self.buffer.begin_command();
                let removed = self.buffer.delete_trailing_whitespace();
                self.buffer.end_command(None);
                self.message = if removed == 0 {
                    "No trailing whitespace".into()
                } else {
                    format!("Deleted {removed} whitespace characters")
                };
            }
            Cmd::Help => self.open_help(),
            Cmd::SearchForward => self.start_search(1),
            Cmd::SearchBackward => self.start_search(-1),
            Cmd::Beginning => self.buffer.move_buffer_start(),
            Cmd::End => self.buffer.move_buffer_end(),
        }
    }

    fn refresh_mode(&mut self) {
        let first_line: String = self
            .buffer
            .lines
            .first()
            .map(|l| l.iter().copied().collect())
            .unwrap_or_default();
        let mode = Lang::detect(self.buffer.path.as_deref(), Some(&first_line));
        if mode != self.mode {
            self.mode = mode;
            self.settings = self.config.settings_for(mode);
            self.highlighter = Highlighter::new(mode, self.buffer.version);
            self.apply_settings();
            self.spell.clear();
            self.configure_spell();
            self.word_cache = None;
            self.editor_completion = None;
        }
        self.start_lsp();
    }

    fn revert_buffer(&mut self) {
        if self.buffer.has_unsaved_changes() {
            self.message = "Buffer modified: not reverting".into();
            return;
        }
        let Some(path) = self.buffer.path.clone() else {
            self.message = "Buffer has no file".into();
            return;
        };
        match Buffer::open(&path) {
            Ok(buffer) => {
                self.set_buffer(buffer);
                self.message = format!("Reverted {}", path.display());
            }
            Err(e) => self.message = format!("Revert failed: {e}"),
        }
    }

    fn open_help(&mut self) {
        self.overlay = Some(Overlay { scroll: 0 });
    }

    fn handle_overlay(&mut self, key: Key) {
        let body = self.view.body_height();
        let max = help::HELP.len().saturating_sub(body);
        let Some(mut overlay) = self.overlay else {
            return;
        };
        let mut close = false;
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::F(1) => close = true,
            KeyCode::Char('g') if key.ctrl => close = true,
            KeyCode::Up => overlay.scroll = overlay.scroll.saturating_sub(1),
            KeyCode::Down => overlay.scroll = (overlay.scroll + 1).min(max),
            KeyCode::Char('p') if key.ctrl => overlay.scroll = overlay.scroll.saturating_sub(1),
            KeyCode::Char('n') if key.ctrl => overlay.scroll = (overlay.scroll + 1).min(max),
            KeyCode::PageUp => overlay.scroll = overlay.scroll.saturating_sub(body),
            KeyCode::PageDown => overlay.scroll = (overlay.scroll + body).min(max),
            KeyCode::Char('v') if key.ctrl => overlay.scroll = (overlay.scroll + body).min(max),
            KeyCode::Char('v') if key.alt => overlay.scroll = overlay.scroll.saturating_sub(body),
            KeyCode::Char(' ') => overlay.scroll = (overlay.scroll + body).min(max),
            KeyCode::Home => overlay.scroll = 0,
            KeyCode::End => overlay.scroll = max,
            _ => {}
        }
        if close {
            self.overlay = None;
            self.message.clear();
        } else {
            self.overlay = Some(overlay);
        }
    }

    pub fn prompt_for_quit(&self) -> Option<&str> {
        self.confirm.as_ref().map(|c| c.question.as_str())
    }
}

fn expand_tilde(input: &str) -> PathBuf {
    let home = std::env::var("HOME").ok();
    if input == "~" {
        if let Some(home) = home {
            return PathBuf::from(home);
        }
    }
    if let Some(rest) = input.strip_prefix("~/") {
        if let Some(home) = home {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(input)
}

fn common_prefix(items: &[String]) -> String {
    let Some(first) = items.first() else {
        return String::new();
    };
    let mut prefix: Vec<char> = first.chars().collect();
    for item in &items[1..] {
        let chars: Vec<char> = item.chars().collect();
        let mut n = 0;
        while n < prefix.len() && n < chars.len() && prefix[n] == chars[n] {
            n += 1;
        }
        prefix.truncate(n);
        if prefix.is_empty() {
            break;
        }
    }
    prefix.into_iter().collect()
}

fn completion_preview(candidates: &[String]) -> String {
    let mut preview = candidates
        .iter()
        .take(5)
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    if candidates.len() > 5 {
        preview.push_str(" ...");
    }
    preview
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    fn editor(text: &str) -> Editor {
        Editor::from_text(text, None, Config::default())
    }

    fn temp_dir() -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("emacette-completion-{}-{}", std::process::id(), n));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn editor_in(dir: &Path, text: &str) -> Editor {
        Editor::from_text(text, Some(dir.join("buffer.txt")), Config::default())
    }

    #[test]
    fn m_x_opens_with_completion_candidates() {
        let mut ed = editor("");
        ed.start_mx();
        let mb = ed.minibuffer.as_ref().unwrap();
        assert_eq!(mb.text(), "");
        let candidates = &mb.completion.as_ref().unwrap().candidates;
        assert!(candidates.iter().any(|name| name == "save-buffer"));
        assert!(mb.hint.contains("commands"));
    }

    #[test]
    fn m_x_filters_candidates_while_typing() {
        let mut ed = editor("");
        ed.start_mx();
        for c in "save-bu".chars() {
            ed.handle_key(Key::ch(c));
        }
        let mb = ed.minibuffer.as_ref().unwrap();
        let candidates = &mb.completion.as_ref().unwrap().candidates;
        assert_eq!(candidates.len(), 2);
        assert!(candidates.iter().all(|name| name.starts_with("save-bu")));
    }

    #[test]
    fn find_file_opens_with_a_directory_listing() {
        let dir = temp_dir();
        fs::write(dir.join("alpha.txt"), "a").unwrap();
        fs::write(dir.join("alphabet.txt"), "b").unwrap();
        let mut ed = editor_in(&dir, "");
        ed.prompt_find_file();
        ed.minibuffer
            .as_mut()
            .unwrap()
            .set_text(&format!("{}/", dir.display()));
        ed.prime_completion();
        let mb = ed.minibuffer.as_ref().unwrap();
        assert_eq!(mb.text(), format!("{}/", dir.display()));
        let candidates = &mb.completion.as_ref().unwrap().candidates;
        assert!(candidates.iter().any(|name| name.ends_with("alpha.txt")));
        assert!(mb.completion.as_ref().unwrap().directory_listing);

        ed.handle_key(Key::code(KeyCode::Tab));
        assert!(ed
            .minibuffer_text()
            .as_deref()
            .is_some_and(|text| text.ends_with("alpha.txt")));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_file_does_not_insert_a_unique_candidate_immediately() {
        let dir = temp_dir();
        fs::write(dir.join("only.txt"), "a").unwrap();
        let mut ed = editor_in(&dir, "");
        ed.prompt_find_file();
        ed.minibuffer
            .as_mut()
            .unwrap()
            .set_text(&format!("{}/", dir.display()));
        ed.prime_completion();
        assert_eq!(
            ed.minibuffer_text().as_deref(),
            Some(format!("{}/", dir.display()).as_str())
        );
        assert_eq!(
            ed.minibuffer
                .as_ref()
                .unwrap()
                .completion
                .as_ref()
                .unwrap()
                .candidates
                .len(),
            1
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn command_completion_extends_and_cycles() {
        let mut ed = editor("");
        ed.start_mx();
        ed.minibuffer.as_mut().unwrap().set_text("save-bu");
        ed.complete(true);
        assert_eq!(ed.minibuffer_text().as_deref(), Some("save-buffer"));
        assert!(ed.minibuffer.as_ref().unwrap().hint.contains("2 commands"));

        ed.complete(true);
        assert_eq!(ed.minibuffer_text().as_deref(), Some("save-buffer"));
        ed.complete(true);
        assert_eq!(
            ed.minibuffer_text().as_deref(),
            Some("save-buffers-kill-terminal")
        );
        ed.complete(false);
        assert_eq!(ed.minibuffer_text().as_deref(), Some("save-buffer"));
    }

    #[test]
    fn path_completion_resolves_against_the_buffer_directory() {
        let dir = temp_dir();
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("sub/inside.txt"), "inside").unwrap();
        let mut ed = editor_in(&dir, "");

        ed.prompt_find_file();
        ed.minibuffer.as_mut().unwrap().set_text("sub/in");
        ed.complete(true);
        assert_eq!(
            ed.minibuffer_text().as_deref(),
            Some("sub/inside.txt"),
            "relative paths must resolve from the buffer's directory"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn path_completion_completes_directories_with_a_slash() {
        let dir = temp_dir();
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("sub/inside.txt"), "inside").unwrap();
        let mut ed = editor_in(&dir, "");

        ed.prompt_find_file();
        ed.minibuffer.as_mut().unwrap().set_text("su");
        ed.complete(true);
        assert_eq!(ed.minibuffer_text().as_deref(), Some("sub/"));

        ed.complete(true);
        assert_eq!(ed.minibuffer_text().as_deref(), Some("sub/inside.txt"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn cycling_into_a_directory_lists_its_contents() {
        let dir = temp_dir();
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("sub/inside.txt"), "inside").unwrap();
        fs::write(dir.join("sun.txt"), "sun").unwrap();
        let mut ed = editor_in(&dir, "");

        ed.prompt_find_file();
        ed.minibuffer.as_mut().unwrap().set_text("su");
        ed.complete(true);
        assert_eq!(ed.minibuffer_text().as_deref(), Some("su"));
        ed.complete(true);
        assert_eq!(ed.minibuffer_text().as_deref(), Some("sub/"));
        ed.complete(true);
        assert_eq!(ed.minibuffer_text().as_deref(), Some("sub/inside.txt"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn path_completion_cycles_candidates() {
        let dir = temp_dir();
        fs::write(dir.join("alpha.txt"), "a").unwrap();
        fs::write(dir.join("alphabet.txt"), "b").unwrap();
        let mut ed = editor_in(&dir, "");

        ed.prompt_find_file();
        ed.minibuffer.as_mut().unwrap().set_text("al");
        ed.complete(true);
        assert_eq!(ed.minibuffer_text().as_deref(), Some("alpha"));
        ed.complete(true);
        assert_eq!(ed.minibuffer_text().as_deref(), Some("alpha.txt"));
        ed.complete(true);
        assert_eq!(ed.minibuffer_text().as_deref(), Some("alphabet.txt"));
        ed.complete(false);
        assert_eq!(ed.minibuffer_text().as_deref(), Some("alpha.txt"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn minibuffer_ctrl_np_selects_completion_candidates() {
        let dir = temp_dir();
        fs::write(dir.join("alpha.txt"), "a").unwrap();
        fs::write(dir.join("alphabet.txt"), "b").unwrap();
        let mut ed = editor_in(&dir, "");

        ed.prompt_find_file();
        ed.minibuffer.as_mut().unwrap().set_text("al");
        ed.complete(true);
        assert_eq!(ed.minibuffer_text().as_deref(), Some("alpha"));

        ed.handle_key(Key::ctrl('n'));
        assert_eq!(ed.minibuffer_text().as_deref(), Some("alpha.txt"));
        ed.handle_key(Key::ctrl('n'));
        assert_eq!(ed.minibuffer_text().as_deref(), Some("alphabet.txt"));
        ed.handle_key(Key::ctrl('p'));
        assert_eq!(ed.minibuffer_text().as_deref(), Some("alpha.txt"));
        ed.handle_key(Key::code(KeyCode::Tab));
        assert_eq!(ed.minibuffer_text().as_deref(), Some("alphabet.txt"));
        ed.handle_key(Key::code(KeyCode::Up));
        assert_eq!(ed.minibuffer_text().as_deref(), Some("alpha.txt"));
        ed.handle_key(Key::code(KeyCode::Down));
        assert_eq!(ed.minibuffer_text().as_deref(), Some("alphabet.txt"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn minibuffer_ctrl_n_advances_past_an_exact_candidate() {
        let mut ed = editor("");
        ed.start_mx();
        ed.minibuffer.as_mut().unwrap().set_text("save-buffer");
        ed.complete(true);
        assert_eq!(ed.minibuffer_text().as_deref(), Some("save-buffer"));
        ed.handle_key(Key::ctrl('n'));
        assert_eq!(
            ed.minibuffer_text().as_deref(),
            Some("save-buffers-kill-terminal")
        );
        ed.handle_key(Key::ctrl('p'));
        assert_eq!(ed.minibuffer_text().as_deref(), Some("save-buffer"));
    }

    #[test]
    fn minibuffer_meta_np_uses_history() {
        let mut ed = editor("");
        ed.history = vec!["one".to_string(), "two".to_string()];
        ed.start_mx();
        ed.handle_key(Key::alt('p'));
        assert_eq!(ed.minibuffer_text().as_deref(), Some("two"));
        ed.handle_key(Key::alt('p'));
        assert_eq!(ed.minibuffer_text().as_deref(), Some("one"));
        ed.handle_key(Key::alt('n'));
        assert_eq!(ed.minibuffer_text().as_deref(), Some("two"));
    }

    #[test]
    fn minibuffer_ctrl_n_selects_from_the_primed_menu() {
        let mut ed = editor("");
        ed.start_mx();
        ed.handle_key(Key::ctrl('n'));
        let text = ed.minibuffer_text().unwrap();
        assert!(text == "find-file" || text == "save-buffer");
        assert!(ed.minibuffer.as_ref().unwrap().hint.contains("[1/"));
    }

    #[test]
    fn editing_clears_a_completion_cycle() {
        let mut ed = editor("");
        ed.start_mx();
        ed.minibuffer.as_mut().unwrap().set_text("save-bu");
        ed.complete(true);
        assert!(ed.minibuffer.as_ref().unwrap().completion.is_some());
        ed.handle_key(Key::ch('x'));
        let mb = ed.minibuffer.as_ref().unwrap();
        assert!(mb.completion.is_none());
        assert_eq!(mb.hint, "No match");
    }

    #[test]
    fn editor_completion_accepts_with_tab() {
        let config = Config {
            editor_completion: true,
            ..Config::default()
        };
        let mut ed = Editor::from_text("alpha alphabet ", None, config);
        ed.buffer.set_point(Point::new(0, 15));
        for c in "al".chars() {
            ed.handle_key(Key::ch(c));
        }
        assert!(ed.editor_completion.is_some());
        ed.handle_key(Key::code(KeyCode::Tab));
        assert_eq!(ed.buffer_text(), "alpha alphabet alpha");
        assert!(ed.editor_completion.is_none());
    }

    #[test]
    fn editor_completion_cycles_with_down() {
        let config = Config {
            editor_completion: true,
            ..Config::default()
        };
        let mut ed = Editor::from_text("alpha alphabet ", None, config);
        ed.buffer.set_point(Point::new(0, 15));
        for c in "al".chars() {
            ed.handle_key(Key::ch(c));
        }
        ed.handle_key(Key::code(KeyCode::Down));
        ed.handle_key(Key::code(KeyCode::Tab));
        assert_eq!(ed.buffer_text(), "alpha alphabet alphabet");
    }

    #[test]
    fn complete_symbol_works_when_auto_completion_is_off() {
        let mut ed = editor("alpha alphabet al");
        ed.buffer.set_point(Point::new(0, 17));
        ed.run_mx("complete-symbol");
        assert!(ed.editor_completion.is_some());
        ed.handle_key(Key::code(KeyCode::Tab));
        assert_eq!(ed.buffer_text(), "alpha alphabet alpha");
    }

    #[test]
    fn editor_completion_is_off_by_default_and_clears_on_space() {
        let mut ed = editor("alpha ");
        for c in "al".chars() {
            ed.handle_key(Key::ch(c));
        }
        assert!(ed.editor_completion.is_none());

        let config = Config {
            editor_completion: true,
            ..Config::default()
        };
        let mut ed = Editor::from_text("alpha ", None, config);
        for c in "al".chars() {
            ed.handle_key(Key::ch(c));
        }
        assert!(ed.editor_completion.is_some());
        ed.handle_key(Key::ch(' '));
        assert!(ed.editor_completion.is_none());
    }

    #[test]
    fn spell_check_marks_and_navigates() {
        let dir = temp_dir();
        let dictionary = dir.join("words");
        fs::write(&dictionary, "hello\nworld\n").unwrap();
        let config = Config {
            spell_check: true,
            spell_dictionary: Some(dictionary.to_string_lossy().into_owned()),
            ..Config::default()
        };
        let mut ed = Editor::from_text("hello wurld\n", Some(dir.join("doc.md")), config);
        assert!(ed.spell.enabled);
        assert_eq!(ed.spell_spans(0), vec![(6, 11)]);

        ed.buffer.set_point(Point::new(0, 0));
        ed.run_mx("next-spelling-error");
        assert_eq!(ed.buffer.point, Point::new(0, 6));
        ed.handle_key(Key::alt('$'));
        assert!(ed.minibuffer.is_some());
        assert_eq!(ed.minibuffer_text().as_deref(), Some("world"));
        ed.handle_key(Key::code(KeyCode::Enter));
        assert_eq!(ed.buffer_text(), "hello world\n");
        assert!(ed.message.contains("Replaced"));
        assert_eq!(ed.spell_check_buffer(), 0);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn spell_check_toggles_with_a_dictionary() {
        let dir = temp_dir();
        let dictionary = dir.join("words");
        fs::write(&dictionary, "hello\nworld\n").unwrap();
        let config = Config {
            spell_dictionary: Some(dictionary.to_string_lossy().into_owned()),
            ..Config::default()
        };
        let mut ed = Editor::from_text("hello wurld\n", Some(dir.join("doc.md")), config);
        assert!(!ed.spell.enabled);
        ed.run_mx("flyspell-mode");
        assert!(ed.spell.enabled);
        ed.run_mx("flyspell-mode");
        assert!(!ed.spell.enabled);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn spell_check_skips_latex_commands() {
        let dir = temp_dir();
        let dictionary = dir.join("words");
        fs::write(&dictionary, "section\n").unwrap();
        let config = Config {
            spell_check: true,
            spell_dictionary: Some(dictionary.to_string_lossy().into_owned()),
            ..Config::default()
        };
        let mut ed = Editor::from_text("\\section{wurld}\n", Some(dir.join("doc.tex")), config);
        assert_eq!(ed.spell_spans(0), vec![(9, 14)]);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn common_prefix_of_commands() {
        let items = vec!["save-buffer".to_string(), "save-buffers".to_string()];
        assert_eq!(common_prefix(&items), "save-buffer");
    }

    #[test]
    fn typing_and_undo_through_keys() {
        let mut ed = editor("");
        for c in "hello".chars() {
            ed.handle_key(Key::ch(c));
        }
        assert_eq!(ed.buffer_text(), "hello");
        ed.handle_key(Key::ctrl('/'));
        assert_eq!(ed.buffer_text(), "");
        ed.handle_key(Key::alt('x'));
        for c in "redo".chars() {
            ed.handle_key(Key::ch(c));
        }
        ed.handle_key(Key::code(KeyCode::Enter));
        assert_eq!(ed.buffer_text(), "hello");
    }

    #[test]
    fn movement_keys() {
        let mut ed = editor("abc\ndef");
        ed.handle_key(Key::alt('f'));
        assert_eq!(ed.buffer.point, Point::new(0, 3));
        ed.handle_key(Key::ctrl('n'));
        assert_eq!(ed.buffer.point, Point::new(1, 3));
        ed.handle_key(Key::ctrl('a'));
        assert_eq!(ed.buffer.point, Point::new(1, 0));
        ed.handle_key(Key::alt('>'));
        assert_eq!(ed.buffer.point, Point::new(1, 3));
    }

    #[test]
    fn mark_and_kill_region() {
        let mut ed = editor("abcdef");
        ed.handle_key(Key::ctrl('f'));
        ed.handle_key(Key::ctrl(' '));
        ed.handle_key(Key::ctrl('f'));
        ed.handle_key(Key::ctrl('f'));
        ed.handle_key(Key::ctrl('w'));
        assert_eq!(ed.buffer_text(), "adef");
        ed.handle_key(Key::ctrl('y'));
        assert_eq!(ed.buffer_text(), "abcdef");
    }

    #[test]
    fn copy_region_and_yank() {
        let mut ed = editor("hello world");
        ed.handle_key(Key::ctrl(' '));
        for _ in 0..5 {
            ed.handle_key(Key::ctrl('f'));
        }
        ed.handle_key(Key::alt('w'));
        ed.handle_key(Key::alt('>'));
        ed.handle_key(Key::ctrl('y'));
        assert_eq!(ed.buffer_text(), "hello worldhello");
    }

    #[test]
    fn delete_and_backspace_remove_an_active_region() {
        let mut ed = editor("hello world");
        ed.handle_key(Key::ctrl(' '));
        for _ in 0..5 {
            ed.handle_key(Key::ctrl('f'));
        }
        ed.handle_key(Key::code(KeyCode::Delete));
        assert_eq!(ed.buffer_text(), " world");

        let mut ed = editor("hello world");
        ed.handle_key(Key::ctrl(' '));
        for _ in 0..5 {
            ed.handle_key(Key::ctrl('f'));
        }
        ed.handle_key(Key::code(KeyCode::Backspace));
        assert_eq!(ed.buffer_text(), " world");
    }

    #[test]
    fn region_commands_via_mx() {
        let mut ed = editor("one two");
        ed.run_mx("set-mark-command");
        for _ in 0..3 {
            ed.handle_key(Key::ctrl('f'));
        }
        ed.run_mx("kill-ring-save");
        ed.run_mx("delete-region");
        assert_eq!(ed.buffer_text(), " two");
        ed.handle_key(Key::ctrl('y'));
        assert_eq!(ed.buffer_text(), "one two");
    }

    #[test]
    fn incremental_search_moves_point() {
        let mut ed = editor("foo bar\nfoo baz");
        ed.handle_key(Key::ctrl('s'));
        for c in "baz".chars() {
            ed.handle_key(Key::ch(c));
        }
        assert_eq!(ed.buffer.point, Point::new(1, 4));
        ed.handle_key(Key::code(KeyCode::Enter));
        assert!(ed.search.is_none());
    }

    #[test]
    fn search_cancel_restores_point() {
        let mut ed = editor("one two");
        ed.handle_key(Key::ctrl('s'));
        ed.handle_key(Key::ch('t'));
        assert_eq!(ed.buffer.point, Point::new(0, 4));
        ed.handle_key(Key::ctrl('g'));
        assert_eq!(ed.buffer.point, Point::new(0, 0));
    }

    #[test]
    fn max_opens_and_runs_commands() {
        let mut ed = editor("a\nb\nc");
        ed.handle_key(Key::alt('x'));
        for c in "goto-line".chars() {
            ed.handle_key(Key::ch(c));
        }
        ed.handle_key(Key::code(KeyCode::Enter));
        for c in "3".chars() {
            ed.handle_key(Key::ch(c));
        }
        ed.handle_key(Key::code(KeyCode::Enter));
        assert_eq!(ed.buffer.point, Point::new(2, 0));
    }

    #[test]
    fn help_overlay_opens_and_closes() {
        let mut ed = editor("");
        ed.handle_key(Key::code(KeyCode::F(1)));
        assert!(ed.overlay.is_some());
        ed.handle_key(Key::code(KeyCode::Esc));
        assert!(ed.overlay.is_none());
    }

    #[test]
    fn quit_without_changes_is_immediate() {
        let mut ed = editor("hi");
        ed.handle_key(Key::ctrl('x'));
        ed.handle_key(Key::ctrl('c'));
        assert!(ed.should_quit);
        assert_eq!(ed.exit_code, 0);
    }

    #[test]
    fn quit_after_undoing_everything_is_immediate() {
        let mut ed = editor("hi");
        ed.handle_key(Key::ch('x'));
        ed.handle_key(Key::ctrl('_'));
        assert_eq!(ed.buffer_text(), "hi");
        ed.handle_key(Key::ctrl('x'));
        ed.handle_key(Key::ctrl('c'));
        assert!(ed.should_quit);
        assert!(ed.prompt_for_quit().is_none());
    }

    #[test]
    fn quit_empty_new_file_after_deleting_typed_text_is_immediate() {
        let dir = temp_dir();
        let path = dir.join("new.txt");
        let mut ed = Editor::from_text("", Some(path.clone()), Config::default());
        ed.handle_key(Key::ch('x'));
        ed.handle_key(Key::code(KeyCode::Backspace));
        assert_eq!(ed.buffer_text(), "");
        ed.handle_key(Key::ctrl('x'));
        ed.handle_key(Key::ctrl('c'));
        assert!(ed.should_quit);
        assert!(ed.prompt_for_quit().is_none());
        assert!(!path.exists());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn quit_with_changes_prompts() {
        let mut ed = editor("");
        ed.handle_key(Key::ch('x'));
        ed.handle_key(Key::ctrl('x'));
        ed.handle_key(Key::ctrl('c'));
        assert!(!ed.should_quit);
        assert!(ed.prompt_for_quit().is_some());
        ed.handle_key(Key::ch('n'));
        assert!(ed.should_quit);
    }

    #[test]
    fn raw_c_underscore_undoes() {
        // The raw 0x1f byte reaches the editor either as C-_ (tests) or as
        // C-7 (crossterm's mapping).  Both must undo.
        for key in [
            Key {
                code: KeyCode::Char('\u{1f}'),
                ctrl: true,
                alt: false,
                shift: false,
            },
            Key::ctrl('7'),
            Key::ctrl('_'),
        ] {
            let mut ed = editor("");
            for c in "abc".chars() {
                ed.handle_key(Key::ch(c));
            }
            ed.handle_key(key);
            assert_eq!(ed.buffer_text(), "", "key {}", key.display());
        }
    }

    #[test]
    fn tab_indents_to_stop() {
        let mut ed = editor("");
        ed.handle_key(Key::ch('a'));
        ed.handle_key(Key::code(KeyCode::Tab));
        assert_eq!(ed.buffer_text(), "a   ");
    }

    #[test]
    fn auto_fill_breaks_at_a_word_boundary() {
        let config = Config {
            auto_fill: true,
            fill_column: 10,
            ..Config::default()
        };
        let mut ed = Editor::from_text("", None, config);
        for c in "one two three four".chars() {
            ed.handle_key(Key::ch(c));
        }
        assert_eq!(ed.buffer_text(), "one two\nthree four");
        assert_eq!(ed.buffer.point, Point::new(1, 10));
    }

    #[test]
    fn auto_fill_uses_display_columns_for_wide_text() {
        let config = Config {
            auto_fill: true,
            fill_column: 8,
            ..Config::default()
        };
        let mut ed = Editor::from_text("", None, config);
        for c in "あああ あああ".chars() {
            ed.handle_key(Key::ch(c));
        }
        assert_eq!(ed.buffer_text(), "あああ\nあああ");
    }

    #[test]
    fn auto_fill_preserves_indentation() {
        let config = Config {
            auto_fill: true,
            fill_column: 10,
            ..Config::default()
        };
        let mut ed = Editor::from_text("", None, config);
        for c in "    one two three".chars() {
            ed.handle_key(Key::ch(c));
        }
        assert_eq!(ed.buffer_text(), "    one\n    two\n    three");
    }

    #[test]
    fn auto_fill_does_nothing_when_disabled() {
        let config = Config {
            fill_column: 10,
            ..Config::default()
        };
        assert!(!config.auto_fill);
        let mut ed = Editor::from_text("", None, config);
        for c in "one two three four".chars() {
            ed.handle_key(Key::ch(c));
        }
        assert_eq!(ed.buffer_text(), "one two three four");
    }

    #[test]
    fn undo_reverts_an_auto_fill_break() {
        let config = Config {
            auto_fill: true,
            fill_column: 10,
            ..Config::default()
        };
        let mut ed = Editor::from_text("one two th", None, config);
        ed.buffer.point = Point::new(0, 10);
        ed.handle_key(Key::ch('r'));
        assert_eq!(ed.buffer_text(), "one two\nthr");
        ed.handle_key(Key::ctrl('/'));
        assert_eq!(ed.buffer_text(), "one two th");
    }

    #[test]
    fn language_section_overrides_editor_settings() {
        let mut config = Config::default();
        config.parse("[markdown]\nauto_fill = true\nfill_column = 24\neditor_completion = true\n");
        let ed = Editor::from_text("", Some(PathBuf::from("note.md")), config);
        assert_eq!(ed.mode, Lang::Markdown);
        assert!(ed.settings.auto_fill);
        assert_eq!(ed.settings.fill_column, 24);
        assert!(ed.settings.editor_completion);
        assert_eq!(ed.settings.comment_start, "");
    }

    #[test]
    fn comment_dwim_uses_the_language_prefix() {
        let mut ed = Editor::from_text(
            "one\ntwo\n",
            Some(PathBuf::from("note.rs")),
            Config::default(),
        );
        assert_eq!(ed.settings.comment_start, "//");
        ed.handle_key(Key::alt(';'));
        assert_eq!(ed.buffer_text(), "// one\ntwo\n");
        ed.handle_key(Key::alt(';'));
        assert_eq!(ed.buffer_text(), "one\ntwo\n");
    }

    #[test]
    fn comment_dwim_reports_missing_syntax() {
        let mut ed = editor("plain\n");
        ed.handle_key(Key::alt(';'));
        assert!(ed.message.contains("No line-comment"));
        assert_eq!(ed.buffer_text(), "plain\n");
    }

    #[test]
    fn tabs_open_switch_and_close() {
        let dir = temp_dir();
        let first = dir.join("first.txt");
        let second = dir.join("second.txt");
        fs::write(&first, "first\n").unwrap();
        fs::write(&second, "second\n").unwrap();
        let mut ed = editor("");
        ed.open_file(first);
        assert_eq!(ed.tab_count(), 1);
        assert_eq!(ed.buffer_text(), "first\n");

        ed.open_file(second);
        assert_eq!(ed.tab_count(), 2);
        assert_eq!(ed.buffer_text(), "second\n");
        assert!(ed
            .tab_labels()
            .iter()
            .any(|(name, active)| name == "second.txt" && *active));

        ed.switch_tab(-1);
        assert_eq!(ed.buffer_text(), "first\n");
        ed.switch_tab(1);
        assert_eq!(ed.buffer_text(), "second\n");

        ed.close_tab();
        assert_eq!(ed.tab_count(), 1);
        assert_eq!(ed.buffer_text(), "first\n");
        ed.close_tab();
        assert!(ed.should_quit);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn tabs_preserve_unsaved_buffers() {
        let dir = temp_dir();
        let first = dir.join("first.txt");
        fs::write(&first, "first\n").unwrap();
        let mut ed = editor("");
        ed.handle_key(Key::ch('d'));
        ed.open_file(first.clone());
        assert_eq!(ed.tab_count(), 2);
        assert_eq!(ed.buffer_text(), "first\n");
        ed.switch_tab(-1);
        assert_eq!(ed.buffer_text(), "d");
        ed.open_file(first);
        assert_eq!(ed.tab_count(), 2);
        assert_eq!(ed.buffer_text(), "first\n");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn quit_switches_to_another_unsaved_tab_first() {
        let dir = temp_dir();
        let path = dir.join("clean.txt");
        fs::write(&path, "clean\n").unwrap();
        let mut ed = editor("");
        ed.handle_key(Key::ch('x'));
        ed.open_file(path);
        assert_eq!(ed.tab_count(), 2);
        assert_eq!(ed.buffer_text(), "clean\n");
        ed.handle_key(Key::ctrl('x'));
        ed.handle_key(Key::ctrl('c'));
        assert!(!ed.should_quit);
        assert_eq!(ed.buffer_text(), "x");
        assert!(ed.message.contains("unsaved"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn tab_close_refuses_unsaved_changes() {
        let mut ed = editor("");
        ed.new_tab();
        assert_eq!(ed.tab_count(), 2);
        ed.handle_key(Key::ch('x'));
        ed.close_tab();
        assert_eq!(ed.tab_count(), 2);
        assert!(!ed.should_quit);
        assert!(ed.message.contains("modified"));
    }

    #[test]
    fn next_error_jumps_and_wraps() {
        let mut ed = editor("one\ntwo\n");
        ed.diagnostics = vec![
            Diagnostic {
                row: 0,
                start: 0,
                end: 3,
                severity: Severity::Warning,
                message: "warning".into(),
                source: None,
            },
            Diagnostic {
                row: 1,
                start: 0,
                end: 3,
                severity: Severity::Error,
                message: "error".into(),
                source: None,
            },
        ];
        ed.next_diagnostic(true);
        assert_eq!(ed.buffer.point, Point::new(1, 0));
        assert!(ed.message.contains("Error: error"));
        ed.next_diagnostic(true);
        assert_eq!(ed.buffer.point, Point::new(0, 0));
        ed.next_diagnostic(false);
        assert_eq!(ed.buffer.point, Point::new(1, 0));
    }

    #[test]
    fn toggle_commands_flip_fill_and_truncate() {
        let mut ed = editor("");
        ed.run_mx("auto-fill-mode");
        assert!(ed.settings.auto_fill);
        ed.run_mx("auto-fill-mode");
        assert!(!ed.settings.auto_fill);

        ed.run_mx("toggle-truncate-lines");
        assert!(!ed.settings.truncate_lines);
        assert!(!ed.view.truncate_lines);
        assert_eq!(ed.view.left, 0);
        ed.run_mx("toggle-truncate-lines");
        assert!(ed.settings.truncate_lines);
        assert!(ed.view.truncate_lines);
    }
}
