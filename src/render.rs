//! Rendering: turns editor state into ANSI escape sequences.
//!
//! Frames are rendered incrementally.  The previous frame's rows are kept and
//! only rows whose contents actually changed are written to the terminal, so
//! moving the cursor does not repaint the whole screen.  This matters a lot
//! over SSH or inside tmux.

use std::fmt::Write as _;

use crate::editor::Editor;
use crate::help::{HelpLine, HELP};
use crate::syntax::Span;
use crate::theme::{Style, Theme};
use crate::view::{cell_width, char_width, display_col};

pub struct Frame {
    pub output: String,
    pub cursor: Option<(u16, u16)>,
}

/// Stateful renderer that remembers the previous frame.
pub struct Renderer {
    rows: Vec<String>,
    width: usize,
    height: usize,
    cursor: Option<(u16, u16)>,
    cursor_visible: bool,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            width: 0,
            height: 0,
            cursor: None,
            cursor_visible: false,
        }
    }

    /// Force the next frame to repaint every row (e.g. after C-l).
    pub fn invalidate(&mut self) {
        self.rows.clear();
    }

    pub fn render(&mut self, editor: &mut Editor, width: usize, height: usize) -> Frame {
        editor.view.width = width.max(1);
        editor.view.height = height.max(1);
        editor.view.tab_width = editor.config.tab_width;
        editor.view.line_numbers = editor.config.line_numbers;
        editor.view.highlight_trailing_ws = editor.config.highlight_trailing_ws;
        editor.sync_highlighter();
        editor.view.ensure_visible(&editor.buffer);

        if editor.needs_redraw {
            self.rows.clear();
            editor.needs_redraw = false;
        }

        let terminal_height = editor.view.height;
        // The bottom two rows are the mode line and the echo area.  On a
        // terminal with fewer than three rows there is no text area.
        let body_rows = terminal_height.saturating_sub(2);

        let mut rows: Vec<String> = Vec::with_capacity(terminal_height);
        let mut cursor = None;
        if let Some(overlay) = editor.overlay {
            rows.extend(build_help_rows(editor, body_rows, overlay.scroll));
        } else {
            rows.extend(build_text_rows(editor, body_rows));
            cursor =
                buffer_cursor(editor).filter(|&(_, y)| body_rows > 0 && y as usize <= body_rows);
        }
        if terminal_height >= 2 {
            rows.push(build_modeline(editor));
        }
        let (echo, echo_cursor) = build_echo(editor);
        rows.push(echo);
        if echo_cursor.is_some() {
            cursor = echo_cursor;
        }

        let size_changed =
            self.width != width || self.height != height || self.rows.len() != rows.len();
        let full = self.rows.is_empty() || size_changed;
        let mut out = String::with_capacity(if full { 1024 } else { 256 });
        let mut dirty = false;

        if full {
            if !self.rows.is_empty() {
                // Resize or forced redraw: clear any leftovers first.
                out.push_str("\x1b[2J");
            }
            // Hide the cursor and ask capable terminals to apply the frame
            // atomically (DEC private mode 2026, "synchronized update").
            out.push_str("\x1b[?25l\x1b[?2026h");
            for (index, row) in rows.iter().enumerate() {
                let _ = write!(out, "\x1b[{};1H", index + 1);
                out.push_str(row);
                out.push_str("\x1b[0m\x1b[K");
            }
            out.push_str("\x1b[0m\x1b[0J");
            dirty = true;
        } else {
            for (index, row) in rows.iter().enumerate() {
                if *row == self.rows[index] {
                    continue;
                }
                if !dirty {
                    out.push_str("\x1b[?25l\x1b[?2026h");
                    dirty = true;
                }
                let _ = write!(out, "\x1b[{};1H", index + 1);
                out.push_str(row);
                out.push_str("\x1b[0m\x1b[K");
            }
        }

        let cursor_moved = cursor != self.cursor;
        if let Some((x, y)) = cursor {
            if dirty || cursor_moved || !self.cursor_visible {
                let _ = write!(out, "\x1b[{};{}H", y, x);
            }
            if dirty || !self.cursor_visible {
                out.push_str("\x1b[?25h");
            }
        } else if self.cursor_visible {
            out.push_str("\x1b[?25l");
        }
        if dirty {
            out.push_str("\x1b[?2026l");
        }

        self.rows = rows;
        self.width = width;
        self.height = height;
        self.cursor = cursor;
        self.cursor_visible = cursor.is_some();

        Frame {
            output: out,
            cursor,
        }
    }
}

