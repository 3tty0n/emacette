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

BIN = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "target", "release", "emacette"))
COLS, ROWS = 80, 20

failures = []


def check(name, condition, detail=""):
    if condition:
        print(f"PASS  {name}")
    else:
        print(f"FAIL  {name}" + (f"\n      {detail}" if detail else ""))
        failures.append(name)


def tmpfile(text, name="note.txt"):
    directory = tempfile.mkdtemp(prefix="emacette-e2e-")
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
        expect=b"emacette",
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
        expect=b"emacette",
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
        expect=b"emacette",
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
        expect=b"emacette",
    )
    with open(path, encoding="utf-8") as handle:
        content = handle.read()
    check("C-SPC / C-w kill a region", code == 0 and content == " world\n", repr(content))


def test_region_delete_and_copy():
    path = tmpfile("hello world\n")
    code, out, _ = run(
        [BIN, path],
        [b"\x00", b"\x06" * 5, b"\x7f", b"\x18\x13", b"\x18\x03"],
        cols=COLS,
        rows=ROWS,
        expect=b"emacette",
    )
    with open(path, encoding="utf-8") as handle:
        content = handle.read()
    check(
        "DEL deletes the active region",
        code == 0 and content == " world\n",
        repr(content),
    )

    path = tmpfile("hello world\n")
    code, out, _ = run(
        [BIN, path],
        [b"\x00", b"\x06" * 5, b"\x1bw", b"\x05", b"\x19", b"\x18\x13", b"\x18\x03"],
        cols=COLS,
        rows=ROWS,
        expect=b"emacette",
    )
    with open(path, encoding="utf-8") as handle:
        content = handle.read()
    check(
        "M-w copies a region and C-y yanks it",
        code == 0 and content == "hello worldhello\n",
        repr(content),
    )


def test_find_file_tab_completion():
    directory = tempfile.mkdtemp(prefix="emacette-e2e-")
    start = os.path.join(directory, "note.txt")
    with open(start, "w", encoding="utf-8") as handle:
        handle.write("start\n")
    for name in ("alpha.txt", "alphabet.txt"):
        with open(os.path.join(directory, name), "w", encoding="utf-8") as handle:
            handle.write(f"{name}\n")
    code, out, _ = run(
        [BIN, start],
        [
            b"\x18\x06",  # C-x C-f
            b"\x01\x0b",  # C-a C-k: clear the initial directory
            b"al",
            b"\t",
            b"\t",
            b"\r",
            b"\x18\x03",
        ],
        cols=COLS,
        rows=ROWS,
        expect=b"emacette",
    )
    screen = body_screen(out)
    check(
        "find-file TAB cycles candidates",
        code == 0 and "alphabet" in screen,
        repr(screen[-300:]),
    )
    check(
        "completion menu lists candidates",
        b"alpha.txt" in out and b"alphabet.txt" in out,
        repr(out[-400:]),
    )


def test_minibuffer_menus_open_immediately():
    directory = tempfile.mkdtemp(prefix="emacette-e2e-")
    path = os.path.join(directory, "note.txt")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write("note\n")
    with open(os.path.join(directory, "zzz-immediate-candidate.txt"), "w", encoding="utf-8") as handle:
        handle.write("candidate\n")
    previous = os.getcwd()
    os.chdir(directory)
    try:
        code, out, _ = run(
            [BIN, path],
            [b"\x1bx", b"\x07", b"\x18\x06", b"\x07", b"\x18\x03"],
            cols=COLS,
            rows=ROWS,
            expect=b"emacette",
            key_delay=0.15,
        )
    finally:
        os.chdir(previous)
    check(
        "M-x and find-file list candidates immediately",
        code == 0
        and b"save-buffers-kill-terminal" in out
        and b"zzz-immediate-candidate.txt" in out,
        f"mx={b'save-buffers-kill-terminal' in out} file={b'zzz-immediate-candidate.txt' in out}",
    )


def test_minibuffer_completion_ctrl_n():
    directory = tempfile.mkdtemp(prefix="emacette-e2e-")
    start = os.path.join(directory, "note.txt")
    with open(start, "w", encoding="utf-8") as handle:
        handle.write("start\n")
    for name, content in (("alpha.txt", "alpha-content\n"), ("alphabet.txt", "alphabet-content\n")):
        with open(os.path.join(directory, name), "w", encoding="utf-8") as handle:
            handle.write(content)
    code, out, _ = run(
        [BIN, start],
        [
            b"\x18\x06",  # C-x C-f
            b"\x01\x0b",  # clear prompt
            b"al",
            b"\x0e",      # C-n: select the first candidate
            b"\r",
            b"\x18\x03",
        ],
        cols=COLS,
        rows=ROWS,
        expect=b"emacette",
    )
    screen = body_screen(out)
    check(
        "minibuffer C-n selects a candidate",
        code == 0 and "alpha-content" in screen,
        repr(screen[-300:]),
    )


