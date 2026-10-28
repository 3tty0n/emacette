# Windows

Part of the [emacette user guide](README.md).

The screen can be split into several panes. All panes show the active
buffer: text edits and point are shared, but each pane keeps its own scroll
position. This is useful for keeping a function definition visible while
editing somewhere else, or for viewing two distant parts of a file.

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

- The active pane has the cursor; the mode line position and echo area refer
  to it.
- Splitting the active pane nests inside it, so layouts can mix horizontal and
  vertical splits.
- Each pane scrolls independently. Moving point in one pane does not scroll
  the others.
- Window layouts belong to a tab. Switching tabs restores that tab's layout
  and scroll positions. See [Tabs](tabs.md).
- Up to eight panes can be open. Splitting is refused when the pane is too
  small to show both halves usefully.
- `C-x o` cycles through panes in layout order.
- The completion popup stays inside the active pane.

## Limitations

- All panes show the same buffer; opening a different file uses a tab.
- Panes are split evenly. There is no command to resize a split or move a pane.
- The mode line and echo area are shared by the whole frame.