/// Convenience wrapper for tests and one-shot rendering.
pub fn render(editor: &mut Editor, width: usize, height: usize) -> Frame {
    Renderer::new().render(editor, width, height)
}

fn push_style(out: &mut String, style: Style) {
    out.push_str("\x1b[0");
    if style.bold {
        out.push_str(";1");
    }
    if style.italic {
        out.push_str(";3");
    }
    if style.underline {
        out.push_str(";4");
    }
    if let Some(fg) = style.fg {
        let _ = write!(out, ";38;5;{fg}");
    }
    if let Some(bg) = style.bg {
        let _ = write!(out, ";48;5;{bg}");
    }
    out.push('m');
}

fn build_text_rows(editor: &mut Editor, body_rows: usize) -> Vec<String> {
    let line_count = editor.buffer.line_count();
    let gutter = editor.view.gutter(line_count);
    let body_width = editor.view.body_width(line_count);
    let digits = line_count.max(1).to_string().len();
    let top = editor.view.top;
    let current_row = editor.buffer.point.row;
    let left = editor.view.left;
    let tab_width = editor.view.tab_width;
    let theme = &editor.theme;
    let highlight_trailing = editor.view.highlight_trailing_ws;
    let mut rows = Vec::with_capacity(body_rows);

    for screen_row in 0..body_rows {
        let row = top + screen_row;
        let mut line = String::new();
        if row >= line_count {
            line.push_str(&" ".repeat(gutter));
            rows.push(line);
            continue;
        }

        if gutter > 0 {
            let number = format!(" {:>width$} ", row + 1, width = digits);
            let style = if row == current_row {
                theme.gutter_current
            } else {
                theme.gutter
            };
            push_style(&mut line, style);
            line.push_str(&number);
        }

        let spans: Vec<Span> = {
            let lines = &editor.buffer.lines;
            editor.highlighter.tokens(row, lines).to_vec()
        };
        let chars = &editor.buffer.lines[row];

        let region = editor.buffer.region().and_then(|(start, end)| {
            if row < start.row || row > end.row {
                return None;
            }
            let from = if row == start.row { start.col } else { 0 };
            let to = if row == end.row {
                end.col
            } else {
                chars.len() + 1
            };
            Some((from, to))
        });

        let search = editor.search.as_ref().and_then(|search| {
            let current = search.current?;
            if current.row != row {
                return None;
            }
            Some((current.col, current.col + search.query.len()))
        });

        draw_line(
            &mut line,
            chars,
            &spans,
            left,
            body_width,
            tab_width,
            theme,
            region,
            search,
            highlight_trailing,
        );
        rows.push(line);
    }
    rows
}

