# Contributing

Contributions are welcome: bug reports, documentation fixes, language modes,
and small features. emacette is deliberately small, so the main thing to
protect is its focus: fast startup, predictable Emacs-style editing, and a
small dependency footprint.

## Before you start

- For a bug, include the version (`emacette --version`), terminal, tmux or SSH
  if relevant, the smallest config that reproduces it, and the keys you
  pressed.
- For a larger feature, open an issue first so the design and scope can be
  agreed before you spend time on it.
- Small fixes and documentation improvements can go straight to a pull
  request.

## Setup

```sh
git clone https://github.com/3tty0n/emacette.git
cd emacette
cargo run -- FILE
cargo test
```

Requirements and the full build/test matrix are in the
[development guide](README.md).

## Workflow

1. Fork the repository and create a topic branch.
2. Make the smallest change that solves the problem.
3. Add or update tests at the appropriate layer (see [Testing](testing.md)).
4. Update documentation if behavior or configuration changed.
5. Run the quality gates:

   ```sh
   cargo fmt
   cargo clippy --all-targets -- -D warnings
   cargo test
   make e2e        # for terminal, rendering, or CLI changes
   ```

6. Open a pull request that explains the problem, the approach, and anything
   you could not test.

## Code conventions

- Run `cargo fmt`; CI rejects unformatted code.
- Keep `cargo clippy --all-targets -- -D warnings` clean.
- Keep `src/editor.rs` and `src/buffer.rs` free of terminal dependencies. The
  headless tests depend on this boundary.
- `Point` columns are characters. Never store display widths in a `Point`;
  compute them in `view.rs` or `render.rs`.
- Buffer edits must go through the helpers that record undo information and
  update `version` / `dirty_from`.
- Do not block the UI thread. Language-server work belongs on its reader
  thread; expensive optional work should be lazy or cached.
- Prefer small, focused modules and explicit error handling. Avoid panics on
  user input or file-system errors.
- Comment non-obvious decisions, especially terminal quirks, tmux behavior, and
  LSP range conversions.
- Do not add a dependency without discussing it first. The project intentionally
  has only `crossterm`, `serde_json`, and `unicode-width`.
- Keep the README user-facing. Put user details in `docs/user/` and internals
  or contributor information in `docs/development/`.

## Tests

- Buffer, config, search, syntax, and view logic: unit tests next to the code.
- Key sequences, prompts, and editor behavior: `tests/editor_integration.rs`.
- Real terminal output, CLI, git, LSP, and spelling flows: `scripts/e2e.py`.
- Rendering: `src/render.rs` tests that assert on ANSI output.

A bug fix should include a regression test that fails before the change. See
[Testing](testing.md) for how to run individual suites and how the PTY harness
works.

## Commit messages

Use an imperative subject line, optionally followed by a short body explaining
why the change is needed:

```text
Fix region selection after incremental search

The search origin was stored as an active mark, so accepting a search
highlighted and could delete the traversed range. Store it as an inactive
mark instead.
```

Keep formatting-only changes in their own commit when practical.

## Pull request checklist

- [ ] The change is scoped to one problem.
- [ ] Tests cover the new or fixed behavior.
- [ ] `cargo fmt`, clippy, and `cargo test` pass.
- [ ] `make e2e` passes for terminal-facing changes.
- [ ] User-facing behavior is documented in `docs/user/`.
- [ ] The README feature list still describes users' experience, not internals.

## License

By contributing, you agree that your contribution is licensed under the
[MIT License](../../LICENSE).
