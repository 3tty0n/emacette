# Key bindings

Part of the [emacette user guide](README.md).

- `C-` means Ctrl.
- `M-` means Alt, or ESC followed by the key.
- `F1` opens the in-editor help.
- `q`, `C-g`, or `F1` closes the help overlay.
- In the help overlay, `C-n` / `C-p` / `SPC` scroll.
- Inside tmux, these bindings take priority over tmux root bindings and a
  conflicting prefix key. See [tmux](tmux.md).

## Files and git

- `C-x C-s` — save the buffer.
- `C-x C-c` — quit; prompt only when the buffer differs from the last loaded or saved text.
- `C-x C-w` — write the buffer to a file (save as).
- `C-x C-f` — find a file; opens in a new tab.
- `C-x t n` / `C-x t p` — next / previous tab.
- `C-x t 2` — new scratch tab.
- `C-x t k` — close the current tab.
- `M-<right>` / `M-<left>` — next / previous tab.
- `C-c C-c` — save and quit, finishing a git commit.
- `C-c C-k` — abort a git commit without saving (prompts only when changed).

## Windows

- `C-x 2` — split the current window, one pane above the other.
- `C-x 3` — split the current window, one pane left of the other.
- `C-x o` — move to the other window.
- `C-x 0` — delete the current window.
- `C-x 1` — delete every other window.

Split panes share the buffer and point but keep their own scroll position.
See [Windows](windows.md).

## Moving

- `C-f` / `C-b` — forward / backward one character.
- `C-n` / `C-p` — next / previous line.
- `M-f` / `M-b` — forward / backward one word.
- `C-a` / `C-e` — beginning / end of line.
- `M-<` / `M->` — beginning / end of buffer.
- `C-v` / `M-v` — page down / page up.
- `M-g g` — go to line.
- `M-g n` / `M-g p` — next / previous LSP diagnostic.
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
- `M-;` — comment or uncomment the current line or region.
- Opening `(`, `[`, `{`, `"`, `'`, and `` ` `` inserts the matching closing
  delimiter when auto-pairs is enabled. A single quote is not paired in Lisp
  quote syntax or prose modes. `Backspace` removes an empty pair.
- `M-$` — prompt to replace the misspelled word at point.

### Region behaviour

- After `C-SPC`, moving point selects a region; the selection is highlighted.
- `C-w` deletes the region and saves it to the kill ring.
- `M-w` copies the region to the kill ring and keeps the region active.
- `DEL`, `Backspace`, `C-d`, and `C-h` delete the active region without saving it.
- `C-x C-x` swaps point and mark.
- `C-x h` marks the whole buffer.
- `C-g` deactivates the mark.
- The kill ring is shared by all tabs; it is not the system clipboard.

## Searching

- `C-s` / `C-r` — incremental search forward / backward.
- `C-s` / `C-r` again — next / previous match.
- `RET` — accept the match and set the mark at the starting point without
  activating the region; `C-x C-x` returns to the starting point.
- `C-g` — abort the search and restore the original point.
- Matching is case-insensitive unless the query contains an uppercase character.

## Minibuffer

- `M-x` — run a command by name.
- `TAB` / `S-TAB` — select or cycle candidates.
- `RET` — accept.
- `C-g` — cancel.
- `C-p` / `C-n` or `Up` / `Down` — select completion candidates when the menu is open; otherwise move through history.
- `M-p` / `M-n` — move through history even while the menu is open.
- `C-a` / `C-e`, `C-b` / `C-f`, `Home` / `End` — move within the input.
- `C-k` — kill to the end of the input.
- `C-w` — delete the previous word.

## Editor completion

- `TAB` or `RET` — accept the selected candidate.
- `Down` / `C-n` — select the next candidate.
- `Up` / `C-p` — select the previous candidate.
- `Esc` or `C-g` — dismiss the completion popup.
- Word characters and `Backspace` — filter the candidates.
- Other keys dismiss the menu and run normally.

## Diagnostics

- Diagnostics come from a configured LSP server.
- `diagnostics_inline` underlines errors and warnings in the buffer.
- `diagnostics_popup` shows the current line's diagnostics above the mode line.
- `M-g n` / `M-g p` move between diagnostics.
- `M-x next-error` / `previous-error` are the command equivalents.

## Spelling

- `M-x flyspell-mode` — toggle misspelling highlights.
- `M-x ispell-next` / `ispell-previous` — jump between misspelled words.
- `M-x ispell-buffer` — check the whole buffer.

See [Completion](completion.md), [Spelling](spelling.md), [LSP](lsp.md), and
[Tabs](tabs.md) for details.