#[allow(clippy::too_many_arguments)]
fn draw_line(
    out: &mut String,
    chars: &[char],
    spans: &[Span],
    left: usize,
    body_width: usize,
    tab_width: usize,
    theme: &Theme,
    region: Option<(usize, usize)>,
    search: Option<(usize, usize)>,
    highlight_trailing: bool,
) {
    let mut idx = 0usize;
    let mut col = 0usize;
    while idx < chars.len() {
        let w = char_width(chars[idx], col, tab_width);
        if col + w > left {
            break;
        }
        col += w;
        idx += 1;
    }
    if col < left {
        // A wide character straddles the left edge: pad and skip it.
        out.push_str(&" ".repeat(left - col));
        idx += 1;
        col = left;
    }

    let trailing_from = chars
        .iter()
        .rposition(|c| *c != ' ' && *c != '\t')
        .map(|i| i + 1)
        .unwrap_or(0);

    let mut emitted = 0usize;
    let mut span_idx = 0usize;
    let mut current = theme.text;
    let mut styled = false;

    while idx < chars.len() {
        let c = chars[idx];
        let w = char_width(c, col, tab_width);
        if w > 0 && emitted + w > body_width {
            break;
        }

        while span_idx < spans.len() && spans[span_idx].end <= idx {
            span_idx += 1;
        }
        let mut style = match spans.get(span_idx) {
            Some(span) if span.start <= idx => theme.tok(span.kind),
            _ => theme.text,
        };

        if let Some((from, to)) = search {
            if idx >= from && idx < to {
                style = theme.search;
            }
        } else if let Some((from, to)) = region {
            if idx >= from && idx < to {
                if let Some(bg) = theme.region.bg {
                    style = style.with_bg(bg);
                }
            }
        }

        if highlight_trailing && idx >= trailing_from && trailing_from < chars.len() {
            if let Some(bg) = theme.trailing_ws.bg {
                style = style.with_bg(bg);
            }
        }

        if style != current || !styled {
            push_style(out, style);
            current = style;
            styled = true;
        }

        if c == '\t' {
            for _ in 0..w {
                out.push(' ');
            }
        } else if c.is_control() {
            out.push('?');
        } else {
            out.push(c);
        }

        emitted += w;
        col += w;
        idx += 1;
    }
}

fn buffer_cursor(editor: &Editor) -> Option<(u16, u16)> {
    let point = editor.buffer.point;
    let body_height = editor.view.body_height();
    if point.row < editor.view.top || point.row >= editor.view.top + body_height {
        return None;
    }
    let line = &editor.buffer.lines[point.row];
    let col = point.col.min(line.len());
    let cursor_col = display_col(line, col, editor.view.tab_width);
    let gutter = editor.view.gutter(editor.buffer.line_count());
    let x = gutter as isize + cursor_col as isize - editor.view.left as isize + 1;
    if x < 1 || x > editor.view.width as isize {
        return None;
    }
    Some((x as u16, (point.row - editor.view.top + 1) as u16))
}

fn build_modeline(editor: &Editor) -> String {
    let width = editor.view.width;
    let name = editor.buffer.display_name();
    let star = if editor.buffer.modified { "**" } else { "--" };
    let point = editor.buffer.point;
    let position = format!("L{} C{}", point.row + 1, point.col + 1);

    let body = editor.view.body_height();
    let line_count = editor.buffer.line_count();
    let percent = if editor.view.top == 0 && body >= line_count {
        "All".to_string()
    } else if editor.view.top + body >= line_count {
        "Bot".to_string()
    } else {
        format!("{}%", editor.view.top * 100 / line_count.max(1))
    };
    let mode = editor.mode.name();
    let eol = editor.buffer.line_ending.name();

    let full_left = format!(" my-emacs  {name} {star}  {position}");
    let full_right = format!("{percent}  {mode}  {eol} ");
    let line = if fits(&full_left, &full_right, width) {
        layout_modeline(&full_left, &full_right, width)
    } else {
        // Narrow terminal: keep the file name and the cursor position,
        // dropping the less important pieces first.
        let compact_left = format!(" {name} {star}  {position}");
        let compact_right = format!("{percent}  {mode} ");
        if fits(&compact_left, &compact_right, width) {
            layout_modeline(&compact_left, &compact_right, width)
        } else {
            let minimal_left = format!(" {name} {position}");
            let minimal_right = format!(" {mode} ");
            if fits(&minimal_left, &minimal_right, width) {
                layout_modeline(&minimal_left, &minimal_right, width)
            } else if fits(&minimal_left, "", width) {
                layout_modeline(&minimal_left, "", width)
            } else {
                let tiny = format!(" {position} {mode}");
                layout_modeline(&tiny, "", width)
            }
        }
    };

    let mut row = String::with_capacity(line.len() + 8);
    push_style(&mut row, editor.theme.modeline);
    row.push_str(&line);
    row
}

