//! Lightweight English spelling checks for prose modes.
//!
//! The checker reads a plain word list or a hunspell `.dic` file.  It uses the
//! syntax spans from the editor so LaTeX commands, Markdown code, comments and
//! math are skipped.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::syntax::{Span, Tok};

pub struct SpellChecker {
    words: HashSet<String>,
    sorted: Vec<String>,
    source: PathBuf,
}

impl SpellChecker {
    /// Load an explicit dictionary, `$EMACETTE_DICT` (or the legacy
    /// `$NANO_EMACS_DICT`), or a common English word list / hunspell
    /// dictionary.
    pub fn load(explicit: Option<&str>) -> Option<Self> {
        let path = explicit
            .map(expand_path)
            .filter(|path| path.is_file())
            .or_else(|| {
                std::env::var("EMACETTE_DICT")
                    .ok()
                    .filter(|value| !value.is_empty())
                    .map(|value| expand_path(&value))
                    .filter(|path| path.is_file())
            })
            .or_else(|| {
                // Migration fallback from the old project name.
                std::env::var("NANO_EMACS_DICT")
                    .ok()
                    .filter(|value| !value.is_empty())
                    .map(|value| expand_path(&value))
                    .filter(|path| path.is_file())
            })
            .or_else(find_dictionary)?;
        Self::from_path(&path)
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        let text = fs::read_to_string(path).ok()?;
        let words = parse_dictionary(
            &text,
            path.extension().and_then(|e| e.to_str()) == Some("dic"),
        );
        if words.is_empty() {
            return None;
        }
        let mut sorted: Vec<String> = words.iter().cloned().collect();
        sorted.sort();
        Some(Self {
            words,
            sorted,
            source: path.to_path_buf(),
        })
    }

    pub fn from_words<I, S>(words: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let words: HashSet<String> = words
            .into_iter()
            .map(|word| word.as_ref().to_lowercase())
            .filter(|word| !word.is_empty())
            .collect();
        let mut sorted: Vec<String> = words.iter().cloned().collect();
        sorted.sort();
        Self {
            words,
            sorted,
            source: PathBuf::from("<memory>"),
        }
    }

    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn is_correct(&self, word: &str) -> bool {
        let word = word.to_lowercase();
        if self.words.contains(&word) {
            return true;
        }
        if word.contains('-') {
            return word
                .split('-')
                .filter(|part| !part.is_empty())
                .all(|part| self.words.contains(part));
        }
        false
    }

    pub fn check_line(&self, line: &[char], spans: &[Span]) -> Vec<(usize, usize)> {
        let mut blocked = vec![false; line.len()];
        for span in spans {
            let prose = is_prose(span.kind) && line.get(span.start) != Some(&'\\');
            if !prose {
                mark_blocked(&mut blocked, span.start, span.end);
            }
        }
        mark_latex_arguments(line, &mut blocked);

        let mut errors = Vec::new();
        let mut i = 0usize;
        while i < line.len() {
            if blocked[i] || !line[i].is_alphabetic() {
                i += 1;
                continue;
            }
            let start = i;
            let mut end = i;
            while end < line.len()
                && !blocked[end]
                && (line[end].is_alphabetic() || matches!(line[end], '\'' | '-'))
            {
                end += 1;
            }
            let token: String = line[start..end].iter().collect();
            if is_checkable(&token) && !self.is_correct(&token) {
                errors.push((start, end));
            }
            i = end.max(start + 1);
        }
        errors
    }

    pub fn suggestions(&self, word: &str, limit: usize) -> Vec<String> {
        let word = word.to_lowercase();
        if self.is_correct(&word) {
            return Vec::new();
        }
        let mut found: Vec<(usize, &str)> = Vec::new();
        for candidate in &self.sorted {
            if candidate.chars().count().abs_diff(word.chars().count()) > 2 {
                continue;
            }
            if let Some(distance) = edit_distance(&word, candidate, 2) {
                found.push((distance, candidate.as_str()));
            }
        }
        found.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));
        found
            .into_iter()
            .take(limit)
            .map(|(_, word)| word.to_string())
            .collect()
    }
}

fn is_checkable(token: &str) -> bool {
    let token = token.trim_matches('-');
    let count = token.chars().count();
    if count < 2 {
        return false;
    }
    if token.contains('\'') {
        return false;
    }
    if token.chars().all(|c| c.is_uppercase()) {
        return false;
    }
    token.chars().all(|c| c.is_alphabetic() || c == '-')
}

