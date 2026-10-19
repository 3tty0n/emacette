//! Incremental search helpers.

use crate::buffer::{Line, Point};

#[derive(Clone, Debug)]
pub struct SearchState {
    pub query: Vec<char>,
    pub origin: Point,
    pub dir: i32,
    pub current: Option<Point>,
    pub wrapped: bool,
}

pub fn fold_char(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

fn matches_at(line: &[char], start: usize, needle: &[char], fold: bool) -> bool {
    if start + needle.len() > line.len() {
        return false;
    }
    for (offset, n) in needle.iter().enumerate() {
        let c = line[start + offset];
        if fold {
            if fold_char(c) != fold_char(*n) {
                return false;
            }
        } else if c != *n {
            return false;
        }
    }
    true
}

fn find_in_line(line: &[char], needle: &[char], from: usize, fold: bool) -> Option<usize> {
    if needle.is_empty() {
        return None;
    }
    if from + needle.len() > line.len() {
        return None;
    }
    (from..=line.len() - needle.len()).find(|&i| matches_at(line, i, needle, fold))
}

pub fn find_forward(lines: &[Line], needle: &[char], from: Point, fold: bool) -> Option<Point> {
    if needle.is_empty() || lines.is_empty() {
        return None;
    }
    let row = from.row.min(lines.len() - 1);
    for (r, line) in lines.iter().enumerate().skip(row) {
        let start = if r == row { from.col } else { 0 };
        if let Some(col) = find_in_line(line, needle, start, fold) {
            return Some(Point::new(r, col));
        }
    }
    None
}

/// Last match strictly before `from`.
pub fn find_backward(lines: &[Line], needle: &[char], from: Point, fold: bool) -> Option<Point> {
    if needle.is_empty() || lines.is_empty() {
        return None;
    }
    let row = from.row.min(lines.len() - 1);
    let mut best = None;
    for (r, line) in lines.iter().enumerate().take(row + 1) {
        let limit = if r == row { from.col } else { line.len() };
        for col in 0..limit {
            if matches_at(line, col, needle, fold) {
                best = Some(Point::new(r, col));
            }
        }
    }
    best
}

pub fn find_last(lines: &[Line], needle: &[char], fold: bool) -> Option<Point> {
    if needle.is_empty() || lines.is_empty() {
        return None;
    }
    let mut best = None;
    for (r, line) in lines.iter().enumerate() {
        if line.len() < needle.len() {
            continue;
        }
        for col in 0..=line.len() - needle.len() {
            if matches_at(line, col, needle, fold) {
                best = Some(Point::new(r, col));
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &str) -> Vec<Line> {
        text.split('\n').map(|l| l.chars().collect()).collect()
    }

    #[test]
    fn forward_and_backward() {
        let l = lines("foo bar foo\nbaz foo");
        let needle: Vec<char> = "foo".chars().collect();
        assert_eq!(
            find_forward(&l, &needle, Point::new(0, 0), true),
            Some(Point::new(0, 0))
        );
        assert_eq!(
            find_forward(&l, &needle, Point::new(0, 1), true),
            Some(Point::new(0, 8))
        );
        assert_eq!(
            find_forward(&l, &needle, Point::new(0, 9), true),
            Some(Point::new(1, 4))
        );
        assert_eq!(
            find_backward(&l, &needle, Point::new(1, 4), true),
            Some(Point::new(0, 8))
        );
        assert_eq!(find_last(&l, &needle, true), Some(Point::new(1, 4)));
    }

    #[test]
    fn smart_case() {
        let l = lines("Foo foo");
        let needle: Vec<char> = "foo".chars().collect();
        assert_eq!(
            find_forward(&l, &needle, Point::new(0, 0), true),
            Some(Point::new(0, 0))
        );
        assert_eq!(
            find_forward(&l, &needle, Point::new(0, 0), false),
            Some(Point::new(0, 4))
        );
    }

    #[test]
    fn japanese_search() {
        let l = lines("これはテストです");
        let needle: Vec<char> = "テスト".chars().collect();
        assert_eq!(
            find_forward(&l, &needle, Point::new(0, 0), true),
            Some(Point::new(0, 3))
        );
    }
}
