# Configuration

Part of [nano-emacs](../README.md).

## Location

The first existing file is used:

- `$NANO_EMACS_CONFIG`
- `$XDG_CONFIG_HOME/nano-emacs/config`
- `~/.config/nano-emacs/config`

## Format

- One `key = value` pair per line.
- Values outside a section are defaults for every language.
- A section such as `[rust]` or `[markdown]` overrides only that language.
- Section names ignore case, spaces, hyphens, and underscores, so
  `[git-commit]`, `[gitcommit]`, and `[GitCommit]` are equivalent.
- `#` starts a comment, including inline comments outside quotes.
- Quote values that start with `#`, for example `comment_start = "#"`.
- Unknown keys and sections are ignored.
- Keys accept underscores or hyphens.
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
diagnostics_inline = true
diagnostics_popup = true
theme = dark                 # dark or light
confirm_on_quit = true

[rust]
tab_width = 4
comment_start = //
lsp_command = rust-analyzer

[python]
comment_start = "#"
lsp_command = pyright-langserver --stdio
lsp_language_id = python

[markdown]
spell_check = true
lsp_command = marksman server
diagnostics_popup = false
```

## Global options

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
  completion candidates while typing; default `false`. See
  [Completion](completion.md).
- `spell_check` (`spell-check`, `spellcheck`) — highlight misspelled words in
  prose modes; default `false`. See [Spelling](spelling.md).
- `spell_dictionary` (`spell-dictionary`) — path to a word list or hunspell
  `.dic` file; default is automatic discovery.
- `comment_start` (`comment-start`) — global line-comment prefix. When unset,
  each language uses its built-in prefix. Used by `M-;` (`comment-dwim`).
- `comment_char` (`comment-char`) — single-character comment prefix used by
  git commit buffers when `comment_start` is unset; default `#`.
- `diagnostics_inline` (`diagnostics-inline`) — underline LSP diagnostics in
  the text; default `true`.
- `diagnostics_popup` (`diagnostics-popup`) — show LSP diagnostics for the
  current line in a popup above the mode line; default `true`.
- `lsp_command` (`lsp-command`) — command line used to start a language server.
  See [LSP](lsp.md).
- `lsp_language_id` (`lsp-language-id`) — LSP `languageId` override for the
  current language.
- `theme` — `dark` or `light`; default `dark`.
- `confirm_on_quit` (`confirm-on-quit`) — ask before quitting when the buffer
  text differs from the last loaded or saved text; default `true`. An empty
  new buffer, or a buffer returned to its saved text with undo, quits without
  a prompt.

## Per-language options

These keys are valid in any language section and override the global value:

- `tab_width`
- `line_numbers`
- `highlight_trailing_whitespace`
- `auto_indent`
- `auto_fill`
- `fill_column`
- `truncate_lines`
- `editor_completion`
- `spell_check`
- `spell_dictionary`
- `comment_start` (or `comment_char`; the value may be multiple characters)
- `diagnostics_inline`
- `diagnostics_popup`
- `lsp_command`
- `lsp_language_id`

Language keys are the names used by the syntax highlighter, for example:

- `plain`, `rust`, `go`, `c`, `cpp`, `java`, `kotlin`, `swift`, `zig`, `dart`
- `python`, `javascript`, `typescript`, `ruby`, `php`, `lua`, `haskell`
- `elixir`, `sql`, `shell`, `markdown`, `latex`, `rst`, `asciidoc`
- `json`, `toml`, `yaml`, `ini`, `hcl`, `nix`, `cmake`, `dockerfile`
- `makefile`, `protobuf`, `html`, `css`, `vue`, `svelte`, `diff`, `gitcommit`

See [Languages](languages.md) for detection rules.

## LSP and diagnostics

- A language server is started only when `lsp_command` is set for the buffer's
  language (or globally).
- Diagnostics come from `textDocument/publishDiagnostics`.
- Inline diagnostics use `diagnostics_inline`; popups use
  `diagnostics_popup`.
- `M-g n` / `M-g p` jump to the next / previous diagnostic.
- Set `lsp_command = off` in a section to disable an inherited command.

See [LSP](lsp.md) for server examples and limitations.

## Comments

- `M-;` runs `comment-dwim` on the current line or active region.
- The prefix comes from `comment_start`, or from the language default.
- Markdown, HTML, CSS, and similar modes have no default line-comment prefix;
  set one explicitly if you want `M-;` there.
- Repeating `M-;` removes the prefix from an already commented region.

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

The mode line shows `Fill`, `Wrap`, `Comp`, `Spell`, and `LSP` while the
corresponding features are active.
