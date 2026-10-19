# Installation

Part of [nano-emacs](../README.md).

## Requirements

- Rust 1.74 or newer for source installs.
- Linux (static musl or glibc) or macOS (universal or per-architecture) for
  prebuilt binaries.

## From source

```sh
cargo install --path .        # ~/.cargo/bin/nemacs
make install                  # ~/.local/bin/nemacs
make install PREFIX=/usr/local
```

- `cargo install --force` updates a source install.
- `cargo uninstall nano-emacs` removes a cargo-installed binary.
- `make uninstall` removes a binary installed with `make install`.
- `~/.local/bin` must be on `PATH` when installing with `make` or `install.sh`.

## Prebuilt binaries

Prebuilt binaries are attached to GitHub Releases. `install.sh` detects the
platform, verifies the SHA-256 checksum, and installs to
`${PREFIX:-$HOME/.local}/bin`.

```sh
curl -fsSL https://raw.githubusercontent.com/3tty0n/nano-emacs/main/install.sh | sh
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
- Set `NEMACS_REPO=owner/repo` when the repository slug differs.
- Re-run the installer, or run `cargo install --force`, to update.
- `--uninstall` never removes `~/.config/nano-emacs`.

The URLs above assume the repository is `3tty0n/nano-emacs`.
