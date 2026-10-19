#!/usr/bin/env python3
"""End-to-end tests driven through a real pseudo-terminal.

Run with:  make e2e   (or: python3 scripts/e2e.py [path-to-binary])
"""

import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, HERE)

from ptyharness import run  # noqa: E402
from screen import screen_from_ansi  # noqa: E402

BIN = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "target", "release", "nemacs"))
COLS, ROWS = 80, 20

failures = []


def check(name, condition, detail=""):
    if condition:
        print(f"PASS  {name}")
    else:
        print(f"FAIL  {name}" + (f"\n      {detail}" if detail else ""))
        failures.append(name)


def tmpfile(text, name="note.txt"):
    directory = tempfile.mkdtemp(prefix="nano-emacs-e2e-")
    path = os.path.join(directory, name)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    return path


def body_screen(output):
    return screen_from_ansi(output, COLS, ROWS)


def test_basic_edit_and_save():
    path = tmpfile("first line\n")
    code, out, _ = run(
        [BIN, path],
        [b"hello\n", b"\x18\x13", b"\x18\x03"],
        cols=COLS,
        rows=ROWS,
        expect=b"nemacs",
    )
    with open(path, encoding="utf-8") as handle:
        content = handle.read()
    check("edit/save/quit exits 0", code == 0, f"exit={code}")
    check("typed text is saved", content == "hello\nfirst line\n", repr(content))
    check("modeline shows file name and mode", "note.txt" in body_screen(out) and "Plain" in body_screen(out))


def test_japanese_input():
    path = tmpfile("")
    text = "日本語のテスト"
    code, out, _ = run(
        [BIN, path],
        [text.encode("utf-8"), b"\x18\x13", b"\x18\x03"],
        cols=COLS,
        rows=ROWS,
        expect=b"nemacs",
    )
    with open(path, encoding="utf-8") as handle:
        content = handle.read()
    check("japanese input is saved", code == 0 and content == text, repr(content))
    check("japanese is rendered", "日本語のテスト" in body_screen(out))


def test_undo_key():
    path = tmpfile("")
    code, out, _ = run(
        [BIN, path],
        [b"abc", b"\x1f", b"\x18\x13", b"\x18\x03"],
        cols=COLS,
        rows=ROWS,
        expect=b"nemacs",
    )
    with open(path, encoding="utf-8") as handle:
        content = handle.read()
    check("C-_ (0x1f) undoes typing", code == 0 and content == "", repr(content))


def test_mark_and_kill_region():
    path = tmpfile("hello world\n")
    code, out, _ = run(
        [BIN, path],
        [b"\x00", b"\x06" * 5, b"\x17", b"\x18\x13", b"\x18\x03"],
        cols=COLS,
        rows=ROWS,
        expect=b"nemacs",
    )
    with open(path, encoding="utf-8") as handle:
        content = handle.read()
    check("C-SPC / C-w kill a region", code == 0 and content == " world\n", repr(content))


def test_escape_meta_word_motion():
    path = tmpfile("")
    code, out, _ = run(
        [BIN, path],
        [b"foo bar", b"\x01", b"\x1bf", b"X", b"\x18\x13", b"\x18\x03"],
        cols=COLS,
        rows=ROWS,
        expect=b"nemacs",
    )
    with open(path, encoding="utf-8") as handle:
        content = handle.read()
    check("ESC-prefixed M-f moves by word", code == 0 and content == "fooX bar", repr(content))


def test_search_and_insert():
    path = tmpfile("alpha\nbeta\ngamma\n")
    code, out, _ = run(
        [BIN, path],
        [b"\x13", b"gamma", b"\r", b"!", b"\x18\x13", b"\x18\x03"],
        cols=COLS,
        rows=ROWS,
        expect=b"nemacs",
    )
    with open(path, encoding="utf-8") as handle:
        content = handle.read()
    check("isearch moves point", code == 0 and content == "alpha\nbeta\n!gamma\n", repr(content))
    check("isearch prompt is visible", "I-search: gamma" in out.decode("utf-8", "replace"))


def test_help_overlay():
    path = tmpfile("")
    # F1 (xterm), q to close, then quit.
    code, out, _ = run(
        [BIN, path],
        [b"\x1bOP", b"q", b"\x18\x03"],
        cols=COLS,
        rows=ROWS,
        expect=b"nemacs",
    )
    check(
        "help opens with F1",
        code == 0 and b"Files and git" in out,
        "help overlay was never drawn",
    )


def test_git_commit():
    repo = tempfile.mkdtemp(prefix="nano-emacs-git-")
    try:
        subprocess.run(["git", "-C", repo, "init", "-q"], check=True)
        subprocess.run(["git", "-C", repo, "config", "user.name", "Test"], check=True)
        subprocess.run(["git", "-C", repo, "config", "user.email", "test@example.com"], check=True)
        with open(os.path.join(repo, "hello.txt"), "w", encoding="utf-8") as handle:
            handle.write("hello\n")
        subprocess.run(["git", "-C", repo, "add", "hello.txt"], check=True)
        code, out, _ = run(
            ["git", "-C", repo, "-c", f"core.editor={BIN}", "commit"],
            [b"e2e: commit message\n", b"\x03\x03"],
            cols=COLS,
            rows=ROWS,
            settle=0.6,
            timeout=20,
            expect=b"COMMIT_EDITMSG",
        )
        log = subprocess.run(
            ["git", "-C", repo, "log", "-1", "--format=%s"],
            capture_output=True,
            text=True,
        )
        subject = log.stdout.strip()
        check("git commit runs the editor", b"COMMIT_EDITMSG" in out)
        check("git commit succeeds", code == 0, f"exit={code} git-log={log.stderr!r}")
        check("commit message is recorded", subject == "e2e: commit message", repr(subject))
        check("git template comments are shown", b"# Please enter the commit message" in out)
    finally:
        shutil.rmtree(repo, ignore_errors=True)


