# Development guide

This guide is for people who build, test, or change emacette. It assumes basic
familiarity with Rust and terminal user interfaces. For day-to-day usage, see
the [user guide](../user/README.md).

## Documentation map

- [Contributing](contributing.md) — workflow, code conventions, and review
  expectations.
- [Architecture](architecture.md) — process model, module map, and data flow.
- [Testing and benchmarks](testing.md) — unit, integration, PTY end-to-end
  tests, and performance measurements.
- [Releasing](releasing.md) — versioning, packaging, and publishing releases.

## Requirements

- Rust 1.74 or newer (`rust-version` in `Cargo.toml`).
- `rustfmt` and `clippy` components.
- Python 3 and git for the PTY end-to-end tests.
- A POSIX shell for `install.sh`, `scripts/release.sh`, and `make`.

## Quick start

```sh
git clone https://github.com/3tty0n/emacette.git
cd emacette

cargo run -- FILE          # run from the checkout
cargo test                 # unit and headless integration tests
cargo fmt -- --check       # formatting
cargo clippy --all-targets -- -D warnings
make e2e                   # PTY end-to-end tests (builds release first)
make bench                 # startup comparison
```

`make check` runs the formatting check and `cargo test`. Run clippy and
`make e2e` as well before opening a pull request; see
[Testing](testing.md) for the full matrix.

## Design goals

- **One small binary.** Runtime dependencies are `crossterm` for the terminal,
  `serde_json` for LSP messages, and `unicode-width` for display widths.
- **Instant start.** Measure changes with `make bench`; avoid work at startup
  that is not needed for the first frame.
- **Emacs habits, not Emacs compatibility.** Key bindings and command names are
  familiar, but the implementation is intentionally small and does not aim for
  full Emacs Lisp or package compatibility.
- **Text editing first.** Language features (syntax highlighting, completion,
  LSP, spelling) are additive and must never block typing or rendering.
- **Headless testability.** The editor core has no terminal dependency, so
  key sequences can be replayed in tests.

## Repository layout

```text
src/main.rs         CLI, terminal lifecycle, event/render loop
src/editor.rs       editor state, key dispatch, tabs, M-x commands
src/buffer.rs       text buffer, point/mark, undo/redo, kill ring
src/key.rs          terminal-independent key types and control normalization
src/minibuffer.rs   minibuffer prompt, history, and editing
src/completion.rs   editor-side completion candidates
src/search.rs       incremental search helpers
src/view.rs         viewport, scrolling, wrapping, and display widths
src/window.rs       split tree and per-pane viewports
src/render.rs       ANSI frame construction and incremental repaint
src/syntax.rs       lexers, token spans, and language detection
src/highlight.rs    per-line syntax highlighting cache
src/theme.rs        styles, colors, and the built-in themes
src/spell.rs        dictionary discovery and spell checking
src/lsp.rs          minimal LSP client over stdio
src/tmux.rs         tmux key-forwarding guard
src/config.rs       configuration parser and per-language settings
src/help.rs         static help text for F1 and --help

tests/              headless integration tests
scripts/            e2e, benchmark, release, and PTY helpers
examples/config     annotated example configuration
docs/user/          user guide
docs/development/   this guide
```

## Where to change what

- **A key binding or `M-x` command:** `src/editor.rs` (dispatch) and
  `src/help.rs` (F1 text). Add the command name to `COMMANDS` so completion
  finds it.
- **Buffer behavior, undo, or kill ring:** `src/buffer.rs`; keep it
  terminal-independent.
- **Rendering, layout, or colors:** `src/render.rs`, `src/view.rs`,
  `src/window.rs`, and `src/theme.rs`. Verify both truncate and soft-wrap
  modes, with and without splits.
- **A language or format:** `src/syntax.rs` for detection and lexing. Update
  [Languages](../user/languages.md) and, when the language should enable
  editor completion automatically, the mode classification.
- **A configuration option:** `src/config.rs`, `examples/config`, and
  [Configuration](../user/configuration.md).
- **LSP or completion integration:** `src/lsp.rs` and the completion code in
  `src/editor.rs`.
- **A new user-visible feature:** document it in the right guide under
  `docs/user/`; keep the README focused on the user-facing overview.

## Continuous integration

The GitHub Actions workflow runs, in order:

1. `cargo fmt -- --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. `cargo build --release`
5. `sh -n install.sh`, an installer smoke test, and the PTY end-to-end suite

Run the same gates locally with `cargo fmt -- --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test`, and `make e2e`.
