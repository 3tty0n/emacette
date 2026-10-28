# Architecture

This document describes how emacette is put together. It is meant to help you
find the right module and avoid breaking assumptions that are not obvious from
individual functions.

## Design constraints

- **Terminal-independent editor core.** `src/editor.rs` and `src/buffer.rs` do
  not depend on `crossterm`; only `main.rs` and `render.rs` touch the terminal.
  This is what makes the headless integration tests possible.
- **Character columns, not bytes.** Buffer lines are `Vec<char>`. `Point.col`
  counts characters. Display columns (tabs, wide CJK, emoji) are computed at
  render time and must not be stored in `Point`.
- **Line-oriented text model.** The buffer is a `Vec<Line>` where
  `Line = Vec<char>`. Inserting or deleting a newline creates or removes a
  line; there is no piece table or rope.
- **One thread for the UI.** Language servers run on a reader thread and talk
  over `mpsc`; everything else (input, editing, syntax, spelling, rendering)
  runs synchronously in the main loop.
- **Bounded optional work.** Any expensive work that is not needed for the
  current frame should be lazy or cached: syntax highlighting per visible row,
  spell checks per visible row, completion only after a prefix exists.

## Process and event loop

`src/main.rs` parses the command line, checks that stdin/stdout are terminals,
enters the alternate screen, installs the tmux key guard, and then runs:

```text
              ┌─────────────────────────────────────────────┐
              │                  main loop                  │
              │                                             │
  LSP thread ─→ poll_lsp() ──changed?──→ redraw = true       │
              │                                             │
              │ redraw? ─→ Renderer::render() ─→ stdout     │
              │                                             │
              │ event::poll(50 ms) ─→ handle_event()        │
              │      │                   │                  │
              │      │ key               └→ Editor::handle_key()
              │      └ paste ─────────────→ Editor::handle_paste()
              │                                             │
              └─────────────────────────────────────────────┘
```

- `redraw` is only set for real input or LSP changes; idle iterations do no
  rendering.
- After the first event, buffered keys are drained (up to 512) before the frame
  is painted, so fast typing and key repeat produce one frame.
- `TerminalGuard` restores cursor visibility, bracketed paste, the alternate
  screen, and raw mode in `Drop`. `tmux::KeyGuard` does the same for temporary
  tmux bindings.

## Module map

| Module | Responsibility |
| --- | --- |
| `main.rs` | CLI, terminal lifecycle, event loop, crossterm key conversion |
| `editor.rs` | `Editor` state, key dispatch, tabs, prompts, `M-x` commands, LSP polling |
| `buffer.rs` | Lines, point/mark, editing primitives, undo/redo, kill ring, file I/O |
| `key.rs` | Terminal-independent `Key` / `KeyCode` and control-code normalization |
| `minibuffer.rs` | Prompt state, input editing, history, completion cursor |
| `completion.rs` | Editor completion candidates, selected entry, word helpers |
| `search.rs` | Forward/backward incremental-search primitives |
| `view.rs` | Viewport, scrolling, truncation vs wrapping, display-width helpers |
| `render.rs` | ANSI frame generation, incremental repaint, layout of all screen rows |
| `syntax.rs` | `Lang` detection, token spans, line-oriented lexers |
| `highlight.rs` | Lazy per-row highlight cache keyed by buffer version |
| `theme.rs` | `Style`, color capabilities, and the built-in themes |
| `spell.rs` | Dictionary discovery, word list parsing, spelling checks |
| `lsp.rs` | Minimal JSON-RPC/LSP client over stdio |
| `tmux.rs` | Temporary tmux key forwarding and prefix handling |
| `config.rs` | Config file parser and per-language `Settings` resolution |
| `help.rs` | Static help text shared by F1 and `--help` |

## Editor state and tabs

`Editor` owns the state of the active tab: `Buffer`, `View`, language mode,
effective `Settings`, `Highlighter`, minibuffer, search, completion, spell
state, LSP client, and diagnostics. Inactive tabs are stored as `TabState`.

Switching tabs is a stash/load operation:

1. `stash_active_tab()` moves the active `Buffer` and related state into the
   current `TabState`.
2. `active_tab` changes.
3. `load_active_tab()` moves the stored state back into `Editor` and rebuilds
   the highlighter.

The kill ring and command history live on `Editor`, not on a tab, so they are
shared. Unsaved buffers are never discarded by tab switching.

## Input dispatch

`Editor::handle_key` is the single entry point used by the terminal loop and by
tests. Precedence is deliberate:

1. Help overlay (`overlay`)
2. Confirmation prompt (`confirm`)
3. Minibuffer (`minibuffer`)
4. Incremental search (`search`)
5. Prefix keys (`C-x`, `C-c`, `M-g`, ...)
6. Normal editing keys

Every path wraps the actual work in `Buffer::begin_command()` /
`Buffer::end_command(coalesce)`. This groups all edits caused by one key press
into one undo step and optionally coalesces consecutive inserts or deletes.

## Buffer and undo

`Buffer` stores `lines: Vec<Line>`, `point`, `mark` (with a separate
`mark_active` flag), `version`, and `dirty_from`.

- **Edits** go through helpers that call `record(row, old_lines, new_count)`
  before changing `lines`. `record` captures the minimal changed line range.
