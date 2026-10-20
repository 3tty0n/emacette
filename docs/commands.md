# Commands

Part of [emacette](../README.md).

`M-x` runs a command by name. See [Completion](completion.md) for editor and
minibuffer completion, and [Spelling](spelling.md) for spell checking.

## Files and buffers

- `find-file` — open a file in a new tab (or select its existing tab).
- `save-buffer` — save the current buffer.
- `write-file` — write the buffer to a different file.
- `revert-buffer` — reload the file from disk.
- `kill-emacs` — quit; ask only when the buffer text has changed.
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
- `toggle-editor-completion` — enable or disable editor completion.
- `complete-symbol` — show completion candidates at point.
- `completion-at-point` — alias for `complete-symbol`.
- `comment-dwim` — comment or uncomment the current line or region.
- `next-error` — jump to the next LSP diagnostic.
- `previous-error` — jump to the previous LSP diagnostic.
- `tab-next` — switch to the next tab.
- `tab-previous` — switch to the previous tab.
- `tab-new` — create a scratch tab.
- `tab-close` — close the current tab.
- `flyspell-mode` — enable or disable spell checking in prose modes.
- `spell-check-mode` — alias for `flyspell-mode`.
- `toggle-spell-check` — alias for `flyspell-mode`.
- `ispell-word` — prompt to replace the word at point.
- `spell-word` — alias for `ispell-word`.
- `ispell-next` — jump to the next misspelled word.
- `next-spelling-error` — alias for `ispell-next`.
- `ispell-previous` — jump to the previous misspelled word.
- `previous-spelling-error` — alias for `ispell-previous`.
- `ispell-buffer` — check the whole buffer and report the count.
- `spell-check-buffer` — alias for `ispell-buffer`.

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

## Related documentation

- [Completion](completion.md) — minibuffer lists, candidate cycling, and optional editor completion.
- [Spelling](spelling.md) — dictionary discovery, highlighting, and `M-$`.
- [LSP](lsp.md) — language servers, diagnostics, and completion.
- [Tabs](tabs.md) — multiple files, switching, and closing tabs.
- [Key bindings](keys.md) — every key, including region and search behaviour.
