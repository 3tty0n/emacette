# Completion

Part of the [emacette user guide](README.md).

## Minibuffer completion

`M-x`, `find-file`, and `write-file` open with their candidates already listed
above the mode line. The list updates while you type.

- Opening a prompt lists all matching candidates immediately.
- `TAB` selects the first candidate, then cycles forward; `S-TAB` cycles
  backward.
- `C-n` / `C-p` (and `Down` / `Up`) select the next / previous candidate
  while the menu is open.
- `M-n` / `M-p` move through minibuffer history.
- If no menu is open, `TAB` extends the input to the longest common prefix.
- Editing, deleting, or moving the cursor resets the cycle and refreshes the
  list.
- `RET` accepts the current input.
- A unique candidate is selected with `TAB`, but opening a prompt does not
  insert it automatically.

### Path completion

- Relative paths resolve against the current buffer's directory.
- Absolute paths and `~/` paths work.
- `.`, `..`, and hidden files (when the input starts with `.`) are supported.
- Directory candidates end with `/`.
- Pressing `TAB` on a completed directory lists its contents.

### Command completion

- Command names from [Commands](commands.md) are completed.
- `TAB` again cycles through matching commands.
- `S-TAB` cycles backwards.

### Minibuffer candidate menu

- The minibuffer list reserves up to six rows above the mode line.
- The selected candidate is highlighted with `>`.
- The list scrolls to keep the selected candidate visible.

## Editor completion

Editor completion is enabled automatically in programming-language files and
off in prose, markup, and data formats.

- Rust, Go, C/C++, Java, Kotlin, Swift, Zig, Dart, Python, JavaScript,
  TypeScript, Ruby, PHP, Lua, Haskell, Elixir, Nix, Lisp, SQL, and Shell
  enable it by default.
- Set `editor_completion = false` in the global configuration to disable the
  automatic behavior, or in a language section to disable it for that
  language only. Set it to `true` to enable completion everywhere.
- `M-x toggle-editor-completion` toggles the current tab at runtime.
- After two word characters, a candidate popup appears at point automatically.
- `M-x complete-symbol` opens the popup manually at any prefix length.
- Candidates come from words and `\commands` in the current buffer.
- If an LSP server is configured for the language, `M-x complete-symbol`
  requests server completions as well. See [LSP](lsp.md).
- LSP requests are also sent while typing when completion is active (automatic in code files, or enabled explicitly).
- Buffer-word candidates appear immediately; LSP results replace the popup when
  they arrive.
- A manual LSP request works with an empty prefix, so member completion after
  `.` can use the server.
- The local prefix is matched case-sensitively.
- Up to 100 local candidates and 200 LSP candidates are kept.

Keys while the editor popup is open:

- `TAB` or `RET` — accept the selected candidate.
- `Down` / `C-n` — select the next candidate.
- `Up` / `C-p` — select the previous candidate.
- `Esc` or `C-g` — dismiss the popup.
- Word characters — continue typing and filter the candidates.
- `Backspace` — shorten the prefix and filter again.
- Any other key — dismiss the popup and handle the key normally.

The popup is anchored at the cursor. It opens below point when there is room
and above it otherwise, without shifting the rest of the screen. The selected
candidate is highlighted, and the list scrolls to keep it visible. The mode
line shows `Comp` while editor completion is enabled.

## Configuration

- `editor_completion` — control automatic editor completion. When unset, it
  is enabled in programming-language files and off elsewhere. Set `true` to
  enable everywhere or `false` to disable it. The value can be overridden per
  language, and `M-x toggle-editor-completion` toggles the current tab.
- `lsp_command` — optional language-server command for LSP completion; see
  [LSP](lsp.md).

See [Configuration](configuration.md) for language sections such as
`[rust]` and `[markdown]`.
