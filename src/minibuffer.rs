//! The minibuffer: a single line prompt with Emacs-ish editing.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PromptKind {
    FindFile,
    WriteFile { then_quit: bool },
    GotoLine,
    Mx,
}

#[derive(Clone, Debug)]
pub struct Minibuffer {
    pub prompt: String,
    pub input: Vec<char>,
    pub cursor: usize,
    pub kind: PromptKind,
    pub hint: String,
    history_index: Option<usize>,
    stash: Vec<char>,
}

impl Minibuffer {
    pub fn new(prompt: impl Into<String>, kind: PromptKind, initial: &str) -> Self {
        let input: Vec<char> = initial.chars().collect();
        Self {
            prompt: prompt.into(),
            cursor: input.len(),
            input,
            kind,
            hint: String::new(),
            history_index: None,
            stash: Vec::new(),
        }
    }

    pub fn text(&self) -> String {
        self.input.iter().collect()
    }

    pub fn insert(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn insert_str(&mut self, s: &str) {
        for c in s.chars() {
            if c != '\n' && c != '\r' {
                self.insert(c);
            }
        }
    }

    pub fn backspace(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.cursor -= 1;
        self.input.remove(self.cursor);
        true
    }

    pub fn delete_forward(&mut self) -> bool {
        if self.cursor >= self.input.len() {
            return false;
        }
        self.input.remove(self.cursor);
        true
    }

    /// Delete the word before the cursor (C-w-ish behaviour).
    pub fn delete_word_backward(&mut self) -> String {
        let end = self.cursor;
        let mut start = end;
        while start > 0 && self.input[start - 1].is_whitespace() {
            start -= 1;
        }
        while start > 0 && !self.input[start - 1].is_whitespace() {
            start -= 1;
        }
        let removed: String = self.input[start..end].iter().collect();
        self.input.drain(start..end);
        self.cursor = start;
        removed
    }

    pub fn kill_to_end(&mut self) -> String {
        let removed: String = self.input[self.cursor..].iter().collect();
        self.input.truncate(self.cursor);
        removed
    }

    pub fn left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn right(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor += 1;
        }
    }

    pub fn home(&mut self) {
        self.cursor = 0;
    }

    pub fn end(&mut self) {
        self.cursor = self.input.len();
    }

    pub fn set_text(&mut self, text: &str) {
        self.input = text.chars().collect();
        self.cursor = self.input.len();
    }

    pub fn history_prev(&mut self, history: &[String]) {
        if history.is_empty() {
            return;
        }
        let index = match self.history_index {
            None => {
                self.stash = self.input.clone();
                history.len() - 1
            }
            Some(0) => 0,
            Some(i) => i - 1,
        };
        self.history_index = Some(index);
        self.input = history[index].chars().collect();
        self.cursor = self.input.len();
    }

    pub fn history_next(&mut self, history: &[String]) {
        let Some(index) = self.history_index else {
            return;
        };
        if index + 1 < history.len() {
            self.history_index = Some(index + 1);
            self.input = history[index + 1].chars().collect();
        } else {
            self.history_index = None;
            self.input = std::mem::take(&mut self.stash);
        }
        self.cursor = self.input.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editing_basics() {
        let mut mb = Minibuffer::new("Find file: ", PromptKind::FindFile, "abc");
        mb.home();
        mb.insert('X');
        assert_eq!(mb.text(), "Xabc");
        assert!(mb.delete_forward());
        assert_eq!(mb.text(), "Xbc");
        mb.end();
        assert!(mb.backspace());
        assert_eq!(mb.text(), "Xb");
    }

    #[test]
    fn kill_and_word_delete() {
        let mut mb = Minibuffer::new(": ", PromptKind::GotoLine, "one two");
        assert_eq!(mb.kill_to_end(), "");
        mb.home();
        assert_eq!(mb.kill_to_end(), "one two");
        let mut mb = Minibuffer::new(": ", PromptKind::GotoLine, "one two");
        assert_eq!(mb.delete_word_backward(), "two");
        assert_eq!(mb.text(), "one ");
    }

    #[test]
    fn history_walks_and_restores() {
        let history = vec!["one".to_string(), "two".to_string()];
        let mut mb = Minibuffer::new(": ", PromptKind::Mx, "current");
        mb.history_prev(&history);
        assert_eq!(mb.text(), "two");
        mb.history_prev(&history);
        assert_eq!(mb.text(), "one");
        mb.history_next(&history);
        assert_eq!(mb.text(), "two");
        mb.history_next(&history);
        assert_eq!(mb.text(), "current");
    }
}
