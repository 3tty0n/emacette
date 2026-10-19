//! Terminal-independent key representation.

/// Map raw control codes that do not have a letter equivalent (C-_, C-\\,
/// C-], C-^, C-SPC, C-h) to the character Emacs users expect.
pub fn normalize_control(c: char) -> char {
    match c {
        '\u{0}' => ' ',
        '\u{8}' => 'h',
        '\u{1c}' => '\\',
        '\u{1d}' => ']',
        '\u{1e}' => '^',
        '\u{1f}' => '_',
        _ => c,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Char(char),
    Enter,
    Tab,
    BackTab,
    Backspace,
    Delete,
    Esc,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    F(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Key {
    pub const fn ch(c: char) -> Self {
        Self {
            code: KeyCode::Char(c),
            ctrl: false,
            alt: false,
            shift: false,
        }
    }

    pub const fn ctrl(c: char) -> Self {
        Self {
            code: KeyCode::Char(c),
            ctrl: true,
            alt: false,
            shift: false,
        }
    }

    pub const fn alt(c: char) -> Self {
        Self {
            code: KeyCode::Char(c),
            ctrl: false,
            alt: true,
            shift: false,
        }
    }

    pub const fn meta_ctrl(c: char) -> Self {
        Self {
            code: KeyCode::Char(c),
            ctrl: true,
            alt: true,
            shift: false,
        }
    }

    pub const fn code(code: KeyCode) -> Self {
        Self {
            code,
            ctrl: false,
            alt: false,
            shift: false,
        }
    }

    pub const fn ctrl_code(code: KeyCode) -> Self {
        Self {
            code,
            ctrl: true,
            alt: false,
            shift: false,
        }
    }

    /// Human readable name, for the echo area.
    pub fn display(&self) -> String {
        let base = match self.code {
            KeyCode::Char(c) => {
                if c == ' ' {
                    "SPC".to_string()
                } else if c.is_control() {
                    format!("^{}", (c as u8 as char).to_ascii_uppercase())
                } else {
                    c.to_string()
                }
            }
            KeyCode::Enter => "RET".to_string(),
            KeyCode::Tab => "TAB".to_string(),
            KeyCode::BackTab => "S-TAB".to_string(),
            KeyCode::Backspace => "DEL".to_string(),
            KeyCode::Delete => "DELETE".to_string(),
            KeyCode::Esc => "ESC".to_string(),
            KeyCode::Up => "UP".to_string(),
            KeyCode::Down => "DOWN".to_string(),
            KeyCode::Left => "LEFT".to_string(),
            KeyCode::Right => "RIGHT".to_string(),
            KeyCode::Home => "HOME".to_string(),
            KeyCode::End => "END".to_string(),
            KeyCode::PageUp => "PGUP".to_string(),
            KeyCode::PageDown => "PGDN".to_string(),
            KeyCode::Insert => "INS".to_string(),
            KeyCode::F(n) => format!("F{n}"),
        };
        let mut out = String::new();
        if self.ctrl {
            out.push_str("C-");
        }
        if self.alt {
            out.push_str("M-");
        }
        out.push_str(&base);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_non_letter_control_codes() {
        assert_eq!(normalize_control('\u{1f}'), '_');
        assert_eq!(normalize_control('\u{0}'), ' ');
        assert_eq!(normalize_control('\u{8}'), 'h');
        assert_eq!(normalize_control('\u{1c}'), '\\');
    }

    #[test]
    fn displays_modifiers() {
        assert_eq!(Key::ctrl('x').display(), "C-x");
        assert_eq!(Key::alt('f').display(), "M-f");
        assert_eq!(Key::meta_ctrl('g').display(), "C-M-g");
        assert_eq!(Key::ctrl(' ').display(), "C-SPC");
        assert_eq!(Key::code(KeyCode::Enter).display(), "RET");
    }
}