fn fits(left: &str, right: &str, width: usize) -> bool {
    str_width(left) + str_width(right) < width
}

/// Lay out `left` and `right` on one row of `width` cells, padding or
/// truncating as needed.
fn layout_modeline(left: &str, right: &str, width: usize) -> String {
    let left_width = str_width(left);
    let right_width = str_width(right);
    if left_width + right_width < width {
        return format!(
            "{left}{}{right}",
            " ".repeat(width - left_width - right_width)
        );
    }
    if right_width + 2 < width {
        let keep = width - right_width - 1;
        let head = truncate_to_width(left, keep);
        let pad = keep - str_width(&head);
        return format!("{head}{} {right}", " ".repeat(pad));
    }
    truncate_to_width(left, width)
}

fn build_echo(editor: &Editor) -> (String, Option<(u16, u16)>) {
    let width = editor.view.width;
    let theme = &editor.theme;
    let mut row = String::new();

    if let Some(mb) = &editor.minibuffer {
        let prompt = &mb.prompt;
        let input = mb.text();
        let before: String = mb.input[..mb.cursor.min(mb.input.len())].iter().collect();
        let cursor_offset = str_width(prompt) + str_width(&before);
        let mut segments: Vec<(String, Style)> = vec![
            (prompt.clone(), theme.prompt),
            (input.clone(), theme.minibuffer),
        ];
        if !mb.hint.is_empty() {
            segments.push(("  ".to_string(), theme.minibuffer));
            segments.push((
                mb.hint.clone(),
                theme.help_dim.with_bg(theme.minibuffer.bg.unwrap_or(0)),
            ));
        }
        let skip = (cursor_offset + 1).saturating_sub(width);
        push_style(&mut row, theme.minibuffer);
        let written = draw_segments(&mut row, &segments, skip, width);
        if written < width {
            push_style(&mut row, theme.minibuffer);
            row.push_str(&" ".repeat(width - written));
        }
        let x = cursor_offset.saturating_sub(skip) + 1;
        return (
            row,
            Some(((x.min(width).max(1)) as u16, editor.view.height as u16)),
        );
    }

    let (text, style) = if let Some(search) = &editor.search {
        let query: String = search.query.iter().collect();
        let status = if search.query.is_empty() {
            "  (C-s forward, C-r backward, RET accept, C-g abort)".to_string()
        } else if search.current.is_none() {
            "  [no match]".to_string()
        } else if search.wrapped {
            "  [wrapped]".to_string()
        } else {
            String::new()
        };
        (format!("I-search: {query}{status}"), theme.prompt)
    } else if let Some(question) = editor.prompt_for_quit() {
        (question.to_string(), theme.prompt)
    } else if !editor.message.is_empty() {
        (editor.message.clone(), theme.text)
    } else {
        (editor.hint(), theme.help_dim)
    };

    let line = truncate_to_width(&text, width);
    let line_width = str_width(&line);
    push_style(&mut row, theme.minibuffer);
    push_style(&mut row, style);
    row.push_str(&line);
    if line_width < width {
        push_style(&mut row, theme.minibuffer);
        row.push_str(&" ".repeat(width - line_width));
    }
    (row, None)
}

/// Draw `segments` starting at display column `skip`, never exceeding `width`.
fn draw_segments(
    out: &mut String,
    segments: &[(String, Style)],
    skip: usize,
    width: usize,
) -> usize {
    let mut col = 0usize;
    let mut written = 0usize;
    let mut current: Option<Style> = None;
    for (text, style) in segments {
        for c in text.chars() {
            let w = char_width(c, col, 8);
            if col + w <= skip {
                col += w;
                continue;
            }
            if col < skip {
                col += w;
                continue;
            }
            if written + w > width {
                return written;
            }
            if current != Some(*style) {
                push_style(out, *style);
                current = Some(*style);
            }
            if c == '\t' {
                for _ in 0..w {
                    out.push(' ');
                }
            } else if c.is_control() {
                out.push('?');
            } else {
                out.push(c);
            }
            written += w;
            col += w;
        }
    }
    written
}

