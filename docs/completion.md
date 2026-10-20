# Completion

Part of [nano-emacs](../README.md).

## Minibuffer completion

`TAB` completes in `M-x`, `find-file`, and `write-file`. A completion menu
lists the candidates above the mode line.

- The first `TAB` extends the input to the longest common prefix.
- If several candidates remain, the echo area shows their count and the menu
  shows the list.
- The next `TAB` selects the first candidate.
- Further `TAB` presses cycle forward; `S-TAB` cycles backward.
- Editing, deleting, or moving the cursor resets the cycle.
- `RET` accepts the current input.
- A unique candidate is inserted immediately.

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
- The prefix is matched case-sensitively.
- Up to 100 candidates are kept.

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
  `M-x toggle-editor-completion`.
