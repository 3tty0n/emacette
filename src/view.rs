//! Screen geometry: display widths for tabs / wide characters and scrolling.

use unicode_width::UnicodeWidthChar;

use crate::buffer::Buffer;

/// Width of a character in a terminal cell (tabs handled separately).
pub fn cell_width(c: char) -> usize {
    if c.is_control() {
        1
    } else {
        c.width().unwrap_or(0)
    }
}

pub fn char_width(c: char, col: usize, tab_width: usize) -> usize {
    if c == '\t' {
        tab_width - (col % tab_width)
    } else {
        cell_width(c)
    }
}

/// Display width of a whole line.
pub fn display_width(line: &[char], tab_width: usize) -> usize {
    let mut col = 0;
    for &c in line {
        col += char_width(c, col, tab_width);
    }
    col
}

/// Display column of a character offset within a line.
pub fn display_col(line: &[char], char_col: usize, tab_width: usize) -> usize {
    let mut col = 0;
    for &c in line.iter().take(char_col) {
        col += char_width(c, col, tab_width);
    }
    col
}

/// End of one wrapped screen row.  Returns `(end_index, end_display_col)`.
/// `start_display` is the absolute display column of `start_idx`, so tabs
/// keep expanding to their real stops after a wrap.
pub(crate) fn wrap_end(
    line: &[char],
    start_idx: usize,
    start_display: usize,
    width: usize,
    tab_width: usize,
) -> (usize, usize) {
    let n = line.len();
    let mut idx = start_idx.min(n);
    let mut col = start_display;
    let width = width.max(1);
    while idx < n {
        let w = char_width(line[idx], col, tab_width);
        if w > 0 && col - start_display + w > width {
            break;
        }
        col += w;
        idx += 1;
    }
    if idx == start_idx && idx < n {
        // A single character wider than the screen: keep it so callers make
        // progress even on absurdly narrow terminals.
        col += char_width(line[idx], col, tab_width);
        idx += 1;
    }
    (idx, col)
}

/// Locate the wrapped segment containing `char_col`.
///
/// Returns `(segment_index, segment_start_idx, segment_start_display,
/// segment_end_idx)`.
pub(crate) fn cursor_segment(
    line: &[char],
    char_col: usize,
    width: usize,
    tab_width: usize,
) -> (usize, usize, usize, usize) {
    let col = char_col.min(line.len());
    let mut segment = 0;
    let mut start_idx = 0;
    let mut start_display = 0;
    loop {
        let (end, end_display) = wrap_end(line, start_idx, start_display, width, tab_width);
        if col < end || end >= line.len() {
            return (segment, start_idx, start_display, end);
        }
        segment += 1;
        start_idx = end;
        start_display = end_display;
    }
}

/// Start char index and absolute display column of a wrapped segment.
pub(crate) fn segment_start(
    line: &[char],
    segment: usize,
    width: usize,
    tab_width: usize,
) -> (usize, usize) {
    let mut start_idx = 0;
    let mut start_display = 0;
    for _ in 0..segment {
        if start_idx >= line.len() {
            break;
        }
        let (end, end_display) = wrap_end(line, start_idx, start_display, width, tab_width);
        if end == start_idx {
            break;
        }
        start_idx = end;
        start_display = end_display;
    }
    (start_idx, start_display)
}

/// Advance one wrapped screen row.  The caller keeps `start_idx` and
/// `start_display` in sync so scanning a very long line stays linear.
pub(crate) fn advance_screen_row(
    line: &[char],
    row: &mut usize,
    offset: &mut usize,
    start_idx: &mut usize,
    start_display: &mut usize,
    width: usize,
    tab_width: usize,
) {
    let (end, end_display) = wrap_end(line, *start_idx, *start_display, width, tab_width);
    if end >= line.len() {
        *row += 1;
        *offset = 0;
        *start_idx = 0;
        *start_display = 0;
    } else {
        *offset += 1;
        *start_idx = end;
        *start_display = end_display;
    }
}

/// Width reserved for the line-number gutter (including its trailing space).
pub fn gutter_width(line_count: usize, enabled: bool) -> usize {
    if !enabled {
        return 0;
    }
    let digits = line_count.max(1).to_string().len();
    digits + 2
}

