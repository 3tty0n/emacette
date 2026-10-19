//! Headless integration tests that drive the editor exactly like a user would.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use nano_emacs::config::Config;
use nano_emacs::editor::Editor;
use nano_emacs::key::{Key, KeyCode};
use nano_emacs::syntax::Lang;

fn temp_dir() -> PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("nano-emacs-it-{}-{}", std::process::id(), n));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn editor(text: &str) -> Editor {
    Editor::from_text(text, None, Config::default())
}

fn press(editor: &mut Editor, keys: &[Key]) {
    for key in keys {
        editor.handle_key(*key);
    }
}

fn type_text(editor: &mut Editor, text: &str) {
    for c in text.chars() {
        editor.handle_key(Key::ch(c));
    }
}

fn clear_minibuffer(editor: &mut Editor) {
    press(editor, &[Key::ctrl('a'), Key::ctrl('k')]);
}

const ENTER: Key = Key::code(KeyCode::Enter);

#[test]
fn save_buffer_writes_the_file() {
    let dir = temp_dir();
    let path = dir.join("note.txt");
    fs::write(&path, "existing\n").unwrap();
    let mut ed = Editor::from_text("existing\n", Some(path.clone()), Config::default());
    ed.buffer.set_point(nano_emacs::buffer::Point::new(0, 0));
    type_text(&mut ed, ">");
    press(&mut ed, &[Key::ctrl('x'), Key::ctrl('s')]);
    assert_eq!(fs::read_to_string(&path).unwrap(), ">existing\n");
    assert!(!ed.buffer.modified);
    assert!(ed.message.contains("Wrote"));
}

#[test]
fn write_file_prompt_creates_a_file_and_switches_mode() {
    let dir = temp_dir();
    let target = dir.join("new.rs");
    let mut ed = editor("fn main() {}");
    press(&mut ed, &[Key::ctrl('x'), Key::ctrl('w')]);
    assert!(ed.minibuffer.is_some());
    clear_minibuffer(&mut ed);
    type_text(&mut ed, target.to_str().unwrap());
    press(&mut ed, &[ENTER]);
    assert_eq!(fs::read_to_string(&target).unwrap(), "fn main() {}");
    assert_eq!(ed.buffer.path.as_deref(), Some(target.as_path()));
    assert_eq!(ed.mode, Lang::Rust);
}

#[test]
fn find_file_prompt_opens_a_file() {
    let dir = temp_dir();
    let target = dir.join("other.md");
    fs::write(&target, "# hello\n").unwrap();
    let mut ed = editor("scratch");
    press(&mut ed, &[Key::ctrl('x'), Key::ctrl('f')]);
    clear_minibuffer(&mut ed);
    type_text(&mut ed, target.to_str().unwrap());
    press(&mut ed, &[ENTER]);
    assert_eq!(ed.buffer_text(), "# hello\n");
    assert_eq!(ed.mode, Lang::Markdown);
    assert_eq!(ed.buffer.point, nano_emacs::buffer::Point::new(0, 0));
}

#[test]
fn find_file_is_refused_when_buffer_is_modified() {
    let dir = temp_dir();
    let target = dir.join("other.txt");
    fs::write(&target, "other\n").unwrap();
    let mut ed = editor("");
    type_text(&mut ed, "dirty");
    press(&mut ed, &[Key::ctrl('x'), Key::ctrl('f')]);
    clear_minibuffer(&mut ed);
    type_text(&mut ed, target.to_str().unwrap());
    press(&mut ed, &[ENTER]);
    assert_eq!(ed.buffer_text(), "dirty");
    assert!(ed.message.contains("modified"));
}

#[test]
fn minibuffer_cancel_keeps_state() {
    let mut ed = editor("body");
    press(&mut ed, &[Key::ctrl('x'), Key::ctrl('f')]);
    type_text(&mut ed, "/nonexistent/path");
    press(&mut ed, &[Key::ctrl('g')]);
    assert!(ed.minibuffer.is_none());
    assert_eq!(ed.buffer_text(), "body");
    assert!(ed.buffer.path.is_none());
}

#[test]
fn undo_stops_at_cursor_movements() {
    let mut ed = editor("");
    type_text(&mut ed, "ab");
    press(&mut ed, &[Key::ctrl('f')]);
    type_text(&mut ed, "cd");
    assert_eq!(ed.buffer_text(), "abcd");
    press(&mut ed, &[Key::ctrl('/')]);
    assert_eq!(ed.buffer_text(), "ab");
}

