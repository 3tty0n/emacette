//! The text buffer: lines, point/mark, undo/redo and the kill ring.
//!
//! Lines are stored as `Vec<char>` so that column arithmetic is
//! straightforward for multi-byte text (Japanese, emoji, ...).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub type Line = Vec<char>;

/// A logical position in the buffer: row and *character* column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    pub row: usize,
    pub col: usize,
}

impl Point {
    pub const fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    Crlf,
}

impl LineEnding {
    pub fn text(self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            LineEnding::Lf => "LF",
            LineEnding::Crlf => "CRLF",
        }
    }
}

/// Which style of edit a group of changes may be merged with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coalesce {
    Insert,
    Delete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseOp {
    Upcase,
    Downcase,
    Capitalize,
}

/// The Emacs-style kill ring. The most recent kill is at `items[0]`.
#[derive(Clone, Debug, Default)]
pub struct KillRing {
    items: Vec<String>,
    index: usize,
}

impl KillRing {
    pub fn push(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        if self.items.first().map(|s| s == text).unwrap_or(false) {
            self.index = 0;
            return;
        }
        self.items.insert(0, text.to_string());
        if self.items.len() > 100 {
            self.items.truncate(100);
        }
        self.index = 0;
    }

    pub fn current(&self) -> Option<&str> {
        self.items.get(self.index).map(String::as_str)
    }

