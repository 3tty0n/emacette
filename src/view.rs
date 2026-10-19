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
    pub top: usize,
    pub left: usize,
    pub line_numbers: bool,
    pub tab_width: usize,
    pub highlight_trailing_ws: bool,
}

impl Default for View {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
            top: 0,
            left: 0,
            line_numbers: true,
            tab_width: 4,
            highlight_trailing_ws: true,
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

    pub fn recenter(&mut self, buffer: &Buffer) {
        let body = self.body_height();
        self.top = buffer.point.row.saturating_sub(body / 2);
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
}
