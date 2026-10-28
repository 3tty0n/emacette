# emacette

**A tiny Emacs-flavoured terminal editor for quick edits, commit messages, and
reading code.**

emacette starts in milliseconds, runs in any terminal, and keeps the Emacs
habits you already have — `C-x C-s`, `C-s`, `M-x`, `C-SPC`, `C-w`/`M-w`, the
kill ring, and undo — without asking you to learn a modal editor or run a full
Emacs.

It is a focused tool, not an Emacs replacement: tabs and split panes, no Emacs
Lisp, and no plugin ecosystem. What it does have is fast startup, a small
binary, and just enough language support for everyday editing.

[Quick start](#quick-start) · [Features](#features) · [Documentation](#documentation) · [Contributing](#contributing)

## Quick start

### Prebuilt binary

```sh
curl -fsSL https://raw.githubusercontent.com/3tty0n/emacette/main/install.sh | sh
```

The installer detects your platform, verifies the checksum, and installs to
`~/.local/bin/emacette` by default.

### From source

```sh
cargo install --path .        # ~/.cargo/bin/emacette
```

Requires Rust 1.74 or newer. See [Installation](docs/user/install.md) for
`make install`, custom prefixes, uninstalling, and migration from nano-emacs.

### Run

```sh
emacette                 # scratch buffer
emacette FILE            # open or create a file
emacette +42 FILE        # start at line 42
emacette --help          # print the key bindings
```

A few keys to get going:

| Action | Key |
| --- | --- |
| Save / quit | `C-x C-s` / `C-x C-c` |
| Incremental search | `C-s` / `C-r` |
| Run a command | `M-x` |
| Set mark / select a region | `C-SPC` |
| Kill / copy / yank | `C-w` / `M-w` / `C-y` |
| Split below / right | `C-x 2` / `C-x 3` |
| Move to the other window | `C-x o` |
| Undo | `C-/` or `C-_` |
| Help | `F1` |

## Features

### Emacs-style editing

- Familiar key bindings for movement, marking, killing, yanking, and undo.
- A real minibuffer with history and completion for `M-x`, file names, and
  `write-file`.
- Mark and region support, a shared kill ring, and undo/redo grouped per
  command.
- Incremental search forward and backward, with case-sensitive matching when
  the query contains uppercase characters.
- In-editor help overlay (`F1`) covering the bindings and commands.

### Fast in a terminal

- Millisecond startup; a single binary with nothing else to install.
- Only changed screen rows are repainted, so editing stays responsive over
  SSH and inside tmux.
- Multiple files in tabs, with unsaved-tab protection and a tab bar.
- Window splits (`C-x 2`, `C-x 3`) with independent scrolling in each pane.
- Auto-indent and automatic delimiter pairs, plus rainbow parentheses in Lisp
  modes.
- Line numbers, trailing-whitespace highlighting, horizontal scrolling, soft
  wrapping, and auto-fill.
- Full-width CJK text, tabs, and other wide characters line up correctly.
- UTF-8 input; CRLF files keep their line endings.

### Language-aware, but lightweight

- Syntax highlighting for 40 [languages and formats](docs/user/languages.md).
- Editor completion from buffer words, with LSP completions when a server is
  configured; candidates appear in a popup at point and completion is
  automatic in programming-language files.
- Optional [language-server integration](docs/user/lsp.md) for inline
  diagnostics, a diagnostics popup, and completion.
- English [spell checking](docs/user/spelling.md) for prose such as Markdown,
  LaTeX, and commit messages.
- Per-language [configuration](docs/user/configuration.md) for indentation,
  comments, completion, spelling, diagnostics, and LSP.

### Fits into your workflow

- Git commit mode: `C-c C-c` finishes the commit and exits, `C-c C-k` aborts.
- [tmux integration](docs/user/tmux.md) keeps emacette's key bindings working
  inside a pane, and restores your tmux settings on exit.
- A plain-text configuration file with global settings and per-language
  sections.
- Eleven built-in [themes](docs/user/configuration.md) — dark, light,
  Solarized, Gruvbox, Nord, Dracula, Monokai, One Dark, and Tokyo Night.

## Git

```sh
git config --global core.editor emacette
```

- `C-c C-c` — save the message and finish the commit.
- `C-c C-k` — abort the commit without saving.
- `C-x C-s`, then `C-x C-c` — save and exit.

emacette also opens `MERGE_MSG`, `TAG_EDITMSG`, and `SQUASH_MSG` in commit
mode. See [Git integration](docs/user/git.md) for details.

## Configuration

Configuration is optional. When present, the file is read from
`$EMACETTE_CONFIG`, `$XDG_CONFIG_HOME/emacette/config`, or
`~/.config/emacette/config`:

```ini
theme = light
tab_width = 2

[markdown]
spell_check = true

[rust]
lsp_command = rust-analyzer
```

See the annotated [example configuration](examples/config) and the
[configuration reference](docs/user/configuration.md).

## Documentation

- **[User guide](docs/user/README.md)** — installation, key bindings, commands,
  windows, tabs, configuration, languages, completion, spelling, LSP, git,
  and tmux.
- **[Development guide](docs/development/README.md)** — architecture, testing,
  contribution workflow, benchmarks, and releases.
- **[Documentation index](docs/README.md)** — both guides in one place.

Popular pages: [Key bindings](docs/user/keys.md) ·
[Configuration](docs/user/configuration.md) ·
[Languages](docs/user/languages.md) ·
[LSP](docs/user/lsp.md) ·
[Architecture](docs/development/architecture.md).

## Contributing

Bug reports and pull requests are welcome. Start with
[CONTRIBUTING.md](CONTRIBUTING.md) and the
[development guide](docs/development/README.md). For a larger change, please
open an issue first so the scope can be discussed.

The project intentionally keeps its dependency footprint small and its
documentation split by audience: user-facing behavior belongs under
`docs/user/`, and internals under `docs/development/`.

## Limitations

- Split panes all show the active buffer; there is no separate buffer per
  pane.
- UTF-8 only; binary and non-UTF-8 files are rejected.
- Kill ring only; no system clipboard integration.
- LSP support covers diagnostics and completion only; no hover, rename, or
  formatting.
- Editor completion is prefix-based; there is no fuzzy matching.
- Spell checking is English-only and needs a word list or hunspell dictionary.
- No regex search, macros, rectangles, org-mode, or Emacs Lisp extension
  language.
- Not intended for very large files: a 200k-line, 7.6 MB file takes about
  100 ms to first paint.

## License

MIT. See [LICENSE](LICENSE).