const LATEX_SKIP_ARGS: &[&str] = &[
    "begin",
    "end",
    "label",
    "ref",
    "eqref",
    "pageref",
    "autoref",
    "nameref",
    "vref",
    "cref",
    "Cref",
    "cite",
    "citep",
    "citet",
    "citeauthor",
    "citeyear",
    "parencite",
    "textcite",
    "footcite",
    "url",
    "path",
    "input",
    "include",
    "includeonly",
    "includegraphics",
    "graphicspath",
    "bibliography",
    "bibliographystyle",
    "documentclass",
    "usepackage",
    "RequirePackage",
    "hypersetup",
    "href",
];

fn mark_blocked(blocked: &mut [bool], start: usize, end: usize) {
    let end = end.min(blocked.len());
    for flag in blocked.iter_mut().take(end).skip(start) {
        *flag = true;
    }
}

fn mark_latex_arguments(line: &[char], blocked: &mut [bool]) {
    let mut i = 0usize;
    while i < line.len() {
        if line[i] != '\\' {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < line.len() && line[j].is_alphabetic() {
            j += 1;
        }
        if j == i + 1 {
            i += 1;
            continue;
        }
        let name: String = line[i + 1..j].iter().collect();
        if !LATEX_SKIP_ARGS.contains(&name.as_str()) {
            i = j;
            continue;
        }
        let mut k = j;
        if k < line.len() && line[k] == '*' {
            k += 1;
        }
        while k < line.len() && line[k].is_whitespace() {
            k += 1;
        }
        while k < line.len() && line[k] == '[' {
            let Some(close) = find_close(line, k, '[', ']') else {
                break;
            };
            mark_blocked(blocked, k, close + 1);
            k = close + 1;
            while k < line.len() && line[k].is_whitespace() {
                k += 1;
            }
        }
        if k < line.len() && line[k] == '{' {
            if let Some(close) = find_close(line, k, '{', '}') {
                mark_blocked(blocked, k, close + 1);
                k = close + 1;
            }
        }
        i = k.max(i + 1);
    }
}

fn find_close(line: &[char], open: usize, opener: char, closer: char) -> Option<usize> {
    let mut depth = 0u32;
    for (index, c) in line.iter().enumerate().skip(open) {
        if *c == opener {
            depth += 1;
        } else if *c == closer {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn is_prose(kind: Tok) -> bool {
    matches!(kind, Tok::Heading | Tok::Emphasis)
}

fn parse_dictionary(text: &str, hunspell: bool) -> HashSet<String> {
    let mut words = HashSet::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if hunspell && index == 0 && line.parse::<usize>().is_ok() {
            continue;
        }
        let base = if hunspell {
            line.split('/').next().unwrap_or(line)
        } else {
            line.split_whitespace().next().unwrap_or(line)
        };
        let word = base.trim().to_lowercase();
        if word.is_empty() {
            continue;
        }
        if word
            .chars()
            .all(|c| c.is_alphabetic() || c == '\'' || c == '-')
        {
            words.insert(word);
        }
    }
    words
}

fn expand_path(input: &str) -> PathBuf {
    if input == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home);
        }
    }
    if let Some(rest) = input.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(input)
}

fn find_dictionary() -> Option<PathBuf> {
    let mut paths = vec![
        PathBuf::from("/usr/share/dict/words"),
        PathBuf::from("/usr/share/dict/american-english"),
        PathBuf::from("/usr/share/dict/british-english"),
        PathBuf::from("/usr/share/hunspell/en_US.dic"),
        PathBuf::from("/usr/share/hunspell/en_GB.dic"),
        PathBuf::from("/usr/share/myspell/dicts/en_US.dic"),
        PathBuf::from("/usr/share/myspell/dicts/en_GB.dic"),
        PathBuf::from("/Library/Spelling/en_US.dic"),
        PathBuf::from("/Library/Spelling/en_GB.dic"),
    ];
    if let Ok(home) = std::env::var("HOME") {
        paths.push(PathBuf::from(&home).join(".local/share/hunspell/en_US.dic"));
        paths.push(PathBuf::from(&home).join(".local/share/hunspell/en_GB.dic"));
    }
    paths.into_iter().find(|path| path.is_file())
}

fn edit_distance(a: &str, b: &str, max: usize) -> Option<usize> {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > max {
        return None;
    }
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        current[0] = i + 1;
        let mut row_min = current[0];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            current[j + 1] = (previous[j + 1] + 1)
                .min(current[j] + 1)
                .min(previous[j] + cost);
            row_min = row_min.min(current[j + 1]);
        }
        if row_min > max {
            return None;
        }
        std::mem::swap(&mut previous, &mut current);
    }
    let distance = previous[b.len()];
    (distance <= max).then_some(distance)
}

