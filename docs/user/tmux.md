# tmux

Part of the [emacette user guide](README.md).

tmux handles its own key tables before forwarding input to the program in a
pane. While emacette is running its key bindings take priority:

- tmux root-table keyboard bindings (for example a `C-y` clipboard binding)
  are forwarded to the active pane when that pane is running emacette.
  Other panes keep the original tmux action.
- If the tmux prefix itself is a key emacette uses (for example the default
  `C-b`), the session prefix is disabled while emacette runs.
- The original tmux bindings and prefix settings are restored when emacette
  exits.

Nothing is written to `~/.tmux.conf` or any other tmux configuration file.

Set `EMACETTE_TMUX_KEYS=0` to disable this integration.

## Notes

- Forwarding uses `pane_current_command`, so a tmux root binding keeps its
  normal action in panes that are not running emacette.
- The integration is best effort. If tmux is unavailable or a command fails,
  emacette starts normally and tmux keeps its current behaviour.
- Mouse bindings are left untouched.
- Changing the tmux prefix to a key that emacette does not use (for example
  `C-q`) avoids disabling the prefix while the editor is open.
