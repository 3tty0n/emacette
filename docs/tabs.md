# Tabs

Part of [emacette](../README.md).

Multiple files can be open at once. A tab bar appears at the top of the screen
when two or more tabs are open.

## Keys

- `C-x t n` or `C-x t o` — next tab.
- `C-x t p` — previous tab.
- `C-x t 2` — new scratch tab.
- `C-x t k` — close the current tab.
- `C-x t f` — find a file (new tab unless the current tab is empty).
- `M-<right>` / `M-<left>` — next / previous tab.

`C-x t` followed by `C-g` cancels the prefix.

## Opening files

- `C-x C-f` (`find-file`) opens the file in a new tab.
- If the current tab is an empty scratch buffer, the file reuses that tab.
- If the file is already open, its existing tab is selected.
- Opening a file no longer refuses just because the current buffer is
  modified: the modified buffer stays in its tab.

## Closing tabs

- `C-x t k` closes the current tab.
- A tab with unsaved changes is not closed; save it first.
- Closing the last clean tab quits the editor.
- Quitting with unsaved changes in another tab first switches to that tab
  instead of discarding it.

## State

The kill ring is shared by all tabs: text killed or copied in one tab can be
yanked in another. Each tab keeps its own:

- buffer text, point, mark, undo/redo history, and line ending
- scroll position
- language mode and effective settings
- spelling state
- LSP server and diagnostics
- modified marker

The tab bar shows a `*` after the name when the tab is modified. The active
tab is highlighted.

## Commands

- `tab-next`
- `tab-previous`
- `tab-new`
- `tab-close`

## Limitations

- No tab reordering, pinning, or drag-and-drop.
- No window splits or multiple panes.
- Very long tab bars show the active tab area and elide older tabs.
