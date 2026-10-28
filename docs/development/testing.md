# Testing and benchmarks

emacette uses three test layers. Each one covers a different kind of failure,
and a change should usually add a test at the lowest layer that can catch it.

## Test layers

| Layer | Location | What it covers |
| --- | --- | --- |
| Unit | `#[cfg(test)]` modules in `src/*.rs` | Buffer edits, undo, search, config parsing, syntax spans, key normalization, tmux parsing |
| Headless integration | `tests/editor_integration.rs` | Full key sequences through `Editor::handle_key`, file I/O, prompts, tabs, git commit flow |
| PTY end-to-end | `scripts/e2e.py` | A real binary on a pseudo-terminal: rendering, CLI, LSP, completion, spelling, git, exit codes |

Rendering has its own unit tests in `src/render.rs`. They render to ANSI
strings and assert on the output, so they are fast and do not need a terminal.

## Running tests

```sh
cargo test                       # unit + integration tests
cargo test search                # filter by name
cargo test --test editor_integration
make e2e                         # builds release, then runs the PTY suite
python3 scripts/e2e.py target/release/emacette   # run against an existing build
```

`make check` runs `cargo fmt -- --check` and `cargo test`; CI additionally runs
`cargo clippy --all-targets -- -D warnings`, a release build, installer checks,
and the PTY suite.

## Unit tests

- Keep buffer tests independent of the editor and the terminal.
- Test edge cases explicitly: empty buffers, last line, multi-byte characters,
  CRLF, wide characters, and undo boundaries.
- For a syntax change, assert the resulting spans or tokens for a representative
  snippet, including a multi-line construct that exercises `LexState`.

## Headless integration tests

`tests/editor_integration.rs` constructs an `Editor` directly and feeds `Key`
values, exactly as the event loop would. Use this layer for:

- key sequences and dispatch precedence;
- prompts, completion menus, search, and region behavior;
- saving, writing, reverting, and quit decisions;
- git commit mode.

Helpers such as `press`, `type_text`, and `clear_minibuffer` keep sequences
readable. Avoid timing assumptions; the editor is synchronous here.

## PTY end-to-end tests

The e2e suite spawns the release binary on a pseudo-terminal using
`scripts/ptyharness.py`, sends raw bytes, and parses the captured ANSI screen
with `scripts/screen.py`. It covers behavior that headless tests cannot see:
the alternate screen, cursor placement, styled output, and process exit codes.

Guidelines for new e2e tests:

- Build the binary first (`make e2e` does this).
- Keep a test self-contained: create its own temporary file and, when
  configuration matters, set `EMACETTE_CONFIG` for the child process only.
- Prefer checking saved file contents and exit codes over fragile timing.
- Look for text and styles in the captured output only when the visual result
  is the behavior under test.
- Tests may invoke `git`; keep repositories in temporary directories.
- The fake LSP server is generated inline by the suite, so no server needs to
  be installed.

The harness waits for the first frame (or an explicit `expect` string), sends
keys with a small delay, and enforces a timeout. If a test is flaky, fix the
synchronization rather than adding a long sleep.

## Benchmarks

```sh
make bench                                  # release build + benchmark
python3 scripts/bench.py [--runs N] [BINARY]
```

`scripts/bench.py` measures the median time from process start to the first
byte of terminal output. It is a "can I start typing yet?" measurement, not a
full workflow benchmark.

Reference numbers from the project history (44-line Rust file, 9 runs, Intel
Core Ultra 7 155U, Arch Linux; values vary by roughly 20%):

| Editor | Median first paint |
| --- | --- |
| emacette | 2.7 ms |
| nano | 3.9 ms |
| vim | 5.3 ms |
| emacs -nw -Q | 80.4 ms |

These numbers are indicative, not CI gates. When changing startup code or the
renderer, run the benchmark a few times and compare medians rather than a
single run.

## Manual smoke test

Before a release, also check by hand:

- start with no arguments, type, undo, and quit;
- open a file with `+LINE`, edit, save, and reopen;
- search forward and backward, accept and abort;
- switch and close tabs, including a modified tab;
- split horizontally and vertically, switch panes, and close them;
- run `git commit` with emacette as `core.editor`;
- resize the terminal while a completion popup is open;
- run inside tmux if you use it.
