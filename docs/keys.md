# Key bindings

Part of [nano-emacs](../README.md).

- `C-` means Ctrl.
- `M-` means Alt, or ESC followed by the key.
- `F1` opens the in-editor help.
- `q`, `C-g`, or `F1` closes the help overlay.
- In the help overlay, `C-n` / `C-p` / `SPC` scroll.

## Files and git

- `C-x C-s` — save the buffer.
- `C-x C-c` — quit, asking before discarding changes.
- `C-x C-w` — write the buffer to a file (save as).
- `C-x C-f` — find a file.
- `C-c C-c` — save and quit, finishing a git commit.
- `C-c C-k` — abort a git commit without saving.

## Moving

- `C-f` / `C-b` — forward / backward one character.
- `C-n` / `C-p` — next / previous line.
- `M-f` / `M-b` — forward / backward one word.
- `C-a` / `C-e` — beginning / end of line.
- `M-<` / `M->` — beginning / end of buffer.
- `C-v` / `M-v` — page down / page up.
- `M-g g` — go to line.
- `C-l` — recentre the screen and repaint.

## Editing and regions

- `C-d` / `DEL` — delete a character; delete the active region when one is selected.
- `M-d` / `M-DEL` — kill the next / previous word.
- `C-k` — kill to the end of the line.
- `C-y` / `M-y` — yank / yank-pop.
- `C-SPC` or `C-@` — set the mark and start a region.
- `C-w` — kill the region.
- `M-w` — copy the region to the kill ring.
- `C-x C-x` — exchange point and mark.
- `C-x h` — mark the whole buffer.
- `C-g` — cancel, or deactivate the mark.
- `C-t` — transpose characters.
- `M-u` / `M-l` / `M-c` — upcase / downcase / capitalise the word or region.
- `C-o` — open a line below point.
- `C-/` or `C-_` — undo; `M-x redo` redoes.
- `TAB` / `S-TAB` — indent / unindent.
- `M-$` — prompt to replace the misspelled word at point.

### Region behaviour

- After `C-SPC`, moving point selects a region; the selection is highlighted.
- `C-w` deletes the region and saves it to the kill ring.
- `M-w` copies the region to the kill ring and keeps the region active.
- `DEL`, `Backspace`, `C-d`, and `C-h` delete the active region without saving it.
- `C-x C-x` swaps point and mark.
- `C-x h` marks the whole buffer.
- `C-g` deactivates the mark.
- The kill ring is internal; it is not the system clipboard.

## Searching

- `C-s` / `C-r` — incremental search forward / backward.
- `C-s` / `C-r` again — next / previous match.
- `RET` — accept the match and set mark at the starting point.
- `C-g` — abort the search and restore the original point.
- Matching is case-insensitive unless the query contains an uppercase character.

## Minibuffer

- `M-x` — run a command by name.
- `TAB` / `S-TAB` — complete or cycle candidates.
- `RET` — accept.
- `C-g` — cancel.
- `C-p` / `C-n` or `Up` / `Down` — history.
- `C-a` / `C-e`, `C-b` / `C-f`, `Home` / `End` — move within the input.
- `C-k` — kill to the end of the input.
- `C-w` — delete the previous word.

## Editor completion

- `TAB` or `RET` — accept the selected candidate.
- `Down` / `C-n` — select the next candidate.
- `Up` / `C-p` — select the previous candidate.
- `Esc` or `C-g` — dismiss the completion menu.
- Word characters and `Backspace` — filter the candidates.
- Other keys dismiss the menu and run normally.

## Spelling

- `M-x flyspell-mode` — toggle misspelling highlights.
- `M-x ispell-next` / `ispell-previous` — jump between misspelled words.
- `M-x ispell-buffer` — check the whole buffer.

See [Completion](completion.md) and [Spelling](spelling.md) for details.