- **Undo groups** are built by `begin_command` / `end_command`. A group stores
  the before/after points and the changed line ranges. Undo/redo swaps the
  stored text and restores the point.
- **Change tracking** bumps `version` and lowers `dirty_from` so the highlighter
  knows where to resume. `sync_highlighter` consumes `dirty_from` and resets it
  to `usize::MAX`.
- **Coalescing** merges consecutive single-character inserts or deletes into
  one undo group. Any non-editing command seals the previous group.
- **File I/O** preserves LF vs CRLF and normalizes a BOM on load. Non-UTF-8
  files are rejected.

## Rendering

`render.rs` builds one styled string per terminal row:

```text
tab bar (optional)
diagnostics popup (optional)
text body        ← gutter + syntax + selection + search + spelling
                   (editor completion popup is drawn over this area)
mode line
echo area / minibuffer
```

The renderer keeps the previous frame and compares rows by string equality. On
a normal frame only changed rows are written. Full repaints happen on startup,
resize, and `C-l`. Frames are wrapped in DEC 2026 synchronized-update sequences
when the terminal supports them.

Two layout modes are supported:

- `truncate_lines = true` — one logical line per screen row, horizontal scroll.
- `truncate_lines = false` — soft wrapping at the terminal width; `View.top`
  and `View.top_offset` identify the first visible wrapped segment.

`View::ensure_visible` owns scrolling. Screen elements below the body, such
as the minibuffer candidate list and the diagnostics popup, reserve rows in
`View::menu_rows`; the tab bar does the same with `View::tab_rows`. The editor
completion popup is different: it is anchored to the cursor and drawn over
the text body, so it does not change the body height.

## Syntax highlighting

`syntax.rs` contains hand-written, line-oriented lexers. `LexState` carries
multi-line context (block comments, raw strings, Markdown fences) from one line
to the next. Each line produces `Span { start, end, tok }`; the renderer maps
`Tok` to a `Style` through the theme.

`Highlighter` caches spans per row and lazily lexes only rows requested by the
renderer. When the buffer version changes, it invalidates from `dirty_from`
onwards. Opening a large file therefore only highlights visible lines.

Lisp-family modes additionally compute parenthesis depth in `Editor` and color
parentheses with the rainbow palette.

Adding a language generally means:

1. Add a `Lang` variant and its display name.
2. Extend `Lang::detect` (file name, extension, shebang, or special case).
3. Add a lexer branch or a dedicated `lex_*` function.
4. Update [Languages](../user/languages.md) and, if relevant, the programming
   mode classification used by automatic completion.
5. Add a highlighting test.

## Completion

There are two independent mechanisms:

- **Minibuffer completion** (`minibuffer.rs`, `completion.rs`) for `M-x`, file
  paths, and other prompts. Candidates are recomputed as the input changes.
- **Editor completion** for text at point. Local candidates come from buffer
  words and `\commands`; if an LSP server is attached, a completion request is
  sent and its results replace the local menu when they arrive.

Editor completion is enabled by default in programming-language modes. The
decision is made per language in `config.rs`; `M-x toggle-editor-completion`
overrides it for the current tab only.

## Language servers

`lsp.rs` is a deliberately small client:

- Spawns the configured command in the file's directory.
- Performs `initialize` / `initialized`.
- Opens the document with full text and sends full-text `didChange`.
- Handles `publishDiagnostics` and completion responses.
- Converts UTF-16 ranges from LSP into character columns.
- Runs a reader thread and sends events through `mpsc`; `Editor::poll_lsp`
  drains them without blocking.

One client is managed per tab; switching tabs keeps the server and its
diagnostics with the tab. There is no hover, rename,
formatting, code action, or workspace edit support.

## Spelling

`spell.rs` loads a plain word list or the base words of a hunspell `.dic` file.
Affixes are not expanded. The checker uses syntax spans to skip code, comments,
math, and markup commands, and keeps its state per tab. Checks are synchronous
and cached; only visible rows are checked for highlighting.

## tmux integration

tmux resolves its root and prefix key tables before passing input to a pane.
`tmux.rs` temporarily wraps root-table bindings so that keys go to the pane when
that pane runs emacette, and disables a conflicting prefix for the duration.
All changes are made through `tmux` commands and restored in `Drop`; no user
configuration file is modified. Failures are ignored so that the editor still
starts outside tmux or when tmux rejects a command. Set `EMACETTE_TMUX_KEYS=0`
to opt out.

## Configuration

`config.rs` parses a line-oriented `key = value` file. Values outside a section
are global defaults; `[language]` sections override them. Keys and section names
are normalized (case, spaces, hyphens, underscores), and unknown keys are
ignored.

`Config::settings_for(Lang)` resolves the effective `Settings` used by a buffer.
Changing language, opening a file, or switching tabs applies those settings to
the editor and view.

## Invariants worth preserving

- `Point` is in characters; display width is only computed in `view.rs` and
  `render.rs`.
- A buffer edit must go through `record` (direct `lines` mutation breaks undo).
- Bump `version` and update `dirty_from` whenever text changes, or the
  highlighter may show stale spans.
- Do not do blocking I/O on the UI thread; LSP is asynchronous by design.
- Keep `editor.rs` and `buffer.rs` free of terminal dependencies.
- Every screen row added above the body must be accounted for in layout and
  cursor positioning.
