//! Syntax highlighting cache: one lexer state per line, computed lazily.

use crate::buffer::Line;
use crate::syntax::{lex, Lang, LexState, Span};

pub struct Highlighter {
    pub lang: Lang,
    states: Vec<LexState>,
    tokens: Vec<Option<Vec<Span>>>,
    valid: usize,
    pub version: u64,
}

impl Highlighter {
    pub fn new(lang: Lang, version: u64) -> Self {
        Self {
            lang,
            states: vec![LexState::default()],
            tokens: Vec::new(),
            valid: 1,
            version,
        }
    }

    /// Forget highlight state from `from` (inclusive) onwards.
    pub fn invalidate(&mut self, from: usize) {
        let from = from.min(self.valid.saturating_sub(1));
        self.states.truncate(from + 1);
        self.valid = from + 1;
        self.tokens.truncate(from);
    }

    fn ensure(&mut self, row: usize, lines: &[Line]) {
        if lines.is_empty() {
            return;
        }
        let row = row.min(lines.len() - 1);
        while self.valid <= row {
            let li = self.valid - 1;
            let state = self.states[li];
            let (spans, end) = lex(self.lang, &lines[li], state);
            if self.tokens.len() <= li {
                self.tokens.resize(li + 1, None);
            }
            self.tokens[li] = Some(spans);
            self.states.push(end);
            self.valid += 1;
        }
        if self.tokens.get(row).map_or(true, |t| t.is_none()) {
            let state = self.states[row];
            let (spans, end) = lex(self.lang, &lines[row], state);
            if self.tokens.len() <= row {
                self.tokens.resize(row + 1, None);
            }
            self.tokens[row] = Some(spans);
            if self.valid == row + 1 {
                self.states.push(end);
                self.valid += 1;
            }
        }
    }

    pub fn tokens(&mut self, row: usize, lines: &[Line]) -> &[Span] {
        self.ensure(row, lines);
        self.tokens
            .get(row)
            .and_then(|t| t.as_deref())
            .unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &str) -> Vec<Line> {
        text.split('\n').map(|l| l.chars().collect()).collect()
    }

    #[test]
    fn caches_and_invalidates() {
        let lines = lines("fn main() {\n/*\ncomment\n*/\n}");
        let mut hl = Highlighter::new(Lang::Rust, 1);
        let first = hl.tokens(0, &lines).to_vec();
        assert!(first.iter().any(|s| s.kind == crate::syntax::Tok::Keyword));
        let comment = hl.tokens(2, &lines).to_vec();
        assert_eq!(comment[0].kind, crate::syntax::Tok::Comment);

        hl.invalidate(2);
        let comment_again = hl.tokens(2, &lines).to_vec();
        assert_eq!(comment_again, comment);
    }

    #[test]
    fn invalidation_recomputes_following_lines() {
        let mut ls = lines("let x = \"a\";\nlet y = 1;");
        let mut hl = Highlighter::new(Lang::Rust, 1);
        assert_eq!(hl.tokens(1, &ls)[0].kind, crate::syntax::Tok::Keyword);
        ls[0] = "/* open".chars().collect();
        hl.invalidate(0);
        let second = hl.tokens(1, &ls).to_vec();
        assert_eq!(second[0].kind, crate::syntax::Tok::Comment);
    }
}
