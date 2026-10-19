# Commands and minibuffer completion

Part of [nano-emacs](../README.md).

`M-x` runs a command by name. `TAB` completes names and paths.

## Files and buffers

- `find-file` — open a file.
- `save-buffer` — save the current buffer.
- `write-file` — write the buffer to a different file.
- `revert-buffer` — reload the file from disk.
- `kill-emacs` — quit, asking about unsaved changes.
- `save-buffers-kill-terminal` — alias for `kill-emacs`.

## Editing

- `undo` — undo the last change.
- `redo` — redo the last undone change.
- `goto-line` — move to a line number.
- `delete-trailing-whitespace` — remove trailing spaces and tabs.
- `search-forward` — incremental search forward.
- `search-backward` — incremental search backward.
- `beginning-of-buffer` — move to the start.
- `end-of-buffer` — move to the end.

## Display and modes

- `toggle-line-numbers` — show or hide the line-number gutter.
- `toggle-trailing-whitespace` — highlight or stop highlighting trailing whitespace.
- `auto-fill-mode` — toggle automatic hard wrapping at `fill_column`.
- `toggle-auto-fill` — alias for `auto-fill-mode`.
- `toggle-truncate-lines` — switch between horizontal scrolling and soft wrapping.

## Regions

- `set-mark-command` — set the mark.
- `kill-region` — delete the region and save it to the kill ring.
- `kill-ring-save` — copy the region to the kill ring.
- `copy-region-as-kill` — alias for `kill-ring-save`.
- `delete-region` — delete the region without saving it.
- `exchange-point-and-mark` — swap point and mark.
- `mark-whole-buffer` — select the whole buffer.
- `deactivate-mark` — clear the mark.

## Help

- `help` — open the help overlay.
- `describe-bindings` — alias for `help`.

## Minibuffer completion

`TAB` completes in `M-x`, `find-file`, and `write-file`.

- The first `TAB` extends the input to the longest common prefix.
- If several candidates remain, the echo area shows their count and a preview.
- The next `TAB` selects the first candidate.
- Further `TAB` presses cycle forward through the candidates.
- `S-TAB` cycles backward.
- Editing, deleting, or moving the cursor resets the completion cycle.
- `RET` accepts the current input.
- A unique candidate is inserted immediately.

### Path completion

- Relative paths are resolved against the current buffer's directory.
- Absolute paths and `~/` paths work.
- `.`, `..`, and hidden files (when the input starts with `.`) are supported.
- Directory candidates end with `/`.
- Pressing `TAB` on a completed directory lists its contents.

### Command completion

- Command names from the list above are completed.
- `TAB` again cycles through matching commands.
- `S-TAB` cycles backwards.

### Candidate hints

- The echo area shows `N candidates: ...` while a completion is pending.
- Once cycling, it shows the position and preview, for example
  `[2/4] alpha.txt alphabet.txt ...`.
