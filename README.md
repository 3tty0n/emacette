# nano-emacs

A lightweight Emacs-flavoured terminal editor for git commit messages, file
viewing, and small edits. The binary is `nemacs`.

- Single Rust binary. Direct dependencies: `crossterm`, `unicode-width`.
- Emacs key bindings, minibuffer, mark, kill ring, undo.
- Incremental rendering: only changed rows reach the terminal.
- Syntax highlighting for 39 languages and formats, detected by file name,
  extension, or shebang.
- CJK/full-width text, tabs, horizontal scrolling, line numbers.
- Optional auto-fill (hard-wraps typed text at `fill_column`) and soft wrapping
  of long lines (`truncate_lines = false`).
- Git commit mode: `C-c C-c` saves and exits, `C-c C-k` aborts with exit code 1.
- UTF-8 input; CRLF line endings are preserved.

## Performance

Median time from process start to first terminal output, 44-line Rust file,
9 runs, Intel Core Ultra 7 155U / Arch Linux. Values vary by about 20%.

| Editor | First paint |
|---|---:|
| nemacs | 2.7 ms |
| nano | 3.9 ms |
| vim | 5.3 ms |
| emacs -nw -Q | 80.4 ms |

Cursor motion writes about 150 bytes per keystroke; a 100-key burst is
coalesced into one frame of about 200 bytes. `make bench` reproduces these
numbers.

## Install

Rust 1.74 or newer for source installs.

```sh
cargo install --path .        # ~/.cargo/bin/nemacs
make install                  # ~/.local/bin/nemacs
make install PREFIX=/usr/local
```

Prebuilt binaries are attached to GitHub Releases. `install.sh` detects the
platform, verifies the SHA-256 checksum, and installs to
`${PREFIX:-$HOME/.local}/bin`:

```sh
curl -fsSL https://raw.githubusercontent.com/3tty0n/nano-emacs/main/install.sh | sh
```

From a checkout, or to pin a release:

```sh
./install.sh                  # latest release
./install.sh --release v0.1.0
./install.sh --from-source    # build and install from this checkout
./install.sh --uninstall      # remove the binary; config is kept
```

The installer accepts Linux assets for static musl or glibc, and macOS
universal or per-architecture assets. The URLs above assume the repository is
`3tty0n/nano-emacs`; set `NEMACS_REPO=owner/repo` when the slug differs.
Updates are installs: re-run the script, or run `cargo install --force`.
`~/.local/bin` must be on `PATH`.

`--uninstall` never removes `~/.config/nano-emacs`. For source installs,
`cargo uninstall nano-emacs` and `make uninstall` remove the binary.

## Usage

```sh
nemacs                  # scratch buffer
nemacs FILE             # open a file, or create it
nemacs +42 FILE         # start at line 42
nemacs --help           # print key bindings
```

## Git

```sh
git config --global core.editor nemacs
git config --global core.editor "$HOME/.local/bin/nemacs"     # not on PATH
GIT_EDITOR=nemacs git commit                                   # one-off
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
| `C-d` `DEL` | Delete char; delete the active region |
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

After `C-SPC`, moving point selects a region. `C-w` kills it, `M-w` copies it
to the kill ring, and `DEL` / `C-d` delete it without copying. `C-x C-x`
swaps point and mark, `C-x h` marks the whole buffer, and `C-g` deactivates
the mark.

During search, `C-s` / `C-r` repeat, `RET` accepts and sets mark, `C-g`
restores the original point. Matching is case-insensitive unless the query
contains an uppercase character.

## Commands

`M-x` runs `save-buffer`, `write-file`, `find-file`, `kill-emacs`, `undo`,
`redo`, `goto-line`, `toggle-line-numbers`, `toggle-trailing-whitespace`,
`auto-fill-mode`, `toggle-truncate-lines`, `delete-trailing-whitespace`,
`revert-buffer`, `search-forward`, `search-backward`, `beginning-of-buffer`,
`end-of-buffer`, and `help`. Region commands are also available:
`set-mark-command`, `kill-region`, `kill-ring-save`, `delete-region`,
`exchange-point-and-mark`, `mark-whole-buffer`, and `deactivate-mark`.

In the minibuffer, `TAB` completes the shared prefix of a command name or
path. Press `TAB` again to cycle through candidates and `S-TAB` to cycle
backwards. Path completion resolves relative paths against the current
buffer's directory and understands `~`, `.`, and `..`.

## Configuration

Read from the first existing path:

1. `$NANO_EMACS_CONFIG`
2. `$XDG_CONFIG_HOME/nano-emacs/config`
3. `~/.config/nano-emacs/config`

Format: `key = value`, one per line; `#` starts a comment; unknown keys are
ignored.

```ini
tab_width = 4
line_numbers = true
highlight_trailing_whitespace = true
auto_indent = true
auto_fill = false
fill_column = 70
truncate_lines = true        # false = soft-wrap long lines
theme = dark                 # dark or light
comment_char = "#"           # match core.commentChar
confirm_on_quit = true
```

## Filling and wrapping

`auto_fill = true` breaks typed text at the last whitespace that fits within
`fill_column`. Columns are display columns, so wide CJK characters count as
two. Toggle the mode with `M-x auto-fill-mode`.

`truncate_lines = true` (the default) keeps one buffer line on one screen row
and scrolls horizontally. Set it to `false`, or run
`M-x toggle-truncate-lines`, to soft-wrap long lines instead.

## Languages

- Systems: Rust, Go, C, C++, Zig, Swift, Java, Kotlin, Dart.
- Scripting and queries: Python, JavaScript, TypeScript, Ruby, PHP, Lua, Shell,
  Elixir, Haskell, SQL.
- Markup and prose: Markdown, LaTeX, reStructuredText, AsciiDoc, HTML, CSS, Vue,
  Svelte.
- Data and configuration: JSON, TOML, YAML, INI/.env, HCL/Terraform, Nix, CMake,
  Dockerfile, Makefile, Protobuf.
- Diff and Git commit messages.

Detection uses the file name (for example `Dockerfile`, `CMakeLists.txt`,
`Gemfile`, `.bashrc`, `.env`), then the extension, then the shebang.
`COMMIT_EDITMSG`, `MERGE_MSG`, `TAG_EDITMSG`, and `SQUASH_MSG` open in GitCommit
mode.

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
make e2e          # PTY end-to-end tests, including a real git commit
make bench        # startup comparison
make check        # rustfmt check and tests
```

## Releasing

Releases are built and published manually.

```sh
./scripts/release.sh                    # host target, version from Cargo.toml
./scripts/release.sh --target x86_64-unknown-linux-musl
./scripts/release.sh --version 0.2.0 --output dist
```

The script writes `dist/nemacs-v<version>-<target>.tar.gz` and a `.sha256`
file. It does not tag or upload. Publish the files with the GitHub CLI or the
web UI:

```sh
git tag v0.1.0
git push origin v0.1.0
gh release create v0.1.0 \
  dist/nemacs-v0.1.0-*.tar.gz dist/nemacs-v0.1.0-*.tar.gz.sha256 \
  --generate-notes
```

`make release ARGS="--target x86_64-unknown-linux-musl"` is equivalent to the
script. Build additional targets on their native platform, or install `cross`
to cross-compile. The script does not produce macOS universal binaries; build
`aarch64-apple-darwin` or `x86_64-apple-darwin` per machine.

## License

MIT
