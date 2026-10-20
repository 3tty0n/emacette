# Completion

Part of [nano-emacs](../README.md).

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

### Candidate menu

- The menu reserves up to six rows above the mode line.
- The selected candidate is highlighted with `>`.
- The list scrolls to keep the selected candidate visible.

## Editor completion

Editor completion is optional and off by default.

- Set `editor_completion = true` in the configuration, or run
  `M-x toggle-editor-completion`.
- After two word characters, a candidate menu appears automatically.
- `M-x complete-symbol` opens the menu manually at any prefix length.
- Candidates come from words and `\commands` in the current buffer.
- If an LSP server is configured for the language, `M-x complete-symbol`
  requests server completions as well. See [LSP](lsp.md).
- LSP requests are also sent while typing when `editor_completion = true`.
- Buffer-word candidates appear immediately; LSP results replace the menu when
  they arrive.
- A manual LSP request works with an empty prefix, so member completion after
  `.` can use the server.
- The local prefix is matched case-sensitively.
- Up to 100 local candidates and 200 LSP candidates are kept.

Keys while the editor menu is open:

- `TAB` or `RET` — accept the selected candidate.
- `Down` / `C-n` — select the next candidate.
- `Up` / `C-p` — select the previous candidate.
- `Esc` or `C-g` — dismiss the menu.
- Word characters — continue typing and filter the candidates.
- `Backspace` — shorten the prefix and filter again.
- Any other key — dismiss the menu and handle the key normally.

The mode line shows `Comp` while editor completion is enabled.

## Configuration

- `editor_completion` — enable automatic editor completion; default `false`.
  Set it in the configuration file or toggle it with
  `M-x toggle-editor-completion`. The value can be overridden per language.
- `lsp_command` — optional language-server command for LSP completion; see
  [LSP](lsp.md).

See [Configuration](configuration.md) for language sections such as
`[rust]` and `[markdown]`.
