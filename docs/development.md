# Development and releases

Part of [emacette](../README.md).

## Requirements

- Rust 1.74 or newer.
- `rustfmt` and `clippy` components for formatting and lint checks.
- Python 3 and git for the PTY end-to-end tests.

## Build and test

```sh
cargo build --release     # target/release/emacette
cargo test                # unit and integration tests
make e2e                  # PTY end-to-end tests, including a real git commit
make check                # rustfmt check and tests
make bench                # startup comparison
```

- `cargo fmt -- --check` checks formatting.
- `cargo clippy --all-targets -- -D warnings` is the lint gate.
- The CI workflow also checks the installer and runs the PTY tests.

## Performance

Measurements are median time from process start to first terminal output, using
a 44-line Rust file, 9 runs, on an Intel Core Ultra 7 155U running Arch Linux.
Values vary by about 20%.

- `emacette` — 2.7 ms
- `nano` — 3.9 ms
- `vim` — 5.3 ms
- `emacs -nw -Q` — 80.4 ms

Rendering details:

- Only changed rows are sent to the terminal.
- Cursor motion writes about 150 bytes per keystroke.
- A 100-key burst is coalesced into one frame of about 200 bytes.
- `make bench` reproduces the startup numbers.

## Releasing

Releases are built and published manually.

```sh
./scripts/release.sh                    # host target, version from Cargo.toml
./scripts/release.sh --target x86_64-unknown-linux-musl
./scripts/release.sh --version 0.2.0 --output dist
```

- The script writes `dist/emacette-v<version>-<target>.tar.gz` and a `.sha256`
  file. Archives contain the binary, README, LICENSE, and `examples/config`.
- The script does not tag or upload.
- `make release ARGS="--target x86_64-unknown-linux-musl"` is equivalent to the
  script.
- Build additional targets on their native platform, or install `cross` to
  cross-compile.
- The script does not produce macOS universal binaries; build
  `aarch64-apple-darwin` or `x86_64-apple-darwin` per machine.

Publish the files with the GitHub CLI or the web UI:

```sh
git tag v0.1.0
git push origin v0.1.0
gh release create v0.1.0 \
  dist/emacette-v0.1.0-*.tar.gz dist/emacette-v0.1.0-*.tar.gz.sha256 \
  --generate-notes
```
