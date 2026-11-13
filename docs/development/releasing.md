# Releasing

Releases are built and published by the
[release workflow](../../.github/workflows/release.yml) when a version tag is
pushed. `scripts/release.sh` remains available for local builds and as a
manual fallback.

## Versioning

- The version lives in `Cargo.toml` and is reported by `emacette --version`.
- Tags are `vMAJOR.MINOR.PATCH` (for example `v0.1.0`).
- The tag must match the version in `Cargo.toml`; the workflow fails otherwise.
- The project is pre-1.0: minor releases may contain user-visible changes, and
  patch releases should stay backward compatible within a minor series.
- Update `Cargo.lock` after changing the version (a normal `cargo build` does
  this) so `--locked` builds keep working.

## Pre-release checklist

1. `git status` is clean, and the branch is up to date.
2. `Cargo.toml` has the intended version and `Cargo.lock` is updated.
3. `make check` passes; CI is green (format, clippy, tests, release build,
   installer checks, PTY suite, and the macOS arm64/Intel jobs).
4. User-visible changes are reflected in the user guide, README feature list,
   or `examples/config`.
5. The manual smoke test in [Testing](testing.md#manual-smoke-test) passes.
6. `make bench` still looks reasonable if startup or rendering changed.

## Automated release

```sh
git tag v0.1.0
git push origin v0.1.0
```

The release workflow:

1. **verify** — checks that the tag matches `Cargo.toml` and runs
   `cargo test --locked`.
2. **build** — builds and packages each target with `scripts/release.sh`.
3. **publish** — verifies every archive and checksum, runs `install.sh`
   against the built assets, then creates the GitHub release with generated
   notes. If the release already exists, its assets are replaced with
   `gh release upload --clobber`.

The workflow uses the repository's `GITHUB_TOKEN`; no extra secrets are
required. It does not tag commits itself, so push the tag only after the release
commit is merged and CI is green.

### Targets

| Target | Runner | Build |
| --- | --- | --- |
| `x86_64-unknown-linux-musl` | `ubuntu-latest` | `cross` |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | native |
| `aarch64-unknown-linux-musl` | `ubuntu-latest` | `cross` |
| `aarch64-unknown-linux-gnu` | `ubuntu-latest` | `cross` |
| `x86_64-apple-darwin` | `macos-15-intel` | native |
| `aarch64-apple-darwin` | `macos-15` | native |
| `universal-apple-darwin` | `macos-15` | builds both Apple slices and joins them with `lipo` |

Each macOS job verifies the code signature, checks the architecture, runs
`emacette --version` from the packaged archive, and exercises `install.sh`
against the built assets. The arm64 job also builds and verifies the universal
archive.

Each target produces:

- `emacette-v<version>-<target>.tar.gz`
- `emacette-v<version>-<target>.tar.gz.sha256`

The archives contain the `emacette` binary, `README.md`, `LICENSE`, and
`examples/config`.

## Local build

```sh
./scripts/release.sh                          # host target, version from Cargo.toml
./scripts/release.sh --target x86_64-unknown-linux-musl
./scripts/release.sh --target universal-apple-darwin   # macOS only
./scripts/release.sh --version 0.2.0 --output dist
make release ARGS="--target aarch64-unknown-linux-musl"
```

The script:

- builds with `cargo build --release --locked`, using `cross` when a non-host
  Linux target is requested and `cross` is installed (Apple targets use the
  Xcode SDK on macOS instead);
- strips Apple binaries with `/usr/bin/strip`, ad-hoc signs them with
  `codesign`, and verifies both the signature and the architecture;
- runs the packaged binary once (`--version`) when the host can execute it, so
  a malformed or wrong-architecture artifact fails before publication;
- builds both Apple architectures and joins them with `lipo` for
  `universal-apple-darwin`;
- warns when the working tree is dirty or `--version` differs from
  `Cargo.toml`;
- stages the binary, `README.md`, `LICENSE`, and `examples/config`;
- writes `dist/emacette-v<version>-<target>.tar.gz` and a `.sha256` file.

It does not create a tag, push, or upload anything.

### Why the release profile does not set `strip`

On macOS, rustc runs the `strip` command found on `PATH` when cargo asks it to
strip a binary. GNU binutils from Homebrew, MacPorts, or Nix can produce a
Mach-O file that macOS rejects with `Killed: 9`, including when git launches
the editor as `$EDITOR`. `Cargo.toml` therefore leaves `strip` unset for
ordinary builds, and `scripts/release.sh` strips macOS binaries with
`/usr/bin/strip` before signing and smoke-testing them. Do not add
`strip = true` back to `[profile.release]`.

## Manual publish

If the workflow is unavailable, build each target locally with
`scripts/release.sh`, then publish the files with the GitHub CLI:

```sh
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

## Supported assets

`install.sh` selects assets in this order:

| Platform | Preferred target | Fallback |
| --- | --- | --- |
| Linux x86_64 | `x86_64-unknown-linux-musl` | `x86_64-unknown-linux-gnu` |
| Linux aarch64 | `aarch64-unknown-linux-musl` | `aarch64-unknown-linux-gnu` |
| macOS arm64 | `universal-apple-darwin` | `aarch64-apple-darwin` |
| macOS x86_64 | `universal-apple-darwin` | `x86_64-apple-darwin` |

The automated workflow publishes all three macOS archives, so installs use
the universal one by default and fall back to a per-architecture build only if
the universal asset is missing. On Apple silicon, `install.sh` checks
`hw.optional.arm64` so it still prefers the universal/arm64 asset when the
shell runs under Rosetta.

## Installer contract

If you change any of these, update `install.sh`, the release workflow, and the
CI smoke test in the same change:

- asset naming: `emacette-<tag>-<target>.tar.gz`;
- archive layout: a top-level `emacette` binary plus README, LICENSE, and
  `examples/config`;
- checksum naming: `<archive>.sha256`, first field is the hash;
- the default repository slug (`3tty0n/emacette`);
- supported target triples and the preferred/fallback order;
- macOS packaging: ad-hoc code signature, quarantine removal during install,
  and the post-install `emacette --version` check.

Legacy `NEMACS_REPO` and `NEMACS_BASE_URL` environment variables are still
accepted by the installer.

## After publishing

- Verify the published install path from a clean directory:

  ```sh
  curl -fsSL https://raw.githubusercontent.com/3tty0n/emacette/main/install.sh | sh
  emacette --version
  ```

- On macOS, confirm the installed file is signed and starts:

  ```sh
  codesign --verify --strict "$HOME/.local/bin/emacette"
  lipo -archs "$HOME/.local/bin/emacette"   # x86_64 arm64 for the universal build
  git commit                                # with core.editor = emacette
  ```

- Check the release notes for missing acknowledgements or breaking changes.
- If a release is broken, mark it as a pre-release or delete it; the installer
  follows GitHub's "latest" marker.
