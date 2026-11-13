use std::env;
use std::ffi::OsString;
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

use emacette::config::Config;
use emacette::editor::Editor;
use emacette::help;
use emacette::key::{Key, KeyCode};
use emacette::render;
use emacette::tmux;

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code.clamp(0, 255) as u8),
        Err(message) => {
            eprintln!("emacette: {message}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug)]
enum ParsedArgs {
    Help,
    Version,
    Run {
        file: Option<PathBuf>,
        line: Option<usize>,
    },
}

/// Parse arguments without assuming they are UTF-8.
///
/// `std::env::args()` panics on a non-UTF-8 argument.  Paths on Unix are
/// arbitrary bytes, and git can pass such a path to the editor, so use
/// `args_os()` and only interpret options and `+LINE` when the argument is
/// valid Unicode.
fn parse_args<I>(args: I) -> Result<ParsedArgs, String>
where
    I: IntoIterator<Item = OsString>,
{
    let mut file: Option<PathBuf> = None;
    let mut line: Option<usize> = None;
    let mut after_double_dash = false;

    for arg in args {
        if !after_double_dash {
            match arg.to_str() {
                Some("-h" | "--help") => return Ok(ParsedArgs::Help),
                Some("-V" | "--version") => return Ok(ParsedArgs::Version),
                Some("--") => {
                    after_double_dash = true;
                    continue;
                }
                _ => {}
            }
            if let Some(text) = arg.to_str() {
                if let Some(number) = text.strip_prefix('+') {
                    if let Ok(number) = number.parse::<usize>() {
                        line = Some(number.max(1));
                        continue;
                    }
                }
                // A lone dash is treated as a (strange) file name.
                if text.starts_with('-') && text != "-" {
                    return Err(format!("unknown option '{text}' (try --help)"));
                }
            }
        }
        if file.is_some() {
            return Err(format!(
                "unexpected extra argument '{}'",
                arg.to_string_lossy()
            ));
        }
        file = Some(PathBuf::from(arg));
    }

    Ok(ParsedArgs::Run { file, line })
}

fn run() -> Result<i32, String> {
    let (file, line) = match parse_args(env::args_os().skip(1))? {
        ParsedArgs::Help => {
            print_help();
            return Ok(0);
        }
        ParsedArgs::Version => {
            println!("emacette {}", env!("CARGO_PKG_VERSION"));
            return Ok(0);
        }
        ParsedArgs::Run { file, line } => (file, line),
    };

    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err("standard input/output is not a terminal".to_string());
    }

    let config = Config::load();
    let mut editor = Editor::open(file, config).map_err(|e| e.to_string())?;
    if let Some(line) = line {
        editor.buffer.move_to_line(line);
    }

    let _terminal = TerminalGuard::enter().map_err(|e| format!("terminal setup failed: {e}"))?;
    // tmux handles its own key tables before input reaches the pane.  Keep
    // emacette's bindings usable there, and restore tmux's bindings on exit.
    let _tmux_keys = tmux::KeyGuard::install();
    let mut stdout = io::stdout();
    let mut renderer = render::Renderer::new();

    let mut redraw = true;
    loop {
        // Language servers run asynchronously.  Poll them even when the user
        // is idle so diagnostics and completion responses can appear.
        if editor.poll_lsp() {
            redraw = true;
        }

        if redraw {
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
            redraw = false;
        }

        if editor.should_quit {
            break;
        }

        let ready = event::poll(Duration::from_millis(50))
            .map_err(|e| format!("terminal poll failed: {e}"))?;
        if !ready {
            continue;
        }
        handle_event(
            &mut editor,
            event::read().map_err(|e| format!("terminal read failed: {e}"))?,
        );
        redraw = true;

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
        "emacette {} - a tiny Emacs-flavoured terminal editor with syntax highlighting",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("Usage: emacette [OPTIONS] [FILE]");
    println!("       emacette +LINE [FILE]");
    println!();
    println!("Options:");
    println!("  -h, --help       show this help and the key bindings");
    println!("  -V, --version    print the version");
    println!();
    print!("{}", help::plain_text());
    println!();
    println!("Configuration file: ~/.config/emacette/config");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<ParsedArgs, String> {
        parse_args(args.iter().map(|arg| OsString::from(*arg)))
    }

    fn run_args(args: &[&str]) -> (Option<PathBuf>, Option<usize>) {
        match parse(args).expect("arguments should parse") {
            ParsedArgs::Run { file, line } => (file, line),
            other => panic!("expected run arguments, got {other:?}"),
        }
    }

    #[test]
    fn parses_files_options_and_line_numbers() {
        assert_eq!(
            run_args(&["+42", "note.txt"]),
            (Some(PathBuf::from("note.txt")), Some(42))
        );
        assert_eq!(run_args(&["-", "+0"]), (Some(PathBuf::from("-")), Some(1)));
        assert_eq!(run_args(&["--", "-h"]), (Some(PathBuf::from("-h")), None));
        assert!(matches!(parse(&["--help"]), Ok(ParsedArgs::Help)));
        assert!(matches!(parse(&["-V"]), Ok(ParsedArgs::Version)));
    }

    #[test]
    fn rejects_unknown_options_and_extra_arguments() {
        assert!(parse(&["--wat"]).unwrap_err().contains("unknown option"));
        assert!(parse(&["a.txt", "b.txt"])
            .unwrap_err()
            .contains("unexpected extra argument"));
    }

    #[cfg(unix)]
    #[test]
    fn accepts_non_utf8_file_names() {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};

        let arg = OsString::from_vec(b"bad-\xff.txt".to_vec());
        match parse_args(vec![arg]).expect("non-UTF-8 paths are valid arguments") {
            ParsedArgs::Run { file, .. } => {
                assert_eq!(file.expect("file").as_os_str().as_bytes(), b"bad-\xff.txt");
            }
            other => panic!("expected run arguments, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_extra_argument_is_reported_without_panicking() {
        use std::os::unix::ffi::OsStringExt;

        let args = vec![
            OsString::from("first.txt"),
            OsString::from_vec(b"second-\xff.txt".to_vec()),
        ];
        let error = parse_args(args).unwrap_err();
        assert!(error.contains("unexpected extra argument"));
        assert!(error.contains('\u{fffd}'));
    }
}
