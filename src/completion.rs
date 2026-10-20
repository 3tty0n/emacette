//! Editor-side completion candidates and word helpers.

use std::collections::BTreeSet;

use crate::buffer::{Line, Point};
use crate::syntax::Lang;

#[derive(Clone, Debug)]
pub struct EditorCompletion {
    pub candidates: Vec<String>,
    pub index: usize,
    pub start: Point,
    pub end: Point,
    pub prefix: String,
}

impl EditorCompletion {
    pub fn selected(&self) -> Option<&str> {
        self.candidates.get(self.index).map(String::as_str)
    }

    pub fn next(&mut self) {
        if !self.candidates.is_empty() {
            self.index = (self.index + 1) % self.candidates.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.candidates.is_empty() {
            self.index = (self.index + self.candidates.len() - 1) % self.candidates.len();
        }
    }
}

pub fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '\'' | '-')
}

/// The word-ish prefix ending at `col`, optionally including a leading LaTeX
/// backslash.
pub fn prefix_at(line: &[char], col: usize) -> (usize, String) {
    let col = col.min(line.len());
    let mut start = col;
    while start > 0 && is_word_char(line[start - 1]) {
        start -= 1;
    }
    if start > 0 && line[start - 1] == '\\' {
        start -= 1;
    }
    (start, line[start..col].iter().collect())
}

/// Find the word containing point, or the next word if point is on
/// punctuation/whitespace.
pub fn word_at(line: &[char], col: usize) -> Option<(usize, usize, String)> {
    let mut start = col.min(line.len());
    if start == line.len() && start > 0 {
        start -= 1;
    }
    if start < line.len() && line[start].is_alphabetic() {
        while start > 0 && line[start - 1].is_alphabetic() {
            start -= 1;
        }
    } else {
        while start < line.len() && !line[start].is_alphabetic() {
            start += 1;
        }
    }
    if start >= line.len() {
        return None;
    }
    let mut end = start;
    while end < line.len() && (line[end].is_alphabetic() || line[end] == '\'') {
        end += 1;
    }
    let word: String = line[start..end].iter().collect();
    if word.chars().count() < 2 {
        return None;
    }
    Some((start, end, word))
}

/// Unique words and `\commands` in the buffer, sorted alphabetically.
pub fn collect_words(lines: &[Line], _lang: Lang) -> Vec<String> {
    let mut words = BTreeSet::new();
    for line in lines {
        if words.len() >= 20_000 {
            break;
        }
        collect_line_words(line, &mut words);
    }
    words.into_iter().collect()
}

fn collect_line_words(line: &[char], words: &mut BTreeSet<String>) {
    let mut i = 0;
    while i < line.len() {
        let c = line[i];
        if c == '\\' && i + 1 < line.len() && line[i + 1].is_alphabetic() {
            let mut j = i + 1;
            while j < line.len() && line[j].is_alphabetic() {
                j += 1;
            }
            let command: String = line[i..j].iter().collect();
            if command.chars().count() <= 64 {
                words.insert(command);
            }
            i = j;
            continue;
        }
        if c.is_alphabetic() {
            let mut j = i;
            while j < line.len() && is_word_char(line[j]) {
                j += 1;
            }
            let word: String = line[i..j].iter().collect();
            let trimmed = word.trim_matches(|c: char| c == '-' || c == '\'');
            if trimmed.chars().count() >= 2
                && trimmed.chars().count() <= 64
                && !trimmed.chars().all(|c| c.is_ascii_digit())
            {
                words.insert(trimmed.to_string());
            }
            i = j;
            continue;
        }
        i += 1;
    }
}

/// Candidates that start with `prefix`, excluding the prefix itself.
pub fn candidates(prefix: &str, words: &[String], limit: usize) -> Vec<String> {
    words
        .iter()
        .filter(|word| word.as_str() != prefix && word.starts_with(prefix))
        .take(limit)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_word_and_command_prefixes() {
        let line: Vec<char> = "hello \\sect".chars().collect();
        assert_eq!(prefix_at(&line, 5).1, "hello");
        assert_eq!(prefix_at(&line, 11).1, "\\sect");
        let line: Vec<char> = "don't".chars().collect();
        assert_eq!(prefix_at(&line, 5).1, "don't");
    }

    #[test]
    fn collects_unique_words_and_commands() {
        let lines = vec![
            "hello world hello".chars().collect(),
            "\\section{World}".chars().collect(),
        ];
        let words = collect_words(&lines, Lang::Latex);
        assert!(words.contains(&"hello".to_string()));
        assert!(words.contains(&"world".to_string()));
        assert!(words.contains(&"\\section".to_string()));
        assert_eq!(words.iter().filter(|w| *w == "hello").count(), 1);
    }

    #[test]
    fn filters_candidates_by_prefix() {
        let words = vec![
            "alpha".to_string(),
            "alphabet".to_string(),
            "beta".to_string(),
        ];
        assert_eq!(candidates("alph", &words, 10), vec!["alpha", "alphabet"]);
        assert_eq!(candidates("alpha", &words, 10), vec!["alphabet"]);
        assert!(candidates("zz", &words, 10).is_empty());
    }

    #[test]
    fn finds_word_under_point() {
        let line: Vec<char> = "one tow three".chars().collect();
        assert_eq!(word_at(&line, 5), Some((4, 7, "tow".to_string())));
        assert_eq!(word_at(&line, 3), Some((4, 7, "tow".to_string())));
        assert_eq!(word_at(&line, 0), Some((0, 3, "one".to_string())));
    }
}
