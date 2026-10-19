//! Editor state and key dispatch. This module has no terminal dependency so it
//! can be driven headlessly from tests.

use std::fs;
use std::io;
use std::path::PathBuf;

use crate::buffer::{Buffer, CaseOp, Coalesce, Point};
use crate::config::Config;
use crate::help;
use crate::highlight::Highlighter;
use crate::key::{Key, KeyCode};
use crate::minibuffer::{Minibuffer, PromptKind};
use crate::search::{find_backward, find_forward, find_last, SearchState};
use crate::syntax::Lang;
use crate::theme::Theme;
use crate::view::{display_col, View};

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

pub struct Editor {
    pub buffer: Buffer,
    pub mode: Lang,
    pub theme: Theme,
    pub view: View,
    pub config: Config,
    pub highlighter: Highlighter,
    pub minibuffer: Option<Minibuffer>,
    pub search: Option<SearchState>,
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
        let version = buffer.version;
        let view = View {
            line_numbers: config.line_numbers,
            tab_width: config.tab_width,
            highlight_trailing_ws: config.highlight_trailing_ws,
            ..View::default()
        };
        Self {
            buffer,
            mode,
            theme,
            view,
            highlighter: Highlighter::new(mode, version),
            config,
            minibuffer: None,
            search: None,
            confirm: None,
            overlay: None,
            message: String::new(),
            prefix: None,
            meta_prefix: false,
            should_quit: false,
            exit_code: 0,
            history: Vec::new(),
            needs_redraw: false,
        }
    }

    // ------------------------------------------------------------- helpers

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
        self.highlighter = Highlighter::new(self.mode, buffer.version);
        self.buffer = buffer;
        self.view.top = 0;
        self.view.left = 0;
        self.search = None;
        self.prefix = None;
        self.needs_redraw = true;
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
        self.buffer.end_command(coalesce);
    }

    pub fn handle_paste(&mut self, text: &str) {
        if self.overlay.is_some() || self.confirm.is_some() {
            return;
        }
        if let Some(mb) = self.minibuffer.as_mut() {
            mb.insert_str(text);
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
        self.buffer.begin_command();
        self.buffer.insert_text(text);
        self.buffer.end_command(None);
    }

    fn handle_normal(&mut self, mut key: Key) {
        self.message.clear();
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
            }
            KeyCode::Enter => {
                self.buffer.newline(self.config.auto_indent);
            }
            KeyCode::Tab => self.indent(),
            KeyCode::BackTab => self.unindent(),
            KeyCode::Backspace if key.alt => {
                self.buffer.kill_word_backward();
            }
            KeyCode::Backspace => {
                self.buffer.delete_backward();
            }
            KeyCode::Delete => {
                self.buffer.delete_forward();
            }
            KeyCode::Esc => {
                self.meta_prefix = true;
                self.message = "M-".into();
            }
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
                self.buffer.delete_forward();
            }
            'h' => {
                self.buffer.delete_backward();
            }
            'k' => {
                self.buffer.kill_line();
            }
            'y' => self.yank(),
            'w' => {
                if self.buffer.kill_region().is_empty() {
                    self.message = "No active region".into();
                }
            }
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
                self.buffer.newline(self.config.auto_indent);
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
            'w' => {
                if self.buffer.copy_region().is_empty() {
                    self.message = "No active region".into();
                } else {
                    self.message = "Copied region".into();
                }
            }
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
                KeyCode::Char('s') => self.save_buffer(),
                _ => self.message = format!("C-x {} is unbound", key.display()),
            },
            Prefix::Cc => match key.code {
                KeyCode::Char('c') if key.ctrl => self.save_and_quit(),
                KeyCode::Char('k') if key.ctrl => self.abort_commit(),
                _ => self.message = format!("C-c {} is unbound", key.display()),
            },
            Prefix::MetaG => match key.code {
                KeyCode::Char('g') if !key.ctrl && !key.alt => self.prompt_goto_line(),
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

    fn indent(&mut self) {
        let row = self.buffer.point.row;
        let col = display_col(
            &self.buffer.lines[row],
            self.buffer.point.col,
            self.config.tab_width,
        );
        let spaces = self.config.tab_width - (col % self.config.tab_width);
        let text = " ".repeat(spaces);
        self.buffer.insert_text(&text);
    }

    fn unindent(&mut self) {
        let row = self.buffer.point.row;
        let width = self.config.tab_width;
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
                    && self.buffer.is_effectively_empty(self.config.comment_char)
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
                    && self.buffer.is_effectively_empty(self.config.comment_char)
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
        if self.buffer.modified {
            self.confirm = Some(Confirm {
                kind: ConfirmKind::AbortCommit,
                question: "Abort commit and discard changes? (y/n)".into(),
            });
        } else {
            self.finish_quit(1);
        }
    }

    fn quit_request(&mut self) {
        if !self.buffer.modified || !self.config.confirm_on_quit {
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
        let mut history_prev = false;
        let mut history_next = false;
        let mut complete = false;
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
                    }
                    'k' => {
                        mb.kill_to_end();
                    }
                    'w' => {
                        mb.delete_word_backward();
                    }
                    'g' => cancel = true,
                    'p' => history_prev = true,
                    'n' => history_next = true,
                    _ => {}
                },
                KeyCode::Char(c) if !key.alt => mb.insert(c),
                KeyCode::Backspace => {
                    mb.backspace();
                }
                KeyCode::Delete => {
                    mb.delete_forward();
                }
                KeyCode::Left => mb.left(),
                KeyCode::Right => mb.right(),
                KeyCode::Home => mb.home(),
                KeyCode::End => mb.end(),
                KeyCode::Tab => complete = true,
                KeyCode::Enter => accept = true,
                KeyCode::Esc => cancel = true,
                KeyCode::Up => history_prev = true,
                KeyCode::Down => history_next = true,
                _ => {}
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
        if complete {
            self.complete();
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
        }
    }

    fn complete(&mut self) {
        let Some(kind) = self.minibuffer.as_ref().map(|m| m.kind.clone()) else {
            return;
        };
        match kind {
            PromptKind::Mx => self.complete_command(),
            PromptKind::FindFile | PromptKind::WriteFile { .. } => self.complete_path(),
            PromptKind::GotoLine => {}
        }
    }

    fn complete_command(&mut self) {
        let Some(mb) = self.minibuffer.as_mut() else {
            return;
        };
        let text = mb.text();
        let names: Vec<String> = COMMANDS
            .iter()
            .map(|(name, _)| (*name).to_string())
            .filter(|name| name.starts_with(&text))
            .collect();
        if names.is_empty() {
            mb.hint = "No match".into();
            return;
        }
        if names.len() == 1 {
            mb.set_text(&names[0]);
            mb.hint.clear();
            return;
        }
        let prefix = common_prefix(&names);
        if prefix.chars().count() > text.chars().count() {
            mb.set_text(&prefix);
        }
        mb.hint = format!(
            "{} commands: {}",
            names.len(),
            names.iter().take(5).cloned().collect::<Vec<_>>().join(" ")
        );
    }

    fn complete_path(&mut self) {
        let base_dir = self.buffer.dir();
        let Some(mb) = self.minibuffer.as_mut() else {
            return;
        };
        let raw = mb.text();
        let (dir_part, name_part) = match raw.rfind('/') {
            Some(i) => (&raw[..=i], &raw[i + 1..]),
            None => ("", raw.as_str()),
        };
        let dir = if dir_part.is_empty() {
            base_dir
        } else {
            expand_tilde(dir_part)
        };
        let mut entries: Vec<(String, bool)> = Vec::new();
        if let Ok(read_dir) = fs::read_dir(&dir) {
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
        }
        entries.sort();
        if entries.is_empty() {
            mb.hint = format!("No match in {}", dir.display());
            return;
        }
        if entries.len() == 1 {
            let (name, is_dir) = &entries[0];
            let mut new = format!("{dir_part}{name}");
            if *is_dir {
                new.push('/');
            }
            mb.set_text(&new);
            mb.hint.clear();
            return;
        }
        let names: Vec<String> = entries.iter().map(|(n, _)| n.clone()).collect();
        let prefix = common_prefix(&names);
        if prefix.chars().count() > name_part.chars().count() {
            mb.set_text(&format!("{dir_part}{prefix}"));
        }
        mb.hint = format!(
            "{} candidates: {}",
            entries.len(),
            names.iter().take(4).cloned().collect::<Vec<_>>().join(" ")
        );
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
        if self.buffer.modified {
            self.message = "Buffer modified: save it first (C-x C-s)".into();
            return;
        }
        match Buffer::open(&path) {
            Ok(buffer) => {
                let name = buffer.display_name();
                let lines = buffer.line_count();
                self.set_buffer(buffer);
                self.message = format!("Opened {name} ({lines} lines)");
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let mut buffer = Buffer::from_text("", Some(path.clone()));
                buffer.modified = false;
                self.set_buffer(buffer);
                self.message = format!("(New file) {}", path.display());
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
                self.config.line_numbers = !self.config.line_numbers;
                self.view.line_numbers = self.config.line_numbers;
                self.message = format!(
                    "Line numbers {}",
                    if self.config.line_numbers {
                        "on"
                    } else {
                        "off"
                    }
                );
            }
            Cmd::ToggleTrailingWs => {
                self.config.highlight_trailing_ws = !self.config.highlight_trailing_ws;
                self.view.highlight_trailing_ws = self.config.highlight_trailing_ws;
                self.message = format!(
                    "Trailing whitespace highlight {}",
                    if self.config.highlight_trailing_ws {
                        "on"
                    } else {
                        "off"
                    }
                );
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
            self.highlighter = Highlighter::new(mode, self.buffer.version);
        }
    }

    fn revert_buffer(&mut self) {
        if self.buffer.modified {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn editor(text: &str) -> Editor {
        Editor::from_text(text, None, Config::default())
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
}
