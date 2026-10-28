# Tabs

Part of the [emacette user guide](README.md).

Tabs are window layouts, in the same spirit as Emacs `tab-bar-mode`. Each tab
keeps its own split layout and the buffers shown in its windows. A tab bar
appears at the top of the screen when two or more tabs are open.

Opening a file with `C-x C-f` does **not** create a tab. It shows the file in
the current window; the file becomes a buffer, and the tab bar is unchanged.

## Keys

- `C-x t n` or `C-x t o` — show the next tab.
- `C-x t p` — show the previous tab.
- `C-x t 2` — create a new tab with an empty buffer.
- `C-x t 0` — close the current tab; `C-x t k` is an alias.
- `C-x t f` — find a file in the current window.
- `M-<right>` / `M-<left>` — next / previous tab.

`C-x t` followed by `C-g` cancels the prefix.

## Creating tabs

- `C-x t 2` creates a tab with one window showing an empty buffer, like a fresh
  Emacs tab.
- The tab bar label follows the buffer in the tab's active window.
- Opening files with `C-x C-f` replaces the current window's buffer. It never
  adds a tab.

## Buffers

- Opening a file that is already open reuses its buffer and shows it in the
  current window.
- The previous buffer is kept when you open another file. It is not shown in a
  tab, but opening the same file again returns to it with its edits and undo
  history intact.
- A buffer can be shown in several windows at once, including windows in
  different tabs. Each window keeps its own cursor and scroll position.

## Switching tabs

- `C-x t n` / `C-x t p` restore the tab's window layout and the buffers shown
  in its windows.
- The active window in the restored tab has the cursor.

## Closing tabs

- `C-x t 0` (or `C-x t k`) removes the tab and its windows.
- Buffers are not discarded when a tab closes. Named buffers remain available
  by reopening the file; unnamed buffers stay in memory until the editor
  exits.
- Closing the last tab quits the editor.
- If hidden buffers have unsaved changes, quitting first asks for confirmation.

## State

The kill ring is shared by all tabs and buffers. Each tab keeps its own window
layout. Each window keeps its cursor and scroll position. Each buffer keeps its
text, mark, undo/redo history, line ending, language mode, spelling state, LSP
server and diagnostics, and modified marker.

## Commands

- `tab-next`
- `tab-previous`
- `tab-new`
- `tab-close`

## Limitations

- There is no buffer list. A hidden unnamed buffer (for example one replaced
  by `C-x C-f`) cannot be selected again, though quitting still warns when it
  has unsaved changes.
- New tabs all show an empty `*scratch*` buffer; there is no buffer naming or
  renaming command.

See [Windows](windows.md) for split layouts.
