#!/bin/sh
# Build and package an emacette release archive locally.
#
# This script does not tag or upload anything. It leaves the archive and its
# SHA-256 file in the output directory and prints the commands to publish
# them manually.
set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
root=$(dirname "$script_dir")
cd "$root"

version=""
target=""
output="dist"

usage() {
    cat <<'EOF'
Usage: scripts/release.sh [OPTIONS]

Build and package an emacette release archive locally. This does not tag or
upload anything.

Options:
  --version X.Y.Z  Release version (default: Cargo.toml version)
  --target TRIPLE  Rust target triple (default: host target)
  --output DIR     Output directory (default: dist)
  -h, --help       Show this help

Notes:
  - Non-host targets use cross when it is installed, otherwise cargo.
  - macOS universal binaries are not built here; build aarch64-apple-darwin
    or x86_64-apple-darwin per machine. install.sh accepts either.
  - Commit or stash changes before releasing; this script only warns.
EOF
}

die() {
    printf 'release.sh: %s\n' "$*" >&2
    exit 1
}

need() {
    command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

while [ $# -gt 0 ]; do
    case "$1" in
        --version)
            [ $# -ge 2 ] || die "--version needs a value"
            version=$2
            shift
            ;;
        --version=*) version=${1#*=} ;;
        --target)
            [ $# -ge 2 ] || die "--target needs a value"
            target=$2
            shift
            ;;
        --target=*) target=${1#*=} ;;
        --output)
            [ $# -ge 2 ] || die "--output needs a directory"
            output=$2
            shift
            ;;
        --output=*) output=${1#*=} ;;
        -h | --help)
            usage
            exit 0
            ;;
        *) die "unknown option: $1 (try --help)" ;;
    esac
    shift
done

need cargo
need rustc
need tar
need awk

cargo_version=$(awk -F'"' '/^version = / {print $2; exit}' Cargo.toml)
[ -n "$cargo_version" ] || die "cannot read the package version from Cargo.toml"

version=${version#v}
version=${version:-$cargo_version}
if [ "$version" != "$cargo_version" ]; then
    printf 'warning: release version %s differs from Cargo.toml (%s)\n' \
        "$version" "$cargo_version" >&2
fi
tag="v$version"

host=$(rustc -vV | awk '/^host:/ {print $2}')
[ -n "$host" ] || die "cannot determine the host target"
target=${target:-$host}

if [ -n "$(git status --porcelain 2>/dev/null || true)" ]; then
    printf 'warning: working tree has uncommitted changes\n' >&2
fi

[ "$target" != "universal-apple-darwin" ] ||
    die "build per-architecture macOS binaries (aarch64-apple-darwin or x86_64-apple-darwin)"

if [ "$target" = "$host" ]; then
    cargo build --release --locked || die "cargo build failed"
    bin="target/release/emacette"
elif command -v cross >/dev/null 2>&1; then
    cross build --release --locked --target "$target" || die "cross build failed for $target"
    bin="target/$target/release/emacette"
else
    cargo build --release --locked --target "$target" ||
        die "cargo build failed for $target (run 'rustup target add $target' or install cross)"
    bin="target/$target/release/emacette"
fi
[ -f "$bin" ] || die "binary not found: $bin"

tmp=$(mktemp -d "${TMPDIR:-/tmp}/emacette-release.XXXXXX") ||
    die "cannot create a temporary directory"
trap 'rm -rf "$tmp"' EXIT HUP INT TERM

stage="$tmp/package"
mkdir -p "$stage"
cp "$bin" "$stage/emacette"
cp README.md LICENSE "$stage/"
chmod 755 "$stage/emacette"

archive="emacette-${tag}-${target}.tar.gz"
mkdir -p "$output"
COPYFILE_DISABLE=1 tar -czf "$output/$archive" -C "$stage" emacette README.md LICENSE

if command -v sha256sum >/dev/null 2>&1; then
    (cd "$output" && sha256sum "$archive" > "$archive.sha256")
elif command -v shasum >/dev/null 2>&1; then
    (cd "$output" && shasum -a 256 "$archive" > "$archive.sha256")
else
    die "a SHA-256 tool is required (sha256sum or shasum)"
fi

printf 'Created %s\n' "$output/$archive"
printf 'Created %s\n' "$output/$archive.sha256"
printf '\nPublish manually:\n'
printf '  git tag %s\n' "$tag"
printf '  git push origin %s\n' "$tag"
printf '  gh release create %s %s %s --generate-notes\n' \
    "$tag" "$output/$archive" "$output/$archive.sha256"
printf '\nOr create the release in the GitHub web UI and upload both files.\n'
