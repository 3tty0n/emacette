# Language servers (LSP)

Part of [emacette](../README.md).

`emacette` can talk to a language server over stdio. Configuration is per
language; nothing is started unless `lsp_command` is set.

## Example

```ini
[rust]
lsp_command = rust-analyzer

[python]
lsp_command = pyright-langserver --stdio
lsp_language_id = python

[typescript]
lsp_command = typescript-language-server --stdio

[c]
lsp_command = clangd

[cpp]
lsp_command = clangd

[go]
lsp_command = gopls

[markdown]
lsp_command = marksman server
```

- `lsp_command` is split on whitespace; single and double quotes group words.
- `lsp_language_id` overrides the default language identifier.
- A global `lsp_command` applies to every language; use a section to override
  or `lsp_command = off` to disable it for one language.
- The server is started in the directory containing the file.
- The mode line shows `LSP` while a server is attached.
- Server stderr is discarded; failures appear in the echo area as
  `LSP: ...`.

## Diagnostics

Servers publish diagnostics with `textDocument/publishDiagnostics`.

- Errors, warnings, information, and hints are underlined in the text when
  `diagnostics_inline = true` (the default).
- Diagnostics for the current line appear in a popup above the mode line when
  `diagnostics_popup = true` (the default).
- Popup markers are `E`, `W`, `I`, and `H`.
- `M-g n` (`M-x next-error`) moves to the next diagnostic.
- `M-g p` (`M-x previous-error`) moves to the previous diagnostic.
- The echo area shows the severity and message after jumping.
- Ranges are converted from LSP's UTF-16 positions to editor character
  columns.

Per-language display settings:

```ini
[rust]
diagnostics_inline = true
diagnostics_popup = false
```

## Completion

- With a server configured, `M-x complete-symbol` (`completion-at-point`)
  asks the server for completions at point.
- If `editor_completion = true`, requests are also sent while typing.
- Buffer-word candidates appear immediately; LSP results replace the menu when
  they arrive.
- A manual request works with an empty prefix, so member completion after `.`
  can use the server.
- `TAB` / `RET` accept, `C-n` / `C-p` or `Down` / `Up` select, and `Esc` /
  `C-g` dismiss the menu.

## Configuration summary

- `lsp_command` — server command line.
- `lsp_language_id` — LSP language identifier.
- `diagnostics_inline` — underline diagnostics.
- `diagnostics_popup` — show the current line's diagnostics in a popup.

See [Configuration](configuration.md) for per-language sections and
[Completion](completion.md) for the candidate menu.

## Limitations

- Only `initialize`, full-text `didOpen` / `didChange`, completion, and
  `publishDiagnostics` are implemented.
- No hover, go-to-definition, rename, formatting, code actions, or workspace
  edits.
- Completion snippets are not expanded; snippet items use their plain label.
- Completion documentation, `additionalTextEdits`, and resolve requests are
  ignored.
- The project root is the file's directory. There is no separate root setting.
- One server is managed per buffer. Reopening the file starts a new server.
- Servers that require a TTY or a different transport are not supported.
- Diagnostics can briefly lag behind edits because synchronization is
  asynchronous.
