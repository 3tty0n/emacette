# Releasing

Releases are prepared locally with `scripts/release.sh` and published manually
on GitHub. There is no tag-and-upload automation.

## Versioning

- The version lives in `Cargo.toml` and is reported by `emacette --version`.
- Tags are `vMAJOR.MINOR.PATCH` (for example `v0.1.0`).
- The project is pre-1.0: minor releases may contain user-visible changes, and
  patch releases should stay backward compatible within a minor series.
- Update `Cargo.lock` after changing the version (a normal `cargo build` does
  this) so `--locked` builds keep working.

## Pre-release checklist

1. `git status` is clean, and the branch is up to date.
2. `Cargo.toml` has the intended version.
3. `make check` passes; CI is green (format, clippy, tests, release build,
   installer checks, PTY suite).
4. User-visible changes are reflected in the user guide, README feature list,
   or `examples/config`.
5. The manual smoke test in [Testing](testing.md#manual-smoke-test) passes.
6. `make bench` still looks reasonable if startup or rendering changed.

## Build the archive

```sh
./scripts/release.sh                          # host target, version from Cargo.toml
./scripts/release.sh --target x86_64-unknown-linux-musl
./scripts/release.sh --version 0.2.0 --output dist
make release ARGS="--target aarch64-unknown-linux-musl"
```

The script:

- builds with `cargo build --release --locked`, using `cross` when a non-host
  target is requested and `cross` is installed;
- warns when the working tree is dirty or `--version` differs from
  `Cargo.toml`;
- stages the binary, `README.md`, `LICENSE`, and `examples/config`;
- writes `dist/emacette-v<version>-<target>.tar.gz` and a `.sha256` file.

It does not create a tag, push, or upload anything.

## Supported release targets

`install.sh` selects assets in this order:

| Platform | Preferred target | Fallback |
| --- | --- | --- |
| Linux x86_64 | `x86_64-unknown-linux-musl` | `x86_64-unknown-linux-gnu` |
| Linux aarch64 | `aarch64-unknown-linux-musl` | `aarch64-unknown-linux-gnu` |
| macOS arm64 | `universal-apple-darwin` | `aarch64-apple-darwin` |
| macOS x86_64 | `universal-apple-darwin` | `x86_64-apple-darwin` |

Build each target on its native platform or with `cross`. The release script
refuses `universal-apple-darwin`; build the two macOS architectures per
machine, or add a universal binary to the release manually.

## Publish

```sh
git tag v0.1.0
git push origin v0.1.0

gh release create v0.1.0 \
  dist/emacette-v0.1.0-*.tar.gz \
  dist/emacette-v0.1.0-*.tar.gz.sha256 \
  --generate-notes
```

Upload both the archive and its `.sha256` file for every target. `install.sh`
downloads the checksum next to the archive and verifies it before installing.
The latest release is discovered through the
`https://github.com/<owner>/<repo>/releases/latest` redirect, so the newest
published (non-draft, non-prerelease) release is the one users get.

## Installer contract

If you change any of these, update `install.sh` and its CI smoke test in the
same change:

- asset naming: `emacette-<tag>-<target>.tar.gz`;
- archive layout: a top-level `emacette` binary plus README, LICENSE, and
  `examples/config`;
- checksum naming: `<archive>.sha256`, first field is the hash;
- the default repository slug (`3tty0n/emacette`);
- supported target triples and the preferred/fallback order.

Legacy `NEMACS_REPO` and `NEMACS_BASE_URL` environment variables are still
accepted by the installer.

## After publishing

- Verify the published install path from a clean directory:

  ```sh
  curl -fsSL https://raw.githubusercontent.com/3tty0n/emacette/main/install.sh | sh
  emacette --version
  ```

- Check the release notes for missing acknowledgements or breaking changes.
- If a release is broken, mark it as a pre-release or delete it; the installer
  follows GitHub's "latest" marker.
