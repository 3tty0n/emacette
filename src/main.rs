use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use crossterm::cursor;
use crossterm::event::{
    self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyEvent, KeyEventKind, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};

use nano_emacs::config::Config;
use nano_emacs::editor::Editor;
use nano_emacs::help;
use nano_emacs::key::{Key, KeyCode};
use nano_emacs::render;

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code.clamp(0, 255) as u8),
        Err(message) => {
            eprintln!("nemacs: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<i32, String> {
    let mut file: Option<PathBuf> = None;
    let mut line: Option<usize> = None;
    let mut after_double_dash = false;

    for arg in env::args().skip(1) {
        if !after_double_dash {
            match arg.as_str() {
                "-h" | "--help" => {
                    print_help();
                    return Ok(0);
                }
                "-V" | "--version" => {
                    println!("nemacs {}", env!("CARGO_PKG_VERSION"));
                    return Ok(0);
                }
                "--" => {
                    after_double_dash = true;
                    continue;
                }
                "-" => {
                    // Treat a lone dash as a (strange) file name.
                }
                _ => {}
            }
            if let Some(number) = arg.strip_prefix('+') {
                if let Ok(number) = number.parse::<usize>() {
                    line = Some(number.max(1));
                    continue;
                }
            }
            if arg.starts_with('-') && arg != "-" {
                return Err(format!("unknown option '{arg}' (try --help)"));
            }
        }
        if file.is_some() {
            return Err(format!("unexpected extra argument '{arg}'"));
        }
        file = Some(PathBuf::from(arg));
    }

    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err("standard input/output is not a terminal".to_string());
    }

    let config = Config::load();
    let mut editor = Editor::open(file, config).map_err(|e| e.to_string())?;
    if let Some(line) = line {
        editor.buffer.move_to_line(line);
    }

    let _terminal = TerminalGuard::enter().map_err(|e| format!("terminal setup failed: {e}"))?;
    let mut stdout = io::stdout();
    let mut renderer = render::Renderer::new();

    loop {
        let (width, height) = terminal::size().unwrap_or((80, 24));
        let (width, height) = if width == 0 || height == 0 {
            (80, 24)
        } else {
            (width, height)
        };
        let frame = renderer.render(&mut editor, width as usize, height as usize);
        if !frame.output.is_empty() {
            stdout
                .write_all(frame.output.as_bytes())
                .and_then(|()| stdout.flush())
                .map_err(|e| format!("write failed: {e}"))?;
        }

        if editor.should_quit {
            break;
        }

        handle_event(
            &mut editor,
            event::read().map_err(|e| format!("terminal read failed: {e}"))?,
        );

        // If several keys are already buffered (fast typing, key repeat,
        // paste), apply them all before painting once.  The renderer only
        // repaints rows that actually differ, so this stays cheap.
        let mut coalesced = 0;
        while coalesced < 512 && !editor.should_quit {
            let more =
                event::poll(Duration::ZERO).map_err(|e| format!("terminal poll failed: {e}"))?;
            if !more {
                break;
            }
            let event = event::read().map_err(|e| format!("terminal read failed: {e}"))?;
            handle_event(&mut editor, event);
            coalesced += 1;
        }

        if editor.should_quit {
            break;
        }
    }

    Ok(editor.exit_code)
}

fn handle_event(editor: &mut Editor, event: Event) {
    match event {
        Event::Key(key) => {
            if key.kind != KeyEventKind::Release {
                if let Some(key) = convert_key(key) {
                    editor.handle_key(key);
                }
            }
        }
        Event::Paste(text) => editor.handle_paste(&text),
        _ => {}
    }
}

fn print_help() {
    println!(
        "nemacs {} - a tiny Emacs-flavoured terminal editor with syntax highlighting",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("Usage: nemacs [OPTIONS] [FILE]");
    println!("       nemacs +LINE [FILE]");
    println!();
    println!("Options:");
    println!("  -h, --help       show this help and the key bindings");
    println!("  -V, --version    print the version");
    println!();
    print!("{}", help::plain_text());
    println!();
    println!("Configuration file: ~/.config/nano-emacs/config");
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let guard = TerminalGuard;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableBracketedPaste,
            cursor::Hide
        )?;
        stdout.flush()?;
        Ok(guard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = execute!(
            stdout,
            cursor::Show,
            DisableBracketedPaste,
            LeaveAlternateScreen
        );
        let _ = stdout.write_all(b"\x1b[?2026l");
        let _ = stdout.flush();
        let _ = terminal::disable_raw_mode();
    }
}

fn convert_key(event: KeyEvent) -> Option<Key> {
    let ctrl = event.modifiers.contains(KeyModifiers::CONTROL);
    let alt = event.modifiers.contains(KeyModifiers::ALT);
    let shift = event.modifiers.contains(KeyModifiers::SHIFT);
    let code = match event.code {
        crossterm::event::KeyCode::Char('\0') | crossterm::event::KeyCode::Null => {
            // Ctrl-Space / Ctrl-@ arrive as NUL.  Some terminals omit the
            // CONTROL modifier, so force it here rather than inserting a
            // literal space.
            return Some(Key {
                code: KeyCode::Char(' '),
                ctrl: true,
                alt: false,
                shift: false,
            });
        }
        crossterm::event::KeyCode::Char(c) => {
            let c = if ctrl && c.is_ascii_alphabetic() {
                c.to_ascii_lowercase()
            } else {
                c
            };
            KeyCode::Char(c)
        }
        crossterm::event::KeyCode::Enter => KeyCode::Enter,
        crossterm::event::KeyCode::Tab => KeyCode::Tab,
        crossterm::event::KeyCode::BackTab => KeyCode::BackTab,
        crossterm::event::KeyCode::Backspace => KeyCode::Backspace,
        crossterm::event::KeyCode::Delete => KeyCode::Delete,
        crossterm::event::KeyCode::Esc => KeyCode::Esc,
        crossterm::event::KeyCode::Up => KeyCode::Up,
        crossterm::event::KeyCode::Down => KeyCode::Down,
        crossterm::event::KeyCode::Left => KeyCode::Left,
        crossterm::event::KeyCode::Right => KeyCode::Right,
        crossterm::event::KeyCode::Home => KeyCode::Home,
        crossterm::event::KeyCode::End => KeyCode::End,
        crossterm::event::KeyCode::PageUp => KeyCode::PageUp,
        crossterm::event::KeyCode::PageDown => KeyCode::PageDown,
        crossterm::event::KeyCode::Insert => KeyCode::Insert,
        crossterm::event::KeyCode::F(n) => KeyCode::F(n),
        _ => return None,
    };
    Some(Key {
        code,
        ctrl,
        alt,
        shift,
    })
}
