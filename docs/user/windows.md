# Windows

Part of the [emacette user guide](README.md).

The screen can be split into several panes. Each pane shows a buffer and keeps
its own cursor and scroll position. Panes showing the same buffer share the
text and edits, so you can keep a function definition visible while editing
elsewhere; panes can also show different files.

## Keys

- `C-x 2` — split the current window, one pane above the other.
- `C-x 3` — split the current window, one pane left of the other.
- `C-x o` — move to the other window.
- `C-x 0` — delete the current window.
- `C-x 1` — delete every other window.

The same commands are available through `M-x` as `split-window-below`,
`split-window-right`, `other-window`, `delete-window`, and
`delete-other-windows`.

## Behaviour

- The active pane has the cursor and its buffer is shown in the mode line; the
  other panes keep their own cursors for when you switch to them.
- Splitting the active pane nests inside it, so layouts can mix horizontal and
  vertical splits.
- Each pane scrolls and moves point independently. Editing a buffer updates
  every pane showing that buffer.
- `C-x C-f` opens a file in the active pane. It does not create a tab, and the
  previous buffer stays available by reopening the file.
- If a file is already open, `C-x C-f` shows its buffer in the active pane
  instead of opening a second copy.
- Window layouts belong to tabs. Switching tabs restores that tab's splits and
  buffers. See [Tabs](tabs.md).
- Up to eight panes can be open per tab. Splitting is refused when the pane is
  too small to show both halves usefully.
- `C-x o` cycles through panes in layout order.
- The completion popup stays inside the active pane.

## Limitations

- Panes are split evenly. There is no command to resize a split or move a pane.
- The mode line and echo area are shared by the whole frame.