fn build_help_rows(editor: &Editor, body_rows: usize, scroll: usize) -> Vec<String> {
    let theme = &editor.theme;
    let width = editor.view.width;
    let mut rows = Vec::with_capacity(body_rows);
    for screen_row in 0..body_rows {
        let mut line = String::new();
        if let Some(entry) = HELP.get(scroll + screen_row) {
            match entry {
                HelpLine::Heading(text) => {
                    push_style(&mut line, theme.help_heading);
                    line.push_str(&truncate_to_width(text, width));
                }
                HelpLine::Key(key, description) => {
                    push_style(&mut line, theme.help_key);
                    line.push_str(key);
                    push_style(&mut line, theme.help_dim);
                    let pad = 16usize.saturating_sub(str_width(key));
                    line.push_str(&" ".repeat(pad));
                    line.push_str(&truncate_to_width(
                        description,
                        width.saturating_sub(2 + str_width(key) + pad),
                    ));
                }
                HelpLine::Text(text) => {
                    push_style(&mut line, theme.help_dim);
                    line.push_str(&truncate_to_width(text, width));
                }
                HelpLine::Blank => {}
            }
        }
        rows.push(line);
    }
    rows
}

fn str_width(text: &str) -> usize {
    text.chars().map(cell_width).sum()
}