    /// Move to the previous kill, for `yank-pop`.
    pub fn rotate(&mut self) {
        if self.items.len() > 1 {
            self.index = (self.index + 1) % self.items.len();
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[derive(Clone, Debug)]
struct Change {
    row: usize,
    old: Vec<Line>,
    new: Vec<Line>,
}

#[derive(Clone, Debug)]
struct UndoGroup {
    changes: Vec<Change>,
    before: Point,
    after: Point,
    coalesce: Option<Coalesce>,
}

pub struct Buffer {
    pub lines: Vec<Line>,
    pub point: Point,
    pub mark: Option<Point>,
    pub path: Option<PathBuf>,
    pub modified: bool,
    pub line_ending: LineEnding,
    pub kill_ring: KillRing,
    /// Bumped on every change; used to invalidate syntax highlighting.
    pub version: u64,
    /// Lowest row touched since the last `dirty_from` read. `usize::MAX` = clean.
    pub dirty_from: usize,

    goal_col: Option<usize>,
    changes: Vec<Change>,
    undo: Vec<UndoGroup>,
    redo: Vec<UndoGroup>,
    group_start: Point,
    last_yank: Option<(Point, Point)>,
    saved_text: String,
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("lines", &self.lines.len())
            .field("point", &self.point)
            .field("path", &self.path)
            .field("modified", &self.modified)
            .finish()
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    pub fn new() -> Self {
        Self::from_text("", None)
    }

    pub fn from_text(text: &str, path: Option<PathBuf>) -> Self {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let crlf = text.contains("\r\n");
        let mut normalized = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\r' {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                normalized.push('\n');
            } else {
                normalized.push(c);
            }
        }
        let lines = if normalized.is_empty() {
            vec![Vec::new()]
        } else {
            normalized
                .split('\n')
                .map(|s| s.chars().collect())
                .collect()
        };
        let mut buffer = Buffer {
            lines,
            point: Point::default(),
            mark: None,
            path,
            modified: false,
            line_ending: if crlf {
                LineEnding::Crlf
            } else {
                LineEnding::Lf
            },
            kill_ring: KillRing::default(),
            version: 1,
            dirty_from: 0,
            goal_col: None,
            changes: Vec::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            group_start: Point::default(),
            last_yank: None,
            saved_text: String::new(),
        };
        buffer.saved_text = buffer.render_text();
        buffer
    }

    pub fn open(path: &Path) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        if bytes.iter().take(8192).any(|&b| b == 0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "file appears to be binary",
            ));
        }
        let text = String::from_utf8(bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "file is not valid UTF-8"))?;
        Ok(Self::from_text(&text, Some(path.to_path_buf())))
    }

    // ---------------------------------------------------------------- info

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn line_len(&self, row: usize) -> usize {
        self.lines.get(row).map_or(0, Vec::len)
    }

    pub fn display_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "*scratch*".to_string())
    }

    pub fn dir(&self) -> PathBuf {
        self.path
            .as_ref()
            .and_then(|p| p.parent())
            .map(Path::to_path_buf)
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Serialize the buffer (honouring the file's line ending style).
    pub fn render_text(&self) -> String {
        let eol = self.line_ending.text();
        let mut out = String::with_capacity(self.lines.iter().map(Vec::len).sum::<usize>() + 16);
        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                out.push_str(eol);
            }
            out.extend(line.iter().copied());
        }
        out
    }

    pub fn text(&self) -> String {
        let mut out = String::new();
        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.extend(line.iter().copied());
        }
        out
    }

    // ------------------------------------------------------------- loading

    pub fn save(&mut self) -> io::Result<PathBuf> {
        let path = self.path.clone().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "buffer has no file name")
        })?;
        self.save_as(&path)?;
        Ok(path)
    }

    pub fn save_as(&mut self, path: &Path) -> io::Result<()> {
        let text = self.render_text();
        fs::write(path, text.as_bytes())?;
        self.path = Some(path.to_path_buf());
        self.modified = false;
        self.saved_text = text;
        Ok(())
    }

    /// True when the buffer text differs from the last loaded/saved text.
    pub fn has_unsaved_changes(&self) -> bool {
        self.render_text() != self.saved_text
    }

    // ------------------------------------------------------------ movement

    fn clamp_point(&mut self) {
        if self.point.row >= self.lines.len() {
            self.point.row = self.lines.len().saturating_sub(1);
        }
        let max = self.lines[self.point.row].len();
        if self.point.col > max {
            self.point.col = max;
        }
    }

    pub fn set_point(&mut self, p: Point) {
        self.point = p;
        self.clamp_point();
        self.goal_col = None;
        self.last_yank = None;
    }

    pub fn set_goal_col(&mut self, col: usize) {
        self.goal_col = Some(col);
    }

    pub fn move_left(&mut self) {
        if self.point.col > 0 {
            self.point.col -= 1;
        } else if self.point.row > 0 {
            self.point.row -= 1;
            self.point.col = self.lines[self.point.row].len();
        }
        self.goal_col = None;
        self.last_yank = None;
    }

    pub fn move_right(&mut self) {
        if self.point.col < self.lines[self.point.row].len() {
            self.point.col += 1;
        } else if self.point.row + 1 < self.lines.len() {
            self.point.row += 1;
            self.point.col = 0;
        }
        self.goal_col = None;
        self.last_yank = None;
    }

    pub fn move_vertical(&mut self, delta: isize) {
        let goal = self.goal_col.unwrap_or(self.point.col);
        let max_row = self.lines.len().saturating_sub(1) as isize;
        let row = (self.point.row as isize + delta).clamp(0, max_row) as usize;
        self.point.row = row;
        self.point.col = goal.min(self.lines[row].len());
        self.goal_col = Some(goal);
        self.last_yank = None;
    }

    pub fn move_line_start(&mut self) {
        self.point.col = 0;
        self.goal_col = None;
        self.last_yank = None;
    }

    pub fn move_line_end(&mut self) {
        self.point.col = self.lines[self.point.row].len();
        self.goal_col = None;
        self.last_yank = None;
    }

    pub fn move_buffer_start(&mut self) {
        self.point = Point::new(0, 0);
        self.goal_col = None;
        self.last_yank = None;
    }

    pub fn move_buffer_end(&mut self) {
        let row = self.lines.len() - 1;
        self.point = Point::new(row, self.lines[row].len());
        self.goal_col = None;
        self.last_yank = None;
    }

    pub fn move_to_line(&mut self, line: usize) {
        let row = line.saturating_sub(1).min(self.lines.len() - 1);
        self.point = Point::new(row, 0);
        self.goal_col = None;
        self.last_yank = None;
    }

    /// Point moved to the first non-blank character of the line.
    pub fn back_to_indentation(&mut self) {
        let indent = self.lines[self.point.row]
            .iter()
            .take_while(|c| **c == ' ' || **c == '\t')
            .count();
        self.point.col = indent;
        self.goal_col = None;
        self.last_yank = None;
    }

    pub fn next_pos(&self, p: Point) -> Option<Point> {
        if p.col < self.lines[p.row].len() {
            Some(Point::new(p.row, p.col + 1))
        } else if p.row + 1 < self.lines.len() {
            Some(Point::new(p.row + 1, 0))
        } else {
            None
        }
    }

    pub fn prev_pos(&self, p: Point) -> Option<Point> {
        if p.col > 0 {
            Some(Point::new(p.row, p.col - 1))
        } else if p.row > 0 {
            Some(Point::new(p.row - 1, self.lines[p.row - 1].len()))
        } else {
            None
        }
    }

    /// The character *after* point, treating a line break as `'\n'`.
    fn char_after(&self, p: Point) -> Option<char> {
        let line = &self.lines[p.row];
        if p.col < line.len() {
            Some(line[p.col])
        } else if p.row + 1 < self.lines.len() {
            Some('\n')
        } else {
            None
        }
    }

    /// The character *before* point, treating a line break as `'\n'`.
    fn char_before(&self, p: Point) -> Option<char> {
        if p.col > 0 {
            Some(self.lines[p.row][p.col - 1])
        } else if p.row > 0 {
            Some('\n')
        } else {
            None
        }
    }

    pub fn word_forward_point(&self) -> Point {
        let mut p = self.point;
        while let Some(c) = self.char_after(p) {
            if is_word_char(c) {
                break;
            }
            match self.next_pos(p) {
                Some(n) => p = n,
                None => return p,
            }
        }
        while let Some(c) = self.char_after(p) {
            if !is_word_char(c) {
                break;
            }
            match self.next_pos(p) {
                Some(n) => p = n,
                None => return p,
            }
        }
        p
    }

    pub fn word_backward_point(&self) -> Point {
        let mut p = self.point;
        while let Some(c) = self.char_before(p) {
            if is_word_char(c) {
                break;
            }
            match self.prev_pos(p) {
                Some(n) => p = n,
                None => return p,
            }
        }
        while let Some(c) = self.char_before(p) {
            if !is_word_char(c) {
                break;
            }
            match self.prev_pos(p) {
                Some(n) => p = n,
                None => return p,
            }
        }
        p
    }

    pub fn forward_word(&mut self) {
        self.point = self.word_forward_point();
        self.goal_col = None;
    }

    pub fn backward_word(&mut self) {
        self.point = self.word_backward_point();
        self.goal_col = None;
    }

    // -------------------------------------------------------------- undo

    pub fn begin_command(&mut self) {
        self.changes.clear();
        self.group_start = self.point;
    }

    pub fn end_command(&mut self, coalesce: Option<Coalesce>) {
        // Any command that is not itself a coalescable edit seals the
        // previous undo group, so typing across a cursor movement does not
        // merge into one undo step.
        if coalesce.is_none() {
            if let Some(last) = self.undo.last_mut() {
                last.coalesce = None;
            }
        }
        if self.changes.is_empty() {
            return;
        }
        let changes = std::mem::take(&mut self.changes);
        self.redo.clear();
        if let Some(c) = coalesce {
            if let Some(last) = self.undo.last_mut() {
                if last.coalesce == Some(c) {
                    last.changes.extend(changes);
                    last.after = self.point;
                    return;
                }
            }
        }
        self.undo.push(UndoGroup {
            changes,
            before: self.group_start,
            after: self.point,
            coalesce,
        });
        if self.undo.len() > 5000 {
            self.undo.remove(0);
        }
    }

    fn record(&mut self, row: usize, old: Vec<Line>, new_count: usize) {
        debug_assert!(new_count > 0);
        let end = (row + new_count).min(self.lines.len());
        let new = self.lines[row..end].to_vec();
        self.changes.push(Change { row, old, new });
        self.version += 1;
        self.dirty_from = self.dirty_from.min(row);
        self.modified = true;
        self.last_yank = None;
    }

    fn apply_undo(&mut self, group: &UndoGroup) {
        let mut dirty = usize::MAX;
        for ch in group.changes.iter().rev() {
            let start = ch.row.min(self.lines.len());
            let end = (ch.row + ch.new.len()).min(self.lines.len()).max(start);
            self.lines.splice(start..end, ch.old.iter().cloned());
            dirty = dirty.min(start);
        }
        self.after_undo_redo(dirty, group.before);
    }

    fn apply_redo(&mut self, group: &UndoGroup) {
        let mut dirty = usize::MAX;
        for ch in group.changes.iter() {
            let start = ch.row.min(self.lines.len());
            let end = (ch.row + ch.old.len()).min(self.lines.len()).max(start);
            self.lines.splice(start..end, ch.new.iter().cloned());
            dirty = dirty.min(start);
        }
        self.after_undo_redo(dirty, group.after);
    }

    fn after_undo_redo(&mut self, dirty: usize, point: Point) {
        debug_assert!(!self.lines.is_empty());
        self.version += 1;
        self.dirty_from = self.dirty_from.min(dirty);
        self.point = clamp_point(point, &self.lines);
        self.goal_col = None;
        self.last_yank = None;
        self.modified = self.render_text() != self.saved_text;
    }

    pub fn undo(&mut self) -> bool {
        let Some(group) = self.undo.pop() else {
            return false;
        };
        self.apply_undo(&group);
        self.redo.push(group);
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(group) = self.redo.pop() else {
            return false;
        };
        self.apply_redo(&group);
        self.undo.push(group);
        true
    }

    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    pub fn has_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    // ------------------------------------------------------------ editing

    pub fn insert_char(&mut self, c: char) {
        if c == '\n' {
            self.newline(false);
            return;
        }
        let row = self.point.row;
        let col = self.point.col;
        let old = self.lines[row].clone();
        self.lines[row].insert(col, c);
        self.point.col += 1;
        self.record(row, vec![old], 1);
        self.goal_col = None;
    }

    /// Insert text, splitting lines on `'\n'`, as one undoable change.
    pub fn insert_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        let row = self.point.row;
        let col = self.point.col.min(self.lines[row].len());
        let old = self.lines[row].clone();
        let mut parts: Vec<Line> = normalized
            .split('\n')
            .map(|s| s.chars().collect())
            .collect();
        if parts.len() == 1 {
            let n = parts[0].len();
            self.lines[row].splice(col..col, parts[0].iter().copied());
            self.point = Point::new(row, col + n);
            self.record(row, vec![old], 1);
        } else {
            let n = parts.len();
            let suffix = old[col..].to_vec();
            let mut new_lines: Vec<Line> = Vec::with_capacity(n);
            let mut first = old[..col].to_vec();
            first.extend(parts[0].iter().copied());
            new_lines.push(first);
            for part in parts.drain(1..) {
                new_lines.push(part);
            }
            let last_len = new_lines[n - 1].len();
            new_lines[n - 1].extend(suffix);
            self.lines.splice(row..row + 1, new_lines);
            self.point = Point::new(row + n - 1, last_len);
            self.record(row, vec![old], n);
        }
        self.goal_col = None;
    }

    pub fn newline(&mut self, auto_indent: bool) {
        let row = self.point.row;
        let col = self.point.col;
        let old = self.lines[row].clone();
        let indent_len = old.iter().take_while(|c| **c == ' ' || **c == '\t').count();
        let indent: Line = if auto_indent && col >= indent_len {
            old[..indent_len].to_vec()
        } else {
            Vec::new()
        };
        let suffix = old[col..].to_vec();
        let indent_col = indent.len();
        let mut second = indent;
        second.extend(suffix);
        self.lines[row].truncate(col);
        self.lines.insert(row + 1, second);
        self.point = Point::new(row + 1, indent_col);
        self.record(row, vec![old], 2);
        self.goal_col = None;
    }

    /// C-o: open a new line after point, leaving point where it was.
    pub fn open_line(&mut self) -> bool {
        let row = self.point.row;
        let col = self.point.col;
        let old = self.lines[row].clone();
        let suffix = old[col..].to_vec();
        self.lines[row].truncate(col);
        self.lines.insert(row + 1, suffix);
        self.record(row, vec![old], 2);
        self.goal_col = None;
        true
    }

    pub fn delete_backward(&mut self) -> bool {
        let row = self.point.row;
        if self.point.col > 0 {
            let col = self.point.col - 1;
            let old = self.lines[row].clone();
            self.lines[row].remove(col);
            self.point.col = col;
            self.record(row, vec![old], 1);
            self.goal_col = None;
            true
        } else if row > 0 {
            let prev = self.lines[row - 1].clone();
            let cur = self.lines[row].clone();
            let new_col = prev.len();
            let old = vec![prev.clone(), cur.clone()];
            let mut joined = prev;
            joined.extend(cur);
            self.lines.splice(row - 1..=row, [joined]);
            self.point = Point::new(row - 1, new_col);
            self.record(row - 1, old, 1);
            self.goal_col = None;
            true
        } else {
            false
        }
    }

    pub fn delete_forward(&mut self) -> bool {
        let row = self.point.row;
        let len = self.lines[row].len();
        if self.point.col < len {
            let old = self.lines[row].clone();
            self.lines[row].remove(self.point.col);
            self.record(row, vec![old], 1);
            self.goal_col = None;
            true
        } else if row + 1 < self.lines.len() {
            let old = self.lines[row..=row + 1].to_vec();
            let next = self.lines[row + 1].clone();
            self.lines[row].extend(next);
            self.lines.remove(row + 1);
            self.record(row, old, 1);
            self.goal_col = None;
            true
        } else {
            false
        }
    }

    /// Delete the range `[start, end)` and return the removed text.
    /// Point is moved to `start`.
    pub fn delete_range(&mut self, start: Point, end: Point) -> String {
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        if start == end {
            return String::new();
        }
        if start.row == end.row {
            let old = self.lines[start.row].clone();
            let removed: String = old[start.col..end.col].iter().copied().collect();
            self.lines[start.row].drain(start.col..end.col);
            self.point = start;
            self.record(start.row, vec![old], 1);
            self.goal_col = None;
            return removed;
        }

        let first = self.lines[start.row].clone();
        let last = self.lines[end.row].clone();
        let old = self.lines[start.row..=end.row].to_vec();
        let mut removed = String::new();
        removed.extend(first[start.col..].iter().copied());
        removed.push('\n');
        for r in start.row + 1..end.row {
            removed.extend(self.lines[r].iter().copied());
            removed.push('\n');
        }
        removed.extend(last[..end.col].iter().copied());

        let mut joined: Line = first[..start.col].to_vec();
        joined.extend(last[end.col..].iter().copied());
        self.lines.splice(start.row..=end.row, [joined]);
        self.point = start;
        self.record(start.row, old, 1);
        self.goal_col = None;
        removed
    }

    pub fn text_between(&self, start: Point, end: Point) -> String {
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        if start == end {
            return String::new();
        }
        let mut out = String::new();
        if start.row == end.row {
            out.extend(self.lines[start.row][start.col..end.col].iter().copied());
        } else {
            out.extend(self.lines[start.row][start.col..].iter().copied());
            out.push('\n');
            for r in start.row + 1..end.row {
                out.extend(self.lines[r].iter().copied());
                out.push('\n');
            }
            out.extend(self.lines[end.row][..end.col].iter().copied());
        }
        out
    }

    pub fn replace_range(&mut self, start: Point, end: Point, text: &str) -> String {
        let removed = self.delete_range(start, end);
        self.point = start;
        self.insert_text(text);
        removed
    }

    // ------------------------------------------------------- kill / yank

    pub fn region(&self) -> Option<(Point, Point)> {
        let mark = self.mark?;
        if mark == self.point {
            return None;
        }
        Some(if mark < self.point {
            (mark, self.point)
        } else {
            (self.point, mark)
        })
    }

    pub fn set_mark(&mut self) {
        self.mark = Some(self.point);
    }

    pub fn deactivate_mark(&mut self) {
        self.mark = None;
    }

    pub fn exchange_point_and_mark(&mut self) -> bool {
        let Some(mark) = self.mark else {
            return false;
        };
        self.mark = Some(self.point);
        self.point = mark;
        self.goal_col = None;
        true
    }

    pub fn mark_whole_buffer(&mut self) {
        self.mark = Some(Point::new(0, 0));
        self.move_buffer_end();
    }

    pub fn kill_region(&mut self) -> String {
        let Some((start, end)) = self.region() else {
            return String::new();
        };
        let removed = self.delete_range(start, end);
        self.kill_ring.push(&removed);
        self.mark = None;
        removed
    }

    pub fn copy_region(&mut self) -> String {
        let Some((start, end)) = self.region() else {
            return String::new();
        };
        let text = self.text_between(start, end);
        self.kill_ring.push(&text);
        text
    }

    /// Delete the active region without touching the kill ring.
    pub fn delete_region(&mut self) -> String {
        let Some((start, end)) = self.region() else {
            return String::new();
        };
        let removed = self.delete_range(start, end);
        self.mark = None;
        removed
    }

    pub fn kill_line(&mut self) -> String {
        let row = self.point.row;
        let col = self.point.col;
        if col < self.lines[row].len() {
            let old = self.lines[row].clone();
            let removed: String = old[col..].iter().copied().collect();
            self.lines[row].truncate(col);
            self.record(row, vec![old], 1);
            self.kill_ring.push(&removed);
            self.goal_col = None;
            removed
        } else if row + 1 < self.lines.len() {
            let old = self.lines[row..=row + 1].to_vec();
            let next = self.lines[row + 1].clone();
            self.lines[row].extend(next);
            self.lines.remove(row + 1);
            self.record(row, old, 1);
            self.kill_ring.push("\n");
            self.goal_col = None;
            "\n".to_string()
        } else {
            String::new()
        }
    }

    pub fn kill_word_forward(&mut self) -> String {
        let end = self.word_forward_point();
        let removed = self.delete_range(self.point, end);
        self.kill_ring.push(&removed);
        removed
    }

    pub fn kill_word_backward(&mut self) -> String {
        let start = self.word_backward_point();
        let removed = self.delete_range(start, self.point);
        self.kill_ring.push(&removed);
        removed
    }

    pub fn yank(&mut self) -> bool {
        let Some(text) = self.kill_ring.current().map(str::to_string) else {
            return false;
        };
        let start = self.point;
        self.insert_text(&text);
        self.last_yank = Some((start, self.point));
        true
    }

    pub fn yank_pop(&mut self) -> bool {
        let Some((start, end)) = self.last_yank else {
            return false;
        };
        if self.kill_ring.len() < 2 {
            return false;
        }
        self.kill_ring.rotate();
        let Some(text) = self.kill_ring.current().map(str::to_string) else {
            return false;
        };
        self.replace_range(start, end, &text);
        self.last_yank = Some((start, self.point));
        true
    }

    // ---------------------------------------------------------- transforms

    pub fn transpose_chars(&mut self) -> bool {
        let row = self.point.row;
        let len = self.lines[row].len();
        if len < 2 {
            return false;
        }
        let col = self.point.col;
        let old = self.lines[row].clone();
        if col == 0 {
            return false;
        } else if col >= len {
            self.lines[row].swap(len - 2, len - 1);
            self.point.col = len;
        } else {
            self.lines[row].swap(col - 1, col);
            self.point.col = col + 1;
        }
        self.record(row, vec![old], 1);
        self.goal_col = None;
        true
    }

    /// Apply a case operation to the active region, or to the word after point.
    pub fn transform_case(&mut self, op: CaseOp) -> bool {
        let (start, end) = if let Some(r) = self.region() {
            r
        } else {
            (self.point, self.word_forward_point())
        };
        if start == end {
            return false;
        }
        let text = self.text_between(start, end);
        let transformed = apply_case(&text, op);
        self.replace_range(start, end, &transformed);
        self.mark = None;
        true
    }

    pub fn delete_trailing_whitespace(&mut self) -> usize {
        let mut start = None;
        let mut end = 0;
        for (i, line) in self.lines.iter().enumerate() {
            if matches!(line.last(), Some(' ') | Some('\t')) {
                start.get_or_insert(i);
                end = i;
            }
        }
        let Some(start) = start else {
            return 0;
        };
        let old = self.lines[start..=end].to_vec();
        let mut removed = 0;
        for line in &mut self.lines[start..=end] {
            while matches!(line.last(), Some(' ') | Some('\t')) {
                line.pop();
                removed += 1;
            }
        }
        self.record(start, old, end - start + 1);
        self.goal_col = None;
        removed
    }

    /// Number of lines that contain only whitespace or comments.
    pub fn is_effectively_empty(&self, comment_prefix: &str) -> bool {
        self.lines.iter().all(|line| {
            let trimmed: String = line
                .iter()
                .skip_while(|c| c.is_whitespace())
                .copied()
                .collect();
            trimmed.is_empty() || trimmed.starts_with(comment_prefix)
        })
    }
}

