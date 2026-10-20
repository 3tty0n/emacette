# nano-emacs

A lightweight Emacs-flavoured terminal editor for git commit messages, file
viewing, and small edits. The binary is `nemacs`.

## Features

- Single Rust binary; direct dependencies are `crossterm` and `unicode-width`.
- Emacs-style key bindings, minibuffer, mark/region, kill ring, and undo.
- Incremental rendering: only changed terminal rows are repainted.
- Syntax highlighting for 39 [languages and formats](docs/languages.md).
- CJK/full-width text, tabs, horizontal scrolling, and line numbers.
- Auto-fill and optional [soft wrapping](docs/configuration.md#soft-wrapping)
  of long lines.
- Minibuffer completion lists and optional [editor completion](docs/completion.md).
- English [spell checking](docs/spelling.md) for prose modes such as Markdown
  and LaTeX.
- Git commit mode with save and abort shortcuts.
- UTF-8 input; CRLF line endings are preserved.

## Quick start

```sh
cargo install --path .        # ~/.cargo/bin/nemacs
nemacs FILE
nemacs +42 FILE
```

See [Installation](docs/install.md) for prebuilt binaries, `install.sh`, and
uninstall instructions.

## Usage

- `nemacs` — start with a scratch buffer.
- `nemacs FILE` — open a file, or create it.
- `nemacs +LINE FILE` — start at a line.
- `nemacs --help` — print the key bindings.

## Git

```sh
git config --global core.editor nemacs
```

- `C-c C-c` — save the message and exit.
- `C-c C-k` — exit without saving; git aborts the commit.
- `C-x C-s`, then `C-x C-c` — save and exit.

See [Git integration](docs/git.md) for the full commit workflow.

## Documentation

- [Key bindings](docs/keys.md) — full key list, regions, search, and minibuffer editing.
- [Commands](docs/commands.md) — `M-x` commands for files, regions, display, and spelling.
- [Completion](docs/completion.md) — minibuffer lists and optional editor completion.
- [Spelling](docs/spelling.md) — English dictionaries, highlighting, and corrections.
- [Configuration](docs/configuration.md) — config paths, options, auto-fill, wrapping, completion, and spelling.
- [Languages](docs/languages.md) — supported languages and detection rules.
- [Installation](docs/install.md) — source installs, prebuilt binaries, and the installer.
- [Git integration](docs/git.md) — editor setup and commit behaviour.
- [Development and releases](docs/development.md) — build, test, benchmark, and release.

## Limitations

- One buffer; no window splits or multiple buffers.
- UTF-8 only; binary and non-UTF-8 files are rejected.
- Kill ring only; no system clipboard integration.
- Editor completion is buffer-local; there is no language server or fuzzy matching.
- Spell checking is English-only and needs a word list or hunspell dictionary.
- No regex search, macros, rectangles, org-mode, or Emacs Lisp.
- Not intended for very large files: a 200k-line, 7.6 MB file takes about
  100 ms to first paint.

## License

MIT. See [LICENSE](LICENSE).