def test_lsp_diagnostics_and_completion():
    # Keep the fake server inside the workspace: some sandboxes give child
    # processes a different /tmp view, which would hide the script.
    target = os.path.join(ROOT, "target")
    os.makedirs(target, exist_ok=True)
    directory = tempfile.mkdtemp(prefix="emacette-e2e-lsp-", dir=target)
    server = os.path.join(directory, "fake_lsp.py")
    with open(server, "w", encoding="utf-8") as handle:
        handle.write(
            """import json, sys

def send(obj):
    body = json.dumps(obj).encode()
    sys.stdout.buffer.write(b"Content-Length: " + str(len(body)).encode() + b"\\r\\n\\r\\n" + body)
    sys.stdout.buffer.flush()

while True:
    headers = {}
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            sys.exit(0)
        if line in (b"\\r\\n", b"\\n"):
            break
        if b":" in line:
            key, value = line.decode().split(":", 1)
            headers[key.strip().lower()] = value.strip()
    body = sys.stdin.buffer.read(int(headers.get("content-length", "0")))
    if not body:
        sys.exit(0)
    message = json.loads(body)
    if message.get("method") == "initialize" and "id" in message:
        send({"jsonrpc": "2.0", "id": message["id"], "result": {"capabilities": {"completionProvider": {}, "textDocumentSync": 1}}})
    elif message.get("method") == "textDocument/didOpen":
        uri = message["params"]["textDocument"]["uri"]
        send({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": {"uri": uri, "diagnostics": [{"range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 5}}, "severity": 1, "message": "fake error", "source": "fake"}]}})
    elif message.get("method") == "textDocument/completion" and "id" in message:
        send({"jsonrpc": "2.0", "id": message["id"], "result": {"items": [{"label": "alpha", "insertText": "alpha"}, {"label": "alphabet", "insertText": "alphabet"}]}})
"""
        )
    config = os.path.join(directory, "config")
    with open(config, "w", encoding="utf-8") as handle:
        handle.write(
            "[plain]\n"
            "editor_completion = true\n"
            f"lsp_command = python3 {server}\n"
            "lsp_language_id = plaintext\n"
        )
    path = os.path.join(directory, "lsp.txt")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write("hello wurld\n")
    previous = os.environ.get("EMACETTE_CONFIG")
    os.environ["EMACETTE_CONFIG"] = config
    try:
        code, out, _ = run(
            [BIN, path],
            [b"\x05", b"al", b"\x18\x13", b"\x18\x03"],
            cols=COLS,
            rows=ROWS,
            expect=b"emacette",
            key_delay=0.5,
        )
    finally:
        if previous is None:
            os.environ.pop("EMACETTE_CONFIG", None)
        else:
            os.environ["EMACETTE_CONFIG"] = previous
    with open(path, encoding="utf-8") as handle:
        content = handle.read()
    check(
        "LSP diagnostics and completion render",
        code == 0
        and content == "hello wurldal\n"
        and b";4;38;5;203" in out
        and b"E: fake error" in out
        and b"> alpha" in out,
        f"content={content!r} diag={b'E: fake error' in out} menu={b'> alpha' in out}",
    )
    shutil.rmtree(directory, ignore_errors=True)


def test_code_completion_is_automatic():
    directory = tempfile.mkdtemp(prefix="nano-emacs-e2e-")
    path = os.path.join(directory, "main.rs")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write("alpha alphabet\n")
    code, out, _ = run(
        [BIN, path],
        [b"\x01", b"al", b"\x07", b"\x18\x03", b"n"],
        cols=COLS,
        rows=ROWS,
        expect=b"emacette",
        key_delay=0.1,
    )
    check(
        "code files enable completion without configuration",
        code == 0 and b"> alalpha" in out,
        repr(out[-400:]),
    )
    shutil.rmtree(directory, ignore_errors=True)


def test_editor_completion_menu():
    directory = tempfile.mkdtemp(prefix="emacette-e2e-")
    config = os.path.join(directory, "config")
    with open(config, "w", encoding="utf-8") as handle:
        handle.write("editor_completion = true\n")
    path = os.path.join(directory, "doc.txt")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write("alpha alphabet \n")
    previous = os.environ.get("EMACETTE_CONFIG")
    os.environ["EMACETTE_CONFIG"] = config
    try:
        code, out, _ = run(
            [BIN, path],
            [b"\x05", b"al", b"\t", b"\x18\x13", b"\x18\x03"],
            cols=COLS,
            rows=ROWS,
            expect=b"emacette",
        )
    finally:
        if previous is None:
            os.environ.pop("EMACETTE_CONFIG", None)
        else:
            os.environ["EMACETTE_CONFIG"] = previous
    with open(path, encoding="utf-8") as handle:
        content = handle.read()
    check(
        "editor completion menu accepts a candidate",
        code == 0 and content == "alpha alphabet alpha\n" and b"> alpha" in out,
        repr(content),
    )


