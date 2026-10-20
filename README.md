# emacette

A lightweight Emacs-flavoured terminal editor for git commit messages, file
viewing, and small edits. The binary is `emacette`.

## Features

- Single Rust binary; direct dependencies are `crossterm`, `serde_json`, and `unicode-width`.
- Emacs-style key bindings, minibuffer, mark/region, kill ring, and undo.
- Incremental rendering: only changed terminal rows are repainted.
- Syntax highlighting for 39 [languages and formats](docs/languages.md).
- CJK/full-width text, tab stops, horizontal scrolling, and line numbers.
- Multiple file [tabs](docs/tabs.md) with a tab bar and unsaved-tab protection.
- Auto-fill and optional [soft wrapping](docs/configuration.md#soft-wrapping)
  of long lines.
- Minibuffer completion lists and optional [editor completion](docs/completion.md).
- English [spell checking](docs/spelling.md) for prose modes such as Markdown
  and LaTeX.
- Per-language [configuration](docs/configuration.md) sections for indentation,
  comments, completion, spelling, and LSP.
- Optional [language-server integration](docs/lsp.md) with inline diagnostics,
  a diagnostics popup, and LSP completion.
- Git commit mode with save and abort shortcuts.
- UTF-8 input; CRLF line endings are preserved.

## Quick start

```sh
cargo install --path .        # ~/.cargo/bin/emacette
emacette FILE
emacette +42 FILE
```

See [Installation](docs/install.md) for prebuilt binaries, `install.sh`, and
uninstall instructions.

## Usage

- `emacette` — start with a scratch buffer.
- `emacette FILE` — open a file, or create it.
- `emacette +LINE FILE` — start at a line.
- `emacette --help` — print the key bindings.

## Git

```sh
git config --global core.editor emacette
```

- `C-c C-c` — save the message and exit.
- `C-c C-k` — exit without saving; git aborts the commit.
- `C-x C-s`, then `C-x C-c` — save and exit.

See [Git integration](docs/git.md) for the full commit workflow.

## Documentation

- [Key bindings](docs/keys.md) — full key list, regions, search, and minibuffer editing.
- [Tabs](docs/tabs.md) — multiple files, switching, and closing tabs.
- [Commands](docs/commands.md) — `M-x` commands for files, regions, display, and spelling.
- [Completion](docs/completion.md) — minibuffer lists and optional editor completion.
- [Spelling](docs/spelling.md) — English dictionaries, highlighting, and corrections.
- [LSP](docs/lsp.md) — language-server setup, diagnostics, and completion.
- [Configuration](docs/configuration.md) — global and per-language options, auto-fill, wrapping, completion, spelling, and diagnostics.
- [Languages](docs/languages.md) — supported languages and detection rules.
- [Installation](docs/install.md) — source installs, prebuilt binaries, and the installer.
- [Git integration](docs/git.md) — editor setup and commit behaviour.
- [Development and releases](docs/development.md) — build, test, benchmark, and release.

## Renaming from nano-emacs

The project and binary are now `emacette`. Existing setups keep working where
practical:

- The new config path is `~/.config/emacette/config`.
- If it does not exist, the old `~/.config/nano-emacs/config` is read.
- `EMACETTE_CONFIG` and `EMACETTE_DICT` replace `NANO_EMACS_CONFIG` and
  `NANO_EMACS_DICT`; the old variables are still accepted as fallbacks.
- Update `git config --global core.editor` and any scripts that invoked
  `nemacs` to use `emacette`.
- `cargo uninstall nano-emacs` removes an old cargo-installed binary.

## Limitations

- One window with tabs; no window splits or multiple panes.
- UTF-8 only; binary and non-UTF-8 files are rejected.
- Kill ring only; no system clipboard integration.
- Editor completion falls back to buffer words without LSP; there is no fuzzy matching.
- LSP support covers diagnostics and completion only; no hover, rename, or formatting.
- Spell checking is English-only and needs a word list or hunspell dictionary.
- No regex search, macros, rectangles, org-mode, or Emacs Lisp.
- Not intended for very large files: a 200k-line, 7.6 MB file takes about
  100 ms to first paint.

## License

MIT. See [LICENSE](LICENSE).