fn clamp_point(p: Point, lines: &[Line]) -> Point {
    let row = p.row.min(lines.len().saturating_sub(1));
    Point::new(row, p.col.min(lines[row].len()))
}

pub fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn apply_case(text: &str, op: CaseOp) -> String {
    match op {
        CaseOp::Upcase => text.chars().flat_map(char::to_uppercase).collect(),
        CaseOp::Downcase => text.chars().flat_map(char::to_lowercase).collect(),
        CaseOp::Capitalize => {
            let mut out = String::with_capacity(text.len());
            let mut boundary = true;
            for c in text.chars() {
                if is_word_char(c) {
                    if boundary {
                        out.extend(c.to_uppercase());
                    } else {
                        out.extend(c.to_lowercase());
                    }
                    boundary = false;
                } else {
                    boundary = true;
                    out.push(c);
                }
            }
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer(text: &str) -> Buffer {
        Buffer::from_text(text, None)
    }

    fn type_text(buf: &mut Buffer, text: &str) {
        buf.begin_command();
        for c in text.chars() {
            buf.insert_char(c);
        }
        buf.end_command(Some(Coalesce::Insert));
    }

    #[test]
    fn roundtrip_keeps_trailing_newline() {
        let buf = buffer("a\nb\n");
        assert_eq!(buf.text(), "a\nb\n");
        assert_eq!(buf.lines.len(), 3);
    }

    #[test]
    fn crlf_is_detected_and_preserved() {
        let mut buf = buffer("a\r\nb\r\n");
        assert_eq!(buf.line_ending, LineEnding::Crlf);
        assert_eq!(buf.render_text(), "a\r\nb\r\n");
        buf.save_as(Path::new("/tmp/emacette-crlf-test.txt"))
            .unwrap();
        let raw = std::fs::read("/tmp/emacette-crlf-test.txt").unwrap();
        assert_eq!(raw, b"a\r\nb\r\n");
        let _ = std::fs::remove_file("/tmp/emacette-crlf-test.txt");
    }

    #[test]
    fn typing_and_undo() {
        let mut buf = buffer("");
        type_text(&mut buf, "hello");
        assert_eq!(buf.text(), "hello");
        assert!(buf.undo());
        assert_eq!(buf.text(), "");
        assert!(buf.redo());
        assert_eq!(buf.text(), "hello");
    }

    #[test]
    fn coalesced_inserts_are_one_undo_step() {
        let mut buf = buffer("");
        type_text(&mut buf, "abc");
        assert_eq!(buf.undo_depth(), 1);
        buf.undo();
        assert_eq!(buf.text(), "");
    }

    #[test]
    fn newline_auto_indent() {
        let mut buf = buffer("    foo");
        buf.point = Point::new(0, 7);
        buf.newline(true);
        assert_eq!(buf.text(), "    foo\n    ");
        assert_eq!(buf.point, Point::new(1, 4));
    }

    #[test]
    fn backspace_joins_lines() {
        let mut buf = buffer("ab\ncd");
        buf.point = Point::new(1, 0);
        assert!(buf.delete_backward());
        assert_eq!(buf.text(), "abcd");
        assert_eq!(buf.point, Point::new(0, 2));
    }

    #[test]
    fn delete_range_across_lines() {
        let mut buf = buffer("one\ntwo\nthree");
        let removed = buf.delete_range(Point::new(0, 1), Point::new(2, 2));
        assert_eq!(removed, "ne\ntwo\nth");
        assert_eq!(buf.text(), "oree");
        assert_eq!(buf.point, Point::new(0, 1));
    }

    #[test]
    fn kill_and_yank() {
        let mut buf = buffer("hello world");
        buf.point = Point::new(0, 5);
        buf.kill_line();
        assert_eq!(buf.text(), "hello");
        buf.point = Point::new(0, 0);
        assert!(buf.yank());
        assert_eq!(buf.text(), " worldhello");
    }

    #[test]
    fn word_motion_handles_japanese() {
        let mut buf = buffer("これは テストです foo");
        buf.forward_word();
        assert_eq!(buf.point, Point::new(0, 3));
        buf.forward_word();
        assert_eq!(buf.point, Point::new(0, 9));
        buf.backward_word();
        assert_eq!(buf.point, Point::new(0, 4));
    }

    #[test]
    fn word_motion_crosses_lines() {
        let mut buf = buffer("foo bar\nbaz");
        buf.forward_word();
        buf.forward_word();
        assert_eq!(buf.point, Point::new(0, 7));
        buf.forward_word();
        assert_eq!(buf.point, Point::new(1, 3));
    }

    #[test]
    fn region_kill_and_copy() {
        let mut buf = buffer("abcdef");
        buf.point = Point::new(0, 1);
        buf.set_mark();
        buf.point = Point::new(0, 4);
        assert_eq!(buf.copy_region(), "bcd");
        assert_eq!(buf.kill_region(), "bcd");
        assert_eq!(buf.text(), "aef");
    }

    #[test]
    fn yank_pop_cycles_kill_ring() {
        let mut buf = buffer("");
        buf.kill_ring.push("one");
        buf.kill_ring.push("two");
        assert!(buf.yank());
        assert_eq!(buf.text(), "two");
        assert!(buf.yank_pop());
        assert_eq!(buf.text(), "one");
    }

    #[test]
    fn transpose_chars_works() {
        let mut buf = buffer("ab");
        buf.point = Point::new(0, 1);
        assert!(buf.transpose_chars());
        assert_eq!(buf.text(), "ba");
        assert_eq!(buf.point, Point::new(0, 2));
    }

    #[test]
    fn trailing_whitespace_is_removed() {
        let mut buf = buffer("a  \nb\t\nc");
        assert_eq!(buf.delete_trailing_whitespace(), 3);
        assert_eq!(buf.text(), "a\nb\nc");
    }

    #[test]
    fn insert_multiline_text() {
        let mut buf = buffer("ac");
        buf.point = Point::new(0, 1);
        buf.insert_text("X\nY\nZ");
        assert_eq!(buf.text(), "aX\nY\nZc");
        assert_eq!(buf.point, Point::new(2, 1));
    }

    #[test]
    fn effectively_empty_ignores_comments() {
        let buf = buffer("# comment\n\n   \n# another\n");
        assert!(buf.is_effectively_empty("#"));
        let buf = buffer("# comment\nsubject\n");
        assert!(!buf.is_effectively_empty("#"));
    }

    #[test]
    fn undo_all_then_redo_all_round_trips() {
        // Deterministic pseudo-random editing, then undo everything and
        // redo everything.  This exercises the patch bookkeeping across
        // line splits and joins.
        let original = "alpha\nbeta\ngamma\n";
        let mut buf = Buffer::from_text(original, None);
        let mut seed: u64 = 0x2545_F491_4F6C_DD1D;
        let mut next = || {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (seed >> 33) as usize
        };
        for _ in 0..300 {
            buf.begin_command();
            match next() % 8 {
                0 | 1 => buf.insert_char((b'a' + (next() % 26) as u8) as char),
                2 => buf.newline(next() % 2 == 0),
                3 => {
                    buf.delete_backward();
                }
                4 => {
                    buf.delete_forward();
                }
                5 => {
                    buf.kill_line();
                }
                6 => buf.insert_text("x\ny"),
                _ => {
                    let row = next() % buf.line_count();
                    let col = next() % (buf.line_len(row) + 1);
                    buf.set_point(Point::new(row, col));
                }
            }
            buf.end_command(None);
        }
        let final_text = buf.text();
        while buf.undo() {}
        assert_eq!(buf.text(), original);
        while buf.redo() {}
        assert_eq!(buf.text(), final_text);
    }

    #[test]
    fn undo_after_multiline_insert() {
        let mut buf = buffer("ac");
        buf.begin_command();
        buf.point = Point::new(0, 1);
        buf.insert_text("X\nY");
        buf.end_command(None);
        assert_eq!(buf.text(), "aX\nYc");
        buf.undo();
        assert_eq!(buf.text(), "ac");
        buf.redo();
        assert_eq!(buf.text(), "aX\nYc");
    }
}