pub struct View {
    pub width: usize,
    pub height: usize,
    /// First visible buffer row.
    pub top: usize,
    /// First visible wrapped segment within `top` (only used when wrapping).
    pub top_offset: usize,
    /// Horizontal scroll in display cells (only used when truncating).
    pub left: usize,
    pub line_numbers: bool,
    pub tab_width: usize,
    pub highlight_trailing_ws: bool,
    pub truncate_lines: bool,
}

impl Default for View {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
            top: 0,
            top_offset: 0,
            left: 0,
            line_numbers: true,
            tab_width: 4,
            highlight_trailing_ws: true,
            truncate_lines: true,
        }
    }
}

impl View {
    /// Rows available for text (the last two rows are the mode line and echo area).
    pub fn body_height(&self) -> usize {
        self.height.saturating_sub(2).max(1)
    }

    pub fn gutter(&self, line_count: usize) -> usize {
        gutter_width(line_count, self.line_numbers)
    }

    pub fn body_width(&self, line_count: usize) -> usize {
        self.width.saturating_sub(self.gutter(line_count)).max(1)
    }

    pub fn ensure_visible(&mut self, buffer: &Buffer) {
        if self.truncate_lines {
            self.ensure_visible_truncated(buffer);
        } else {
            self.ensure_visible_wrapped(buffer);
        }
    }

    fn ensure_visible_truncated(&mut self, buffer: &Buffer) {
        self.top_offset = 0;
        let body = self.body_height();
        let point = buffer.point;

        if point.row < self.top {
            self.top = point.row;
        }
        if body > 0 && point.row >= self.top + body {
            self.top = point.row + 1 - body;
        }
        let max_top = buffer.line_count().saturating_sub(1);
        if self.top > max_top {
            self.top = max_top;
        }

        let body_width = self.body_width(buffer.line_count());
        let line = &buffer.lines[point.row];
        let col = point.col.min(line.len());
        let cursor_col = display_col(line, col, self.tab_width);

        let w = if col < line.len() {
            char_width(line[col], cursor_col, self.tab_width).max(1)
        } else {
            1
        };
        if col == 0 {
            self.left = 0;
        } else if cursor_col < self.left {
            self.left = cursor_col;
        }
        if cursor_col + w > self.left + body_width {
            self.left = cursor_col + w - body_width;
        }
        // If the beginning of the line up to the cursor fits on screen there
        // is no reason to stay horizontally scrolled.  This also resets the
        // scroll after moving from a long line to a short one, without having
        // to measure the whole line.
        if cursor_col + w <= body_width {
            self.left = 0;
        }
    }

    fn ensure_visible_wrapped(&mut self, buffer: &Buffer) {
        let body = self.body_height();
        let body_width = self.body_width(buffer.line_count());
        let line_count = buffer.line_count();
        let point = buffer.point;
        let point_line = &buffer.lines[point.row];
        let col = point.col.min(point_line.len());
        let (point_seg, _, _, _) = cursor_segment(point_line, col, body_width, self.tab_width);

        if point.row < self.top || (point.row == self.top && point_seg < self.top_offset) {
            self.top = point.row;
            self.top_offset = point_seg;
        }
        if self.top >= line_count {
            self.top = line_count.saturating_sub(1);
            self.top_offset = 0;
        }
        // A resize or an edit can leave top_offset past the last wrapped
        // segment of the top row; start that row from the beginning instead
        // of showing an empty continuation row.
        if self.top_offset > 0 {
            let line = &buffer.lines[self.top];
            let (start_idx, _) = segment_start(line, self.top_offset, body_width, self.tab_width);
            if start_idx >= line.len() {
                self.top_offset = 0;
            }
        }
        // Each buffer row needs at least one screen row, so a point this far
        // below cannot be on screen.  Jump near it before counting wrapped
        // rows, which keeps movement through huge files cheap.
        if point.row >= self.top.saturating_add(body) {
            self.top = point.row.saturating_sub(body.saturating_sub(1));
            self.top_offset = 0;
        }

        let mut row = self.top;
        let mut offset = self.top_offset;
        let (mut start_idx, mut start_display) =
            segment_start(&buffer.lines[row], offset, body_width, self.tab_width);
        let mut before = 0usize;
        while row < point.row || (row == point.row && offset < point_seg) {
            if row >= line_count {
                break;
            }
            advance_screen_row(
                &buffer.lines[row],
                &mut row,
                &mut offset,
                &mut start_idx,
                &mut start_display,
                body_width,
                self.tab_width,
            );
            before += 1;
        }
        if before >= body {
            let shift = before - body + 1;
            let mut row = self.top;
            let mut offset = self.top_offset;
            let (mut start_idx, mut start_display) =
                segment_start(&buffer.lines[row], offset, body_width, self.tab_width);
            for _ in 0..shift {
                if row >= line_count {
                    break;
                }
                advance_screen_row(
                    &buffer.lines[row],
                    &mut row,
                    &mut offset,
                    &mut start_idx,
                    &mut start_display,
                    body_width,
                    self.tab_width,
                );
            }
            self.top = row;
            self.top_offset = offset;
        }
        if self.top >= line_count {
            self.top = line_count.saturating_sub(1);
            self.top_offset = 0;
        }
        self.left = 0;
    }