#[test]
fn quit_confirmation_can_be_cancelled_and_declined() {
    let path = temp_dir().join("quit.txt");
    fs::write(&path, "").unwrap();
    let mut ed = Editor::from_text("", Some(path), Config::default());
    type_text(&mut ed, "x");
    press(&mut ed, &[Key::ctrl('x'), Key::ctrl('c')]);
    assert!(!ed.should_quit);
    assert!(ed.prompt_for_quit().unwrap().contains("Save changes"));
    press(&mut ed, &[Key::ctrl('g')]);
    assert!(!ed.should_quit);
    press(&mut ed, &[Key::ctrl('x'), Key::ctrl('c')]);
    press(&mut ed, &[Key::ch('n')]);
    assert!(ed.should_quit);
    assert_eq!(ed.exit_code, 0);
}

#[test]
fn quitting_an_unnamed_modified_buffer_prompts_for_a_file() {
    let dir = temp_dir();
    let target = dir.join("saved.txt");
    let mut ed = editor("");
    type_text(&mut ed, "body");
    press(&mut ed, &[Key::ctrl('x'), Key::ctrl('c')]);
    assert!(ed.prompt_for_quit().unwrap().contains("Save this buffer"));
    press(&mut ed, &[Key::ch('y')]);
    assert!(ed.minibuffer.is_some());
    clear_minibuffer(&mut ed);
    type_text(&mut ed, target.to_str().unwrap());
    press(&mut ed, &[ENTER]);
    assert!(ed.should_quit);
    assert_eq!(fs::read_to_string(&target).unwrap(), "body");
}

#[test]
fn git_commit_finish_with_empty_message_asks_first() {
    let dir = temp_dir();
    let path = dir.join("COMMIT_EDITMSG");
    fs::write(&path, "# Please enter the commit message\n#\n").unwrap();
    let mut ed = Editor::from_text(
        &fs::read_to_string(&path).unwrap(),
        Some(path),
        Config::default(),
    );
    assert_eq!(ed.mode, Lang::GitCommit);
    press(&mut ed, &[Key::ctrl('c'), Key::ctrl('c')]);
    assert!(!ed.should_quit);
    assert!(ed
        .prompt_for_quit()
        .unwrap()
        .contains("Empty commit message"));
    press(&mut ed, &[Key::ch('y')]);
    assert!(ed.should_quit);
    assert_eq!(ed.exit_code, 0);
}

#[test]
fn git_commit_abort_discards_and_exits_nonzero() {
    let dir = temp_dir();
    let path = dir.join("COMMIT_EDITMSG");
    fs::write(&path, "# template\n").unwrap();
    let mut ed = Editor::from_text(
        &fs::read_to_string(&path).unwrap(),
        Some(path.clone()),
        Config::default(),
    );
    type_text(&mut ed, "not committed");
    press(&mut ed, &[Key::ctrl('c'), Key::ctrl('k')]);
    assert!(ed.prompt_for_quit().unwrap().contains("Abort commit"));
    press(&mut ed, &[Key::ch('y')]);
    assert!(ed.should_quit);
    assert_eq!(ed.exit_code, 1);
    // The file on disk must still contain the template only.
    assert_eq!(fs::read_to_string(&path).unwrap(), "# template\n");
}

#[test]
fn search_sets_mark_at_the_origin() {
    let mut ed = editor("alpha\nbeta\n");
    ed.buffer.set_point(nano_emacs::buffer::Point::new(1, 0));
    press(&mut ed, &[Key::ctrl('s')]);
    type_text(&mut ed, "alpha");
    press(&mut ed, &[ENTER]);
    assert_eq!(ed.buffer.point, nano_emacs::buffer::Point::new(0, 0));
    assert_eq!(ed.buffer.mark, Some(nano_emacs::buffer::Point::new(1, 0)));
}

#[test]
fn save_without_a_name_prompts_instead_of_panicking() {
    let mut ed = editor("anonymous");
    press(&mut ed, &[Key::ctrl('x'), Key::ctrl('s')]);
    assert!(ed.minibuffer.is_some());
    assert!(!ed.should_quit);
}

#[test]
fn toggle_line_numbers_command() {
    let mut ed = editor("one\ntwo\n");
    assert!(ed.view.line_numbers);
    press(&mut ed, &[Key::alt('x')]);
    type_text(&mut ed, "toggle-line-numbers");
    press(&mut ed, &[ENTER]);
    assert!(!ed.view.line_numbers);
}

#[test]
fn japanese_editing_and_kill_ring() {
    let mut ed = editor("日本語のテスト");
    ed.buffer.set_point(nano_emacs::buffer::Point::new(0, 3));
    press(&mut ed, &[Key::ctrl(' ')]);
    press(&mut ed, &[Key::ctrl('f'), Key::ctrl('f')]);
    press(&mut ed, &[Key::ctrl('w')]);
    assert_eq!(ed.buffer_text(), "日本語スト");
    press(&mut ed, &[Key::ctrl('y')]);
    assert_eq!(ed.buffer_text(), "日本語のテスト");
}
