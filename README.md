# my-emacs

A lightweight Emacs-flavoured terminal editor for git commit messages, file
viewing, and small edits.

- Single Rust binary. Direct dependencies: `crossterm`, `unicode-width`.
- Emacs key bindings, minibuffer, mark, kill ring, undo.
- Incremental rendering: only changed rows reach the terminal.
- Syntax highlighting for 17 languages and formats, detected by extension or
  shebang.
- CJK/full-width text, tabs, horizontal scrolling, line numbers.
- Git commit mode: `C-c C-c` saves and exits, `C-c C-k` aborts with exit code 1.
- UTF-8 input; CRLF line endings are preserved.

## Performance

Median time from process start to first terminal output, 44-line Rust file,
9 runs, Intel Core Ultra 7 155U / Arch Linux. Values vary by about 20%.

| Editor | First paint |
|---|---:|
| my-emacs | 2.7 ms |
| nano | 3.9 ms |
| vim | 5.3 ms |
| emacs -nw -Q | 80.4 ms |

Cursor motion writes about 150 bytes per keystroke; a 100-key burst is
coalesced into one frame of about 200 bytes. `make bench` reproduces these
numbers.

## Install

Rust 1.74 or newer.

```sh
cargo build --release
install -Dm755 target/release/my-emacs ~/.local/bin/my-emacs
```

Alternatives:

```sh
cargo install --path .
make install                 # PREFIX=/usr/local to override
```

`~/.local/bin` must be on `PATH`. To run without installing:

```sh
./target/release/my-emacs README.md
```

## Usage

```sh
my-emacs                # scratch buffer
my-emacs FILE           # open a file, or create it
my-emacs +42 FILE       # start at line 42
my-emacs --help         # print key bindings
```

## Git

```sh
git config --global core.editor my-emacs
git config --global core.editor "$HOME/.local/bin/my-emacs"   # not on PATH
GIT_EDITOR=my-emacs git commit                                 # one-off
```

Commit workflow:

- `C-c C-c` saves the message and exits.
- `C-c C-k` exits without saving; git aborts the commit.
- `C-x C-s` followed by `C-x C-c` also saves and exits.

The same editor serves `git commit --amend`, `git tag -a`, `git rebase -i`,
and every other command that invokes `$EDITOR`. `C-c C-c` asks for confirmation
when the message is empty or contains only comments.

Set `comment_char` to match `core.commentChar` when it is not `#`.

## Keys

`C-` is Ctrl. `M-` is Alt, or ESC followed by the key. `F1` lists bindings in
the editor.

| Key | Action |
|---|---|
| `C-x C-s` | Save |
| `C-x C-c` | Quit; confirm unsaved changes |
| `C-x C-w` | Write file |
| `C-x C-f` | Find file |
| `C-c C-c` / `C-c C-k` | Finish / abort a git commit |
| `C-f` `C-b` `C-n` `C-p` | Move by character or line |
| `M-f` `M-b` | Move by word |
| `C-a` `C-e` | Line start / end |
| `M-<` `M->` | Buffer start / end |
| `C-v` `M-v` | Page down / up |
| `M-g g` | Go to line |
| `C-l` | Recenter and repaint |
| `C-d` `DEL` | Delete forward / backward |
| `M-d` `M-DEL` | Kill word |
| `C-k` | Kill to end of line |
| `C-y` `M-y` | Yank / yank-pop |
| `C-SPC` `C-w` `M-w` | Set mark / kill region / copy region |
| `C-x C-x` `C-x h` | Exchange point and mark / mark buffer |
| `C-t` | Transpose characters |
| `M-u` `M-l` `M-c` | Upcase / downcase / capitalize |
| `C-o` | Open a line |
| `C-/` `C-_` | Undo; `M-x redo` redoes |
| `TAB` `S-TAB` | Indent / unindent |
| `C-s` `C-r` | Incremental search forward / backward |
| `C-g` | Cancel, deactivate mark |
| `M-x` | Run a command by name |
| `F1` | Help |

During search, `C-s` / `C-r` repeat, `RET` accepts and sets mark, `C-g`
restores the original point. Matching is case-insensitive unless the query
contains an uppercase character.

## Commands

`M-x` runs `save-buffer`, `write-file`, `find-file`, `kill-emacs`, `undo`,
`redo`, `goto-line`, `toggle-line-numbers`, `toggle-trailing-whitespace`,
`delete-trailing-whitespace`, `revert-buffer`, `search-forward`,
`search-backward`, `beginning-of-buffer`, `end-of-buffer`, and `help`.
`TAB` completes command names and file paths.

## Configuration

Read from the first existing path:

1. `$MY_EMACS_CONFIG`
2. `$XDG_CONFIG_HOME/my-emacs/config`
3. `~/.config/my-emacs/config`

Format: `key = value`, one per line; `#` starts a comment; unknown keys are
ignored.

```ini
tab_width = 4
line_numbers = true
highlight_trailing_whitespace = true
auto_indent = true
theme = dark                 # dark or light
comment_char = "#"           # match core.commentChar
confirm_on_quit = true
```

## Languages

Rust, Go, C, C++, Python, JavaScript, TypeScript, Shell, Markdown, JSON, TOML,
YAML, Makefile, HTML, CSS, Diff, and Git commit messages. Detection uses the
file extension, then the shebang. `COMMIT_EDITMSG`, `MERGE_MSG`, `TAG_EDITMSG`,
and `SQUASH_MSG` open in GitCommit mode.

## Limitations

- One buffer; no window splits or multiple buffers.
- UTF-8 only; binary and non-UTF-8 files are rejected.
- Kill ring only; no system clipboard integration.
- No regex search, macros, rectangles, org-mode, or Emacs Lisp.
- Not intended for very large files: a 200k-line, 7.6 MB file takes about 100 ms
  to first paint.

## Development

```sh
cargo test        # unit and integration tests
make smoke        # PTY end-to-end tests, including a real git commit
make bench        # startup comparison
make check        # rustfmt check and tests
```

## License

MIT