    pub fn recenter(&mut self, buffer: &Buffer) {
        let body = self.body_height();
        self.top = buffer.point.row.saturating_sub(body / 2);
        self.top_offset = 0;
        let max_top = buffer.line_count().saturating_sub(1);
        if self.top > max_top {
            self.top = max_top;
        }
        self.ensure_visible(buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabs_expand_to_stops() {
        let line: Vec<char> = "\tx".chars().collect();
        assert_eq!(display_width(&line, 4), 5);
        assert_eq!(display_col(&line, 1, 4), 4);
        let line: Vec<char> = "a\tb".chars().collect();
        assert_eq!(display_col(&line, 2, 4), 4);
    }

    #[test]
    fn wide_characters_are_two_cells() {
        let line: Vec<char> = "日本語".chars().collect();
        assert_eq!(display_width(&line, 4), 6);
        assert_eq!(display_col(&line, 2, 4), 4);
    }

    #[test]
    fn view_scrolls_to_cursor() {
        let text: String = (0..100).map(|i| format!("line {i}\n")).collect();
        let buffer = Buffer::from_text(&text, None);
        let mut view = View {
            width: 40,
            height: 10,
            ..View::default()
        };
        let mut buffer = buffer;
        buffer.point = crate::buffer::Point::new(50, 0);
        view.ensure_visible(&buffer);
        assert!(view.top <= 50 && view.top + view.body_height() > 50);
    }

    #[test]
    fn wrapping_splits_at_display_width() {
        let line: Vec<char> = "abcdefghij".chars().collect();
        assert_eq!(wrap_end(&line, 0, 0, 4, 4), (4, 4));
        assert_eq!(wrap_end(&line, 4, 4, 4, 4), (8, 8));
        assert_eq!(wrap_end(&line, 8, 8, 4, 4), (10, 10));

        let line: Vec<char> = "あああ".chars().collect();
        assert_eq!(wrap_end(&line, 0, 0, 4, 4), (2, 4));
    }

    #[test]
    fn cursor_segment_maps_columns_to_rows() {
        let line: Vec<char> = "abcdefghij".chars().collect();
        assert_eq!(cursor_segment(&line, 0, 4, 4), (0, 0, 0, 4));
        assert_eq!(cursor_segment(&line, 3, 4, 4), (0, 0, 0, 4));
        assert_eq!(cursor_segment(&line, 4, 4, 4), (1, 4, 4, 8));
        assert_eq!(cursor_segment(&line, 10, 4, 4), (2, 8, 8, 10));
    }

    #[test]
    fn wrapped_view_keeps_cursor_visible() {
        let text = "x".repeat(200);
        let mut buffer = Buffer::from_text(&text, None);
        buffer.point = crate::buffer::Point::new(0, 200);
        let mut view = View {
            width: 20,
            height: 8,
            line_numbers: false,
            truncate_lines: false,
            ..View::default()
        };
        view.ensure_visible(&buffer);
        assert_eq!(view.left, 0);
        assert!(view.top_offset > 0);
    }

    #[test]
    fn wrapped_view_scrolls_between_lines() {
        let text = format!("{}\nsecond\n", "x".repeat(100));
        let mut buffer = Buffer::from_text(&text, None);
        buffer.point = crate::buffer::Point::new(1, 0);
        let mut view = View {
            width: 20,
            height: 6,
            line_numbers: false,
            truncate_lines: false,
            ..View::default()
        };
        view.ensure_visible(&buffer);
        assert_eq!(view.left, 0);
        assert!(view.top > 0 || view.top_offset > 0);
    }
}
