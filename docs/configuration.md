# Configuration

Part of [nano-emacs](../README.md).

## Location

The first existing file is used:

- `$NANO_EMACS_CONFIG`
- `$XDG_CONFIG_HOME/nano-emacs/config`
- `~/.config/nano-emacs/config`

## Format

- One `key = value` pair per line.
- `#` starts a comment, including inline comments outside quotes.
- Unknown keys are ignored.
- Keys accept underscores or hyphens where noted.
- Runtime toggles through `M-x` are not written back to the file.

```ini
tab_width = 4
line_numbers = true
highlight_trailing_whitespace = true
auto_indent = true
auto_fill = false
fill_column = 70
truncate_lines = true        # false = soft-wrap long lines
editor_completion = false
spell_check = false
# spell_dictionary = ~/.config/nano-emacs/en.dic
theme = dark                 # dark or light
comment_char = "#"           # match core.commentChar
confirm_on_quit = true
```

## Options

- `tab_width` (`tab-width`) — display width of a tab stop; default `4`; range `1`–`16`.
- `line_numbers` (`line-numbers`) — show the line-number gutter; default `true`.
- `highlight_trailing_whitespace` (`highlight-trailing-whitespace`) — highlight
  trailing spaces and tabs; default `true`.
- `auto_indent` (`auto-indent`) — copy the current indentation on `RET`; default `true`.
- `auto_fill` (`auto-fill`, `auto_fill_mode`, `auto-fill-mode`) — enable
  automatic hard wrapping; default `false`.
- `fill_column` (`fill-column`) — auto-fill column; default `70`; range `1`–`1000`.
- `truncate_lines` (`truncate-lines`, `truncate`) — truncate long lines and
  scroll horizontally; set to `false` for soft wrapping; default `true`.
- `editor_completion` (`editor-completion`, `completion_in_editor`) — show
  word-completion candidates while typing; default `false`. See
  [Completion](completion.md).
- `spell_check` (`spell-check`, `spellcheck`) — highlight misspelled words in
  prose modes; default `false`. See [Spelling](spelling.md).
- `spell_dictionary` (`spell-dictionary`) — path to a word list or hunspell
  `.dic` file; default is automatic discovery.
- `theme` — `dark` or `light`; default `dark`.
- `comment_char` (`comment-char`) — comment character used by git commit
  templates; default `#`.
- `confirm_on_quit` (`confirm-on-quit`) — ask before quitting with unsaved
  changes; default `true`.

The mode line shows `Fill`, `Wrap`, `Comp`, and `Spell` while the corresponding
modes are active.

## Auto-fill

- Enable with `auto_fill = true` or `M-x auto-fill-mode`.
- Typed text breaks at the last whitespace that fits within `fill_column`.
- Columns are display columns, so wide CJK characters count as two.
- Leading indentation is copied to the new line.
- The mode line shows `Fill` while auto-fill is enabled.
- The break is part of the same undo step as the character that triggered it.

## Soft wrapping

- `truncate_lines = true` (default) keeps one buffer line on one screen row and
  scrolls horizontally.
- `truncate_lines = false` soft-wraps long lines at the terminal width.
- `M-x toggle-truncate-lines` switches the setting at runtime.
- The mode line shows `Wrap` while soft wrapping is enabled.
- Tabs and wide characters are measured in display cells.
- Cursor placement and viewport scrolling follow the wrapped layout.
- `C-n` / `C-p` still move by logical line.