def test_git_empty_message_warning():
    repo = tempfile.mkdtemp(prefix="nano-emacs-git-")
    try:
        subprocess.run(["git", "-C", repo, "init", "-q"], check=True)
        subprocess.run(["git", "-C", repo, "config", "user.name", "Test"], check=True)
        subprocess.run(["git", "-C", repo, "config", "user.email", "test@example.com"], check=True)
        with open(os.path.join(repo, "hello.txt"), "w", encoding="utf-8") as handle:
            handle.write("hello\n")
        subprocess.run(["git", "-C", repo, "add", "hello.txt"], check=True)
        code, out, _ = run(
            ["git", "-C", repo, "-c", f"core.editor={BIN}", "commit"],
            # Empty message -> warning; say no, type a message, finish.
            [b"\x03\x03", b"n", b"after warning\n", b"\x03\x03"],
            cols=COLS,
            rows=ROWS,
            settle=0.6,
            timeout=20,
            expect=b"COMMIT_EDITMSG",
        )
        log = subprocess.run(
            ["git", "-C", repo, "log", "-1", "--format=%s"],
            capture_output=True,
            text=True,
        )
        subject = log.stdout.strip()
        check("empty message warning is shown", b"Empty commit message" in out)
        check("commit after warning succeeds", code == 0 and subject == "after warning", repr(subject))
    finally:
        shutil.rmtree(repo, ignore_errors=True)


def test_git_abort():
    repo = tempfile.mkdtemp(prefix="nano-emacs-git-")
    try:
        subprocess.run(["git", "-C", repo, "init", "-q"], check=True)
        subprocess.run(["git", "-C", repo, "config", "user.name", "Test"], check=True)
        subprocess.run(["git", "-C", repo, "config", "user.email", "test@example.com"], check=True)
        with open(os.path.join(repo, "hello.txt"), "w", encoding="utf-8") as handle:
            handle.write("hello\n")
        subprocess.run(["git", "-C", repo, "add", "hello.txt"], check=True)
        code, out, _ = run(
            ["git", "-C", repo, "-c", f"core.editor={BIN}", "commit"],
            [b"will be discarded\n", b"\x03\x0b", b"y"],
            cols=COLS,
            rows=ROWS,
            settle=0.6,
            timeout=20,
            expect=b"COMMIT_EDITMSG",
        )
        has_commits = subprocess.run(
            ["git", "-C", repo, "rev-list", "-n", "1", "HEAD"],
            capture_output=True,
        ).returncode == 0
        check("C-c C-k asks before aborting", b"Abort commit" in out)
        check("abort exits non-zero", code != 0, f"exit={code}")
        check("abort creates no commit", not has_commits)
    finally:
        shutil.rmtree(repo, ignore_errors=True)


def test_plus_line_argument():
    path = tmpfile("one\ntwo\nthree\n")
    code, out, _ = run(
        [BIN, "+2", path],
        [b"\x18\x03"],
        cols=COLS,
        rows=ROWS,
        expect=b"nemacs",
    )
    check("+LINE positions the cursor", code == 0 and b"L2 C1" in out)


def test_cli():
    version = subprocess.run([BIN, "--version"], capture_output=True, text=True)
    check(
        "--version works",
        version.returncode == 0 and "nemacs" in version.stdout,
        repr(version.stdout),
    )
    help_text = subprocess.run([BIN, "--help"], capture_output=True, text=True)
    check(
        "--help lists key bindings",
        help_text.returncode == 0 and "C-x C-s" in help_text.stdout,
        repr(help_text.stdout[:80]),
    )
    bad = subprocess.run([BIN, "--definitely-not-an-option"], capture_output=True, text=True)
    check("unknown options are rejected", bad.returncode != 0)


def test_not_a_tty():
    result = subprocess.run(
        [BIN, "/tmp/whatever.txt"],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    check(
        "refuses to run without a tty",
        result.returncode != 0 and b"not a terminal" in result.stderr,
        repr(result.stderr),
    )


def main():
    if not os.path.exists(BIN):
        print(f"binary not found: {BIN}\nrun `cargo build --release` first")
        return 2
    print(f"testing {BIN}")
    test_basic_edit_and_save()
    test_japanese_input()
    test_undo_key()
    test_mark_and_kill_region()
    test_escape_meta_word_motion()
    test_search_and_insert()
    test_help_overlay()
    test_git_commit()
    test_git_empty_message_warning()
    test_git_abort()
    test_plus_line_argument()
    test_cli()
    test_not_a_tty()
    print()
    if failures:
        print(f"{len(failures)} test(s) failed: {', '.join(failures)}")
        return 1
    print("all e2e tests passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
