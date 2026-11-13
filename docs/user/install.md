# Installation

Part of the [emacette user guide](README.md).

## Requirements

- Rust 1.74 or newer for source installs.
- Linux (static musl or glibc) or macOS (universal or per-architecture) for
  prebuilt binaries. The release workflow publishes an arm64, an Intel, and a
  universal macOS archive; the universal one is preferred by the installer.

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
- On Apple silicon, the installer detects the hardware even when the shell
  runs under Rosetta, so it installs the universal or arm64 archive instead of
  the Intel one.
- On macOS the installer removes the quarantine attribute, repairs a missing
  ad-hoc signature when possible, and runs the new binary once before
  replacing an existing installation. A binary that macOS would kill on
  launch therefore does not replace a working copy.
- Set `EMACETTE_REPO=owner/repo` when the repository slug differs.
- `NEMACS_REPO` and `NEMACS_BASE_URL` are accepted as legacy aliases.
- Re-run the installer, or run `cargo install --force`, to update.
- `--uninstall` never removes `~/.config/emacette`.

The URLs above assume the repository is `3tty0n/emacette`.

## macOS troubleshooting

If macOS terminates emacette immediately (`Killed: 9`, or git reports that
the editor failed to start), the binary was rejected before it could run.
Reinstall with the current script; it clears quarantine and verifies that the
installed file starts. To inspect an existing binary:

```sh
file "$HOME/.local/bin/emacette"
codesign --verify --strict "$HOME/.local/bin/emacette"
xattr -l "$HOME/.local/bin/emacette"
```

For a source build, also check that `strip` resolves to the Apple tool:

```sh
command -v strip     # should print /usr/bin/strip
```

emacette's release profile does not ask cargo to strip during ordinary
`cargo build --release`; `scripts/release.sh` strips Apple binaries explicitly
with `/usr/bin/strip` and signs them afterwards. A GNU/Homebrew/MacPorts/Nix
`strip` earlier in `PATH` can corrupt a Mach-O file and produce exactly this
crash.

## Migrating from nano-emacs

The project and binary were renamed to emacette. Most existing setups keep
working after updating the command:

- The configuration file is now `~/.config/emacette/config`; the old
  `~/.config/nano-emacs/config` is not read.
- `EMACETTE_CONFIG` and `EMACETTE_DICT` replace `NANO_EMACS_CONFIG` and
  `NANO_EMACS_DICT`. The old environment variables are still accepted as
  fallbacks.
- Update `git config --global core.editor` and any scripts that invoke `nemacs`
  to use `emacette`.
- `cargo uninstall nano-emacs` removes an old cargo-installed binary.

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
