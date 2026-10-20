# Installation

Part of [emacette](../README.md).

## Requirements

- Rust 1.74 or newer for source installs.
- Linux (static musl or glibc) or macOS (universal or per-architecture) for
  prebuilt binaries.

## From source

```sh
cargo install --path .        # ~/.cargo/bin/emacette
make install                  # ~/.local/bin/emacette
make install PREFIX=/usr/local
```

- `cargo install --force` updates a source install.
- `cargo uninstall emacette` removes a cargo-installed binary.
- `make uninstall` removes a binary installed with `make install`.
- `~/.local/bin` must be on `PATH` when installing with `make` or `install.sh`.
- An annotated example configuration is in `examples/config`.

## Prebuilt binaries

Prebuilt binaries are attached to GitHub Releases. `install.sh` detects the
platform, verifies the SHA-256 checksum, and installs to
`${PREFIX:-$HOME/.local}/bin`.

```sh
curl -fsSL https://raw.githubusercontent.com/3tty0n/emacette/main/install.sh | sh
```

## Installer options

From a checkout, or to pin a release:

```sh
./install.sh                  # latest release
./install.sh --release v0.1.0
./install.sh --from-source    # build and install from this checkout
./install.sh --uninstall      # remove the binary; config is kept
```

- The installer supports Linux assets for static musl or glibc.
- The installer supports macOS universal or per-architecture assets.
- Set `EMACETTE_REPO=owner/repo` when the repository slug differs.
- `NEMACS_REPO` and `NEMACS_BASE_URL` are accepted as legacy aliases.
- Re-run the installer, or run `cargo install --force`, to update.
- `--uninstall` never removes `~/.config/emacette`.

The URLs above assume the repository is `3tty0n/emacette`.

## Optional dictionary for spell checking

Spell checking needs an English word list or hunspell dictionary. Any of the
paths listed in [Spelling](spelling.md) work.

- Install a system dictionary such as `hunspell-en_US` or a `words` package.
- Or set `spell_dictionary` in the configuration.
- Or set `EMACETTE_DICT` for one run (`NANO_EMACS_DICT` is a legacy alias).

## Optional language servers

LSP support is opt-in. Install a server separately and configure
`lsp_command` in the relevant language section.

- Rust: `rust-analyzer`
- Python: `pyright-langserver` or `pylsp`
- C/C++: `clangd`
- Go: `gopls`
- TypeScript/JavaScript: `typescript-language-server`
- Markdown: `marksman`

See [LSP](lsp.md) for commands, diagnostics, and completion.