def test_spell_check():
    directory = tempfile.mkdtemp(prefix="emacette-e2e-")
    dictionary = os.path.join(directory, "words")
    with open(dictionary, "w", encoding="utf-8") as handle:
        handle.write("hello\nworld\n")
    config = os.path.join(directory, "config")
    with open(config, "w", encoding="utf-8") as handle:
        handle.write(f"spell_check = true\nspell_dictionary = {dictionary}\n")
    path = os.path.join(directory, "doc.md")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write("hello wurld\n")
    previous = os.environ.get("EMACETTE_CONFIG")
    os.environ["EMACETTE_CONFIG"] = config
    try:
        code, out, _ = run(
            [BIN, path],
            [b"\x1bxnext-spelling-error\r", b"\x1b$", b"\r", b"\x18\x13", b"\x18\x03"],
            cols=COLS,
            rows=ROWS,
            expect=b"emacette",
        )
    finally:
        if previous is None:
            os.environ.pop("EMACETTE_CONFIG", None)
        else:
            os.environ["EMACETTE_CONFIG"] = previous
    with open(path, encoding="utf-8") as handle:
        content = handle.read()
    check(
        "spell check underlines and suggests words",
        code == 0
        and content == "hello world\n"
        and b";4;38;5;203" in out
        and b"Replace 'wurld' with: " in out,
        repr(content),
    )


def test_tabs_switch_files():
    directory = tempfile.mkdtemp(prefix="emacette-e2e-")
    alpha = os.path.join(directory, "alpha.txt")
    beta = os.path.join(directory, "beta.txt")
    with open(alpha, "w", encoding="utf-8") as handle:
        handle.write("alpha-content\n")
    with open(beta, "w", encoding="utf-8") as handle:
        handle.write("beta-content\n")
    previous = os.getcwd()
    os.chdir(directory)
    try:
        code, out, _ = run(
            [BIN, alpha],
            [
                b"\x18\x06",  # C-x C-f
                b"\x01\x0b",  # clear prompt
                b"beta.txt",
                b"\r",
                b"\x18t", b"p",  # C-x t p: previous tab
                b"\x18\x03",
            ],
            cols=COLS,
            rows=ROWS,
            expect=b"emacette",
            key_delay=0.1,
        )
    finally:
        os.chdir(previous)
    screen = body_screen(out)
    check(
        "tabs switch between open files",
        code == 0
        and "alpha-content" in screen
        and b"beta-content" in out
        and b"alpha.txt" in out
        and b"beta.txt" in out,
        repr(screen[-400:]),
    )
    shutil.rmtree(directory, ignore_errors=True)


def test_quit_without_changes_is_immediate():
    directory = tempfile.mkdtemp(prefix="emacette-e2e-")
    path = os.path.join(directory, "note.txt")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write("hello\n")
    code, out, _ = run(
        [BIN, path],
        [b"X", b"\x1f", b"\x18\x03"],
        cols=COLS,
        rows=ROWS,
        expect=b"emacette",
    )
    with open(path, encoding="utf-8") as handle:
        content = handle.read()
    check(
        "quit after undoing all changes is immediate",
        code == 0 and content == "hello\n" and b"Save changes" not in out,
        repr(out[-300:]),
    )

    new_path = os.path.join(directory, "new.txt")
    code, out, _ = run(
        [BIN, new_path],
        [b"x", b"\x7f", b"\x18\x03"],
        cols=COLS,
        rows=ROWS,
        expect=b"emacette",
    )
    check(
        "empty new file quits without prompting",
        code == 0 and not os.path.exists(new_path) and b"Save this buffer" not in out,
        repr(out[-300:]),
    )
    shutil.rmtree(directory, ignore_errors=True)


def test_escape_meta_word_motion():
    path = tmpfile("")
    code, out, _ = run(
        [BIN, path],
        [b"foo bar", b"\x01", b"\x1bf", b"X", b"\x18\x13", b"\x18\x03"],
        cols=COLS,
        rows=ROWS,
        expect=b"emacette",
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
        expect=b"emacette",
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
        expect=b"emacette",
    )
    check(
        "help opens with F1",
        code == 0 and b"Files and git" in out,
        "help overlay was never drawn",
    )


def test_git_commit():
    repo = tempfile.mkdtemp(prefix="emacette-git-")
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
    repo = tempfile.mkdtemp(prefix="emacette-git-")
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
    repo = tempfile.mkdtemp(prefix="emacette-git-")
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
        expect=b"emacette",
    )
    check("+LINE positions the cursor", code == 0 and b"L2 C1" in out)


def test_cli():
    version = subprocess.run([BIN, "--version"], capture_output=True, text=True)
    check(
        "--version works",
        version.returncode == 0 and "emacette" in version.stdout,
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
    test_region_delete_and_copy()
    test_find_file_tab_completion()
    test_minibuffer_menus_open_immediately()
    test_minibuffer_completion_ctrl_n()
    test_code_completion_is_automatic()
    test_lsp_diagnostics_and_completion()
    test_editor_completion_menu()
    test_spell_check()
    test_tabs_switch_files()
    test_quit_without_changes_is_immediate()
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