pub struct SpellState {
    pub enabled: bool,
    pub checker: Option<SpellChecker>,
    version: u64,
    cache: Vec<Option<Vec<(usize, usize)>>>,
}

impl Default for SpellState {
    fn default() -> Self {
        Self {
            enabled: false,
            checker: None,
            version: u64::MAX,
            cache: Vec::new(),
        }
    }
}

impl SpellState {
    pub fn new(enabled: bool, checker: Option<SpellChecker>) -> Self {
        Self {
            enabled: enabled && checker.is_some(),
            checker,
            ..Self::default()
        }
    }

    pub fn set(&mut self, enabled: bool, checker: Option<SpellChecker>) {
        self.enabled = enabled && checker.is_some();
        self.checker = checker;
        self.cache.clear();
        self.version = u64::MAX;
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.version = u64::MAX;
    }

    pub fn spans(
        &mut self,
        version: u64,
        row: usize,
        line: &[char],
        syntax: &[Span],
    ) -> Vec<(usize, usize)> {
        if !self.enabled {
            return Vec::new();
        }
        let Some(checker) = self.checker.as_ref() else {
            return Vec::new();
        };
        if self.version != version {
            self.cache.clear();
            self.version = version;
        }
        if self.cache.len() <= row {
            self.cache.resize(row + 1, None);
        }
        let entry = &mut self.cache[row];
        if entry.is_none() {
            *entry = Some(checker.check_line(line, syntax));
        }
        entry.clone().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(start: usize, end: usize, kind: Tok) -> Span {
        Span::new(start, end, kind)
    }

    #[test]
    fn flags_unknown_words_and_skips_code() {
        let checker = SpellChecker::from_words(["hello", "world", "rust"]);
        let line: Vec<char> = "hello wurld".chars().collect();
        assert_eq!(checker.check_line(&line, &[]), vec![(6, 11)]);
        let line: Vec<char> = "hello wurld".chars().collect();
        let spans = [span(6, 11, Tok::String)];
        assert!(checker.check_line(&line, &spans).is_empty());
        let line: Vec<char> = "\\command{hello}".chars().collect();
        let spans = [span(0, 8, Tok::Function)];
        assert!(checker.check_line(&line, &spans).is_empty());
    }

    #[test]
    fn skips_latex_section_commands() {
        let checker = SpellChecker::from_words(["hello"]);
        let line: Vec<char> = r"\section{wurld}".chars().collect();
        let spans = [
            span(0, 8, Tok::Heading),
            span(8, 9, Tok::Operator),
            span(14, 15, Tok::Operator),
        ];
        assert_eq!(checker.check_line(&line, &spans), vec![(9, 14)]);
    }

    #[test]
    fn skips_latex_label_and_reference_arguments() {
        let checker = SpellChecker::from_words(["hello"]);
        let line: Vec<char> = r"\label{sec:intro} hello wurld".chars().collect();
        let spans = [
            span(0, 6, Tok::Function),
            span(6, 7, Tok::Operator),
            span(10, 11, Tok::Operator),
            span(16, 17, Tok::Operator),
        ];
        assert_eq!(checker.check_line(&line, &spans), vec![(24, 29)]);
    }

    #[test]
    fn ignores_short_and_capitalised_acronyms() {
        let checker = SpellChecker::from_words(["hello"]);
        let line: Vec<char> = "a NASA hello".chars().collect();
        assert!(checker.check_line(&line, &[]).is_empty());
    }

    #[test]
    fn suggests_close_words() {
        let checker = SpellChecker::from_words(["hello", "world", "word"]);
        let suggestions = checker.suggestions("wurld", 3);
        assert!(suggestions.contains(&"world".to_string()));
        assert!(suggestions.len() <= 3);
    }

    #[test]
    fn parses_plain_and_hunspell_lists() {
        let plain = parse_dictionary("hello\nworld\n# comment\n", false);
        assert!(plain.contains("hello"));
        let hunspell = parse_dictionary("2\nhello/A\nworld\n", true);
        assert!(hunspell.contains("hello"));
        assert!(hunspell.contains("world"));
    }

    #[test]
    fn cache_tracks_buffer_version() {
        let checker = SpellChecker::from_words(["hello"]);
        let mut state = SpellState::new(true, Some(checker));
        let line: Vec<char> = "wurld".chars().collect();
        assert_eq!(state.spans(1, 0, &line, &[]), vec![(0, 5)]);
        assert_eq!(state.spans(1, 0, &line, &[]), vec![(0, 5)]);
        assert_eq!(state.spans(2, 0, &line, &[]), vec![(0, 5)]);
    }
}
