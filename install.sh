#!/bin/sh
# Install, update, or remove emacette.
#
# Installs a prebuilt binary from GitHub Releases by default. Use
# --from-source to build the checkout with cargo instead. Re-running the
# script updates the binary; --uninstall removes it. Configuration is never
# touched.
set -eu

REPO="${EMACETTE_REPO:-${NEMACS_REPO:-3tty0n/emacette}}"
BASE_URL="${EMACETTE_BASE_URL:-${NEMACS_BASE_URL:-https://github.com/$REPO/releases/download}}"

prefix="${PREFIX:-$HOME/.local}"
bindir=""
release=""
from_source=0
uninstall=0

usage() {
    cat <<'EOF'
Usage: install.sh [OPTIONS]

Install, update, or remove the emacette binary.

Options:
  --release TAG    Install a specific release tag (default: latest)
  --from-source    Build with cargo and install from this checkout
  --prefix DIR     Install under DIR (default: ~/.local)
  --uninstall      Remove the installed binary
  -h, --help       Show this help

Environment:
  PREFIX           Same as --prefix
  BINDIR           Binary directory (default: PREFIX/bin)
  EMACETTE_REPO        GitHub repository (default: 3tty0n/emacette)
  EMACETTE_BASE_URL    Release download base URL (mirrors, testing)
  NEMACS_REPO          Legacy alias for EMACETTE_REPO
  NEMACS_BASE_URL      Legacy alias for EMACETTE_BASE_URL

Supported assets:
  Linux x86_64/aarch64: static musl or glibc
  macOS x86_64/arm64: universal or per-architecture

Updating is the same as installing: run this script again.
Configuration in ~/.config/emacette is kept by --uninstall.
EOF
}

die() {
    printf 'install.sh: %s\n' "$*" >&2
    exit 1
}

need() {
    command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

while [ $# -gt 0 ]; do
    case "$1" in
        --from-source) from_source=1 ;;
        --uninstall) uninstall=1 ;;
        --prefix)
            [ $# -ge 2 ] || die "--prefix needs a directory"
            prefix=$2
            shift
            ;;
        --prefix=*) prefix=${1#*=} ;;
        --release | --tag)
            [ $# -ge 2 ] || die "$1 needs a tag"
            release=$2
            shift
            ;;
        --release=* | --tag=*) release=${1#*=} ;;
        -h | --help)
            usage
            exit 0
            ;;
        *) die "unknown option: $1 (try --help)" ;;
    esac
    shift
done

[ -n "$prefix" ] || die "empty install prefix"
bindir=${BINDIR:-"$prefix/bin"}

if [ "$uninstall" = 1 ]; then
    [ "$from_source" = 0 ] || die "--uninstall cannot be combined with --from-source"
    [ -z "$release" ] || die "--uninstall cannot be combined with --release"
    dest="$bindir/emacette"
    if [ -e "$dest" ]; then
        rm -f "$dest" || die "cannot remove $dest"
        printf 'Removed %s\n' "$dest"
    else
        printf 'Not installed: %s\n' "$dest"
    fi
    printf 'Configuration is kept in %s\n' "${XDG_CONFIG_HOME:-$HOME/.config}/emacette"
    exit 0
fi

install_binary() {
    src=$1
    dest="$bindir/emacette"
    tmp="$bindir/.emacette.$$"
    cp "$src" "$tmp" || die "cannot write to $bindir"
    chmod 755 "$tmp" || {
        rm -f "$tmp"
        die "cannot make $tmp executable"
    }
    mv -f "$tmp" "$dest" || {
        rm -f "$tmp"
        die "cannot replace $dest"
    }
    printf 'Installed %s\n' "$dest"
    installed=$("$dest" --version 2>/dev/null || true)
    [ -n "$installed" ] && printf '%s\n' "$installed"
}

if [ "$from_source" = 1 ]; then
    [ -z "$release" ] || die "--from-source cannot be combined with --release"
    need cargo
    script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
    [ -f "$script_dir/Cargo.toml" ] || die "--from-source needs a source checkout"
    (cd "$script_dir" && cargo build --release --locked) || die "cargo build failed"
    mkdir -p "$bindir" || die "cannot create $bindir"
    install_binary "$script_dir/target/release/emacette"
    exit 0
fi

need curl
need tar
need uname
need awk

os=$(uname -s)
arch=$(uname -m)
case "$os:$arch" in
    Linux:x86_64 | Linux:amd64)
        candidates="x86_64-unknown-linux-musl x86_64-unknown-linux-gnu"
        ;;
    Linux:aarch64 | Linux:arm64)
        candidates="aarch64-unknown-linux-musl aarch64-unknown-linux-gnu"
        ;;
    Darwin:arm64)
        candidates="universal-apple-darwin aarch64-apple-darwin"
        ;;
    Darwin:x86_64)
        candidates="universal-apple-darwin x86_64-apple-darwin"
        ;;
    *) die "unsupported platform: $os $arch (build with --from-source instead)" ;;
esac

if command -v sha256sum >/dev/null 2>&1; then
    sha256_cmd=sha256sum
elif command -v shasum >/dev/null 2>&1; then
    sha256_cmd=shasum
elif command -v openssl >/dev/null 2>&1; then
    sha256_cmd=openssl
else
    die "a SHA-256 tool is required (sha256sum, shasum, or openssl)"
fi

sha256_of() {
    case "$sha256_cmd" in
        sha256sum) sha256sum "$1" | awk '{print $1}' ;;
        shasum) shasum -a 256 "$1" | awk '{print $1}' ;;
        openssl) openssl dgst -sha256 "$1" | awk '{print $NF}' ;;
    esac
}

if [ -z "$release" ]; then
    latest_url=$(curl -fsSIL -o /dev/null -w '%{url_effective}' \
        "https://github.com/$REPO/releases/latest") ||
        die "cannot find the latest release for $REPO; pass --release TAG"
    release=${latest_url##*/}
    case "$release" in
        '' | latest | releases | tags)
            die "cannot find the latest release for $REPO; pass --release TAG"
            ;;
    esac
fi

tmpdir=$(mktemp -d "${TMPDIR:-/tmp}/emacette.XXXXXX") || die "cannot create a temporary directory"
trap 'rm -rf "$tmpdir"' EXIT HUP INT TERM

# Try the preferred asset for this platform, then fall back to the other
# format (static musl -> glibc, universal macOS -> per-architecture).
archive=""
url=""
for candidate in $candidates; do
    try_archive="emacette-${release}-${candidate}.tar.gz"
    try_url="${BASE_URL%/}/${release}/${try_archive}"
    printf 'Trying %s\n' "$try_url"
    if curl -fsSL "$try_url" -o "$tmpdir/$try_archive" 2>/dev/null; then
        archive=$try_archive
        url=$try_url
        break
    fi
done
[ -n "$archive" ] || die "no release asset found for $os $arch (tried: $candidates)"

curl -fsSL "$url.sha256" -o "$tmpdir/$archive.sha256" ||
    die "checksum download failed: $url.sha256"
expected=$(awk 'NR == 1 {print $1}' "$tmpdir/$archive.sha256")
[ -n "$expected" ] || die "empty checksum file: $url.sha256"
actual=$(sha256_of "$tmpdir/$archive")
[ "$expected" = "$actual" ] || die "checksum mismatch for $archive"

tar -xzf "$tmpdir/$archive" -C "$tmpdir" || die "cannot extract $archive"
[ -f "$tmpdir/emacette" ] || die "$archive does not contain an emacette binary"

mkdir -p "$bindir" || die "cannot create $bindir"
install_binary "$tmpdir/emacette"
