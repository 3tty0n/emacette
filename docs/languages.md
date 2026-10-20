# Languages

Part of [emacette](../README.md).

Syntax highlighting covers 40 languages and formats.

## Systems

- Rust
- Go
- C
- C++
- Zig
- Swift
- Java
- Kotlin
- Dart

## Scripting and queries

- Python
- JavaScript
- TypeScript
- Ruby
- PHP
- Lua
- Shell
- Elixir
- Haskell
- SQL

## Lisp family

- Common Lisp
- Emacs Lisp
- Scheme
- Clojure
- Racket

All use the `Lisp` mode. Parentheses are coloured by nesting depth in this
mode. When using an LSP server for Clojure, Scheme, or Racket, set
`lsp_language_id` to the appropriate identifier in the `[lisp]` section.

## Markup and prose

- Markdown
- LaTeX
- reStructuredText
- AsciiDoc
- HTML
- CSS
- Vue
- Svelte

## Data and configuration

- JSON
- TOML
- YAML
- INI and `.env`
- HCL / Terraform
- Nix
- CMake
- Dockerfile
- Makefile
- Protobuf

## Other formats

- Diff
- Git commit messages

## Detection

- Match known file names first, for example `Dockerfile`, `CMakeLists.txt`,
  `Gemfile`, `.bashrc`, and `.env`.
- Otherwise use the file extension.
- Otherwise inspect the shebang for interpreters such as Python, Node, Ruby,
  PHP, Lua, Elixir, Haskell, Dart, Nix, and common shells.
- `COMMIT_EDITMSG`, `MERGE_MSG`, `TAG_EDITMSG`, and `SQUASH_MSG` open in
  Git commit mode.
- `.lisp`, `.lsp`, `.cl`, `.asd`, `.el`, `.scm`, `.ss`, `.rkt`, `.clj`,
  `.cljs`, `.cljc`, and `.edn` open in Lisp mode.
- Lisp shebangs for `sbcl`, `clisp`, `scheme`, `racket`, `guile`, and
  `clojure` are detected.

## Notes

- Vue and Svelte use HTML-style tag and attribute highlighting.
- Embedded JavaScript and CSS inside `<script>` and `<style>` are not parsed as
  separate languages.
- Markdown fences are highlighted as strings; fenced code is not recursively
  parsed.

## Per-language settings

Each language can have its own configuration section, for example `[rust]`,
`[python]`, or `[git-commit]`. Section names ignore case, spaces, hyphens, and
underscores.

Overridable keys include indentation, comment prefixes, completion, spelling,
diagnostics, and the LSP command. Programming-language modes enable editor
completion automatically; set `editor_completion = false` in a section to
disable it there. See
[Configuration](configuration.md#per-language-options) and [LSP](lsp.md).