fn truncate_to_width(text: &str, width: usize) -> String {
    let mut out = String::with_capacity(text.len());
    let mut col = 0usize;
    for c in text.chars() {
        let w = cell_width(c);
        if col + w > width {
            break;
        }
        out.push(c);
        col += w;
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::editor::Editor;
    use crate::key::Key;

    use super::*;

    fn strip_ansi(s: &str) -> String {
        let mut out = String::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                if chars.peek() == Some(&'[') {
                    chars.next();
                    for c in chars.by_ref() {
                        if c.is_ascii_alphabetic() || c == '~' {
                            break;
                        }
                    }
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    #[test]
    fn renders_text_and_modeline() {
        let mut ed = Editor::from_text(
            "hello world\nsecond line",
            Some(std::path::PathBuf::from("/tmp/sample.rs")),
            Config::default(),
        );
        let frame = render(&mut ed, 60, 10);
        let plain = strip_ansi(&frame.output);
        assert!(plain.contains("hello world"));
        assert!(plain.contains("second line"));
        assert!(plain.contains("sample.rs"));
        assert!(plain.contains("Rust"));
        assert!(frame.cursor.is_some());
    }

    #[test]
    fn tiny_terminal_does_not_panic() {
        let mut ed = Editor::from_text("x", None, Config::default());
        for (w, h) in [(1, 1), (2, 2), (3, 1), (10, 2)] {
            let frame = render(&mut ed, w, h);
            assert!(!frame.output.is_empty());
        }
    }

    #[test]
    fn wide_characters_render() {
        let mut ed = Editor::from_text("日本語", None, Config::default());
        ed.buffer.point.col = 3;
        let frame = render(&mut ed, 40, 8);
        assert!(strip_ansi(&frame.output).contains("日本語"));
        let (x, _) = frame.cursor.unwrap();
        assert!(x >= 2);
    }

    #[test]
    fn help_overlay_renders() {
        let mut ed = Editor::from_text("", None, Config::default());
        ed.overlay = Some(crate::editor::Overlay { scroll: 0 });
        let frame = render(&mut ed, 70, 20);
        assert!(strip_ansi(&frame.output).contains("my-emacs"));
        assert!(frame.cursor.is_none());
    }

    #[test]
    fn horizontal_scroll_keeps_cursor_visible() {
        let mut ed = Editor::from_text(&("あ".repeat(80) + "\n"), None, Config::default());
        ed.buffer.point.col = 80;
        ed.view.height = 8;
        let frame = render(&mut ed, 30, 8);
        let (x, _) = frame.cursor.expect("cursor should be visible");
        assert!(x as usize <= 30);
        let plain = strip_ansi(&frame.output);
        assert!(plain.contains("あああ"));
        assert!(!plain.contains("L1 C1 "), "cursor should have moved");
    }

    #[test]
    fn cursor_moves_do_not_redraw_the_text() {
        let mut ed = Editor::from_text("hello\nworld\n", None, Config::default());
        let mut renderer = Renderer::new();
        let first = renderer.render(&mut ed, 40, 8);
        assert!(first.output.contains("hello"));

        ed.handle_key(Key::ctrl('f'));
        let second = renderer.render(&mut ed, 40, 8);
        assert!(!second.output.is_empty());
        assert!(
            !second.output.contains("hello"),
            "body should not be re-sent on cursor motion: {:?}",
            second.output
        );
        assert!(
            second.output.len() < 250,
            "cursor motion should only update the mode line: {} bytes",
            second.output.len()
        );

        // Moving down changes the highlighted line-number gutter, so two
        // rows are repainted but the rest is untouched.
        ed.handle_key(Key::ctrl('n'));
        let third = renderer.render(&mut ed, 40, 8);
        assert!(third.output.contains("world"));

        // Nothing changed at all -> no output.
        let fourth = renderer.render(&mut ed, 40, 8);
        assert!(
            fourth.output.is_empty(),
            "unexpected repaint: {:?}",
            fourth.output
        );
    }

    #[test]
    fn resize_repaints_everything() {
        let mut ed = Editor::from_text("hello\nworld\n", None, Config::default());
        let mut renderer = Renderer::new();
        renderer.render(&mut ed, 40, 8);
        ed.handle_key(Key::ctrl('f'));
        let frame = renderer.render(&mut ed, 50, 10);
        let plain = strip_ansi(&frame.output);
        assert!(plain.contains("hello"));
        assert!(plain.contains("world"));
    }

    #[test]
    fn forced_redraw_sends_the_whole_buffer() {
        let mut ed = Editor::from_text("hello\nworld\n", None, Config::default());
        let mut renderer = Renderer::new();
        renderer.render(&mut ed, 40, 8);
        ed.needs_redraw = true;
        let frame = renderer.render(&mut ed, 40, 8);
        let plain = strip_ansi(&frame.output);
        assert!(plain.contains("hello"));
        assert!(plain.contains("world"));
    }

    #[test]
    fn incremental_frames_match_a_fresh_render() {
        let text: String = (0..60)
            .map(|i| format!("line {i} with some text"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut ed = Editor::from_text(&text, None, Config::default());
        let mut renderer = Renderer::new();
        renderer.render(&mut ed, 60, 12);

        let keys = [
            Key::ctrl('n'),
            Key::ctrl('n'),
            Key::ctrl('n'),
            Key::ctrl('f'),
            Key::ctrl('f'),
            Key::ch('X'),
            Key::code(crate::key::KeyCode::Enter),
            Key::ctrl('a'),
            Key::ctrl('k'),
            Key::ctrl('/'),
            Key::alt('>'),
            Key::alt('<'),
            Key::code(crate::key::KeyCode::PageDown),
            Key::code(crate::key::KeyCode::PageUp),
            Key::code(crate::key::KeyCode::F(1)),
            Key::code(crate::key::KeyCode::Esc),
            Key::ctrl('e'),
        ];
        for key in keys {
            ed.handle_key(key);
            renderer.render(&mut ed, 60, 12);
        }
        let incremental_rows = renderer.rows.clone();
        let mut fresh = Renderer::new();
        fresh.render(&mut ed, 60, 12);
        assert_eq!(incremental_rows, fresh.rows);
    }

    #[test]
    fn cursor_only_updates_are_tiny() {
        let mut ed = Editor::from_text("abcdef", None, Config::default());
        let mut renderer = Renderer::new();
        renderer.render(&mut ed, 40, 8);
        ed.handle_key(Key::ctrl('e'));
        let frame = renderer.render(&mut ed, 40, 8);
        // Mode line + cursor addressing only.
        assert!(frame.output.len() < 200, "{} bytes", frame.output.len());
        assert!(frame.output.contains("C7"));
    }
}
