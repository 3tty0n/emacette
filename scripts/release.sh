#!/bin/sh
# Build and package an emacette release archive locally.
#
# This script does not tag or upload anything. It leaves the archive and its
# SHA-256 file in the output directory and prints the commands to publish
# them manually.
#
# macOS notes:
#   - Apple targets are built with cargo and the Xcode SDK on macOS.  The
#     script never trusts `strip` from PATH: Apple binaries are stripped with
#     /usr/bin/strip, ad-hoc signed with codesign, and executed once to prove
#     that the packaged file actually starts.  A GNU/Homebrew/MacPorts/Nix
#     strip can produce a malformed Mach-O that macOS kills on launch.
#   - `--target universal-apple-darwin` builds both Apple architectures and
#     joins them with lipo.  It requires macOS.
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
  --target TRIPLE  Rust target triple, or universal-apple-darwin
                   (default: host target)
  --output DIR     Output directory (default: dist)
  -h, --help       Show this help

Notes:
  - Non-Apple cross targets use cross when it is installed, otherwise cargo.
  - Apple targets are built with cargo and the Xcode SDK on macOS.
  - universal-apple-darwin builds aarch64-apple-darwin and
    x86_64-apple-darwin and joins them with lipo; it requires macOS.
  - macOS binaries are stripped with /usr/bin/strip and ad-hoc signed with
    codesign.  The script runs the result before packaging it.
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

os=$(uname -s)
if [ "$os" = Darwin ]; then
    # Keep Apple's toolchain ahead of Homebrew/MacPorts/Nix binutils.  rustc
    # asks PATH for `strip` if a profile or RUSTFLAGS requests it, and the
    # wrong strip can produce a binary that macOS kills on launch.
    PATH="/usr/bin:/bin:/usr/sbin:/sbin:$PATH"
    export PATH
fi
case "$target" in
    universal-apple-darwin)
        [ "$os" = Darwin ] ||
            die "universal-apple-darwin builds require macOS (lipo and codesign)"
        need lipo
        need codesign
        ;;
    *-apple-darwin)
        [ "$os" = Darwin ] || die "Apple targets must be built on macOS"
        need codesign
        ;;
esac

built_bin=""
build_one() {
    build_target=$1
    if [ "$build_target" = "$host" ]; then
        cargo build --release --locked || die "cargo build failed"
        built_bin="target/release/emacette"
    elif [ "$os" = Darwin ]; then
        cargo build --release --locked --target "$build_target" ||
            die "cargo build failed for $build_target (run 'rustup target add $build_target')"
        built_bin="target/$build_target/release/emacette"
    elif command -v cross >/dev/null 2>&1; then
        cross build --release --locked --target "$build_target" ||
            die "cross build failed for $build_target"
        built_bin="target/$build_target/release/emacette"
    else
        cargo build --release --locked --target "$build_target" ||
            die "cargo build failed for $build_target (run 'rustup target add $build_target' or install cross)"
        built_bin="target/$build_target/release/emacette"
    fi
    [ -f "$built_bin" ] || die "binary not found: $built_bin"
}

strip_apple_binary() {
    [ -x /usr/bin/strip ] ||
        die "macOS strip not found at /usr/bin/strip (install the Xcode command line tools)"
    /usr/bin/strip -x "$1" || die "failed to strip $1 with /usr/bin/strip"
}

sign_apple_binary() {
    need codesign
    codesign --force --sign - --timestamp=none "$1" ||
        die "ad-hoc codesign failed for $1"
    codesign --verify --strict "$1" ||
        die "codesign verification failed for $1"
}

strip_host_binary() {
    # Prefer the platform tool, not whatever a language environment or a
    # cross toolchain put first on PATH.
    if [ -x /usr/bin/strip ]; then
        /usr/bin/strip --strip-all "$1" || die "failed to strip $1"
    elif command -v strip >/dev/null 2>&1; then
        strip --strip-all "$1" || die "failed to strip $1"
    else
        printf 'warning: strip not found; packaging an unstripped binary\n' >&2
    fi
}

verify_arch() {
    bin_path=$1
    case "$target" in
        universal-apple-darwin)
            need lipo
            archs=$(lipo -archs "$bin_path") || die "cannot inspect $bin_path"
            case " $archs " in
                *" arm64 "*) ;;
                *) die "$bin_path is missing the arm64 slice (got: $archs)" ;;
            esac
            case " $archs " in
                *" x86_64 "*) ;;
                *) die "$bin_path is missing the x86_64 slice (got: $archs)" ;;
            esac
            ;;
        aarch64-apple-darwin)
            need lipo
            archs=$(lipo -archs "$bin_path") || die "cannot inspect $bin_path"
            case " $archs " in
                *" arm64 "*) ;;
                *) die "$bin_path is not an arm64 Mach-O (got: $archs)" ;;
            esac
            ;;
        x86_64-apple-darwin)
            need lipo
            archs=$(lipo -archs "$bin_path") || die "cannot inspect $bin_path"
            case " $archs " in
                *" x86_64 "*) ;;
                *) die "$bin_path is not an x86_64 Mach-O (got: $archs)" ;;
            esac
            ;;
        *)
            if command -v file >/dev/null 2>&1; then
                file "$bin_path" | grep -q 'ELF' ||
                    die "$bin_path is not an ELF binary"
            fi
            ;;
    esac
}

# Print the version, or return 2 when this host cannot execute the target.
run_binary() {
    if [ "$target" = "$host" ] || [ "$target" = universal-apple-darwin ]; then
        "$bin" --version
    elif [ "$os" = Darwin ] && [ "$target" = x86_64-apple-darwin ] &&
        [ "$(uname -m)" = arm64 ] && [ -x /usr/bin/arch ] &&
        /usr/bin/arch -x86_64 /usr/bin/true >/dev/null 2>&1; then
        # Rosetta is installed: run the Intel slice for a real smoke test.
        /usr/bin/arch -x86_64 "$bin" --version
    else
        return 2
    fi
}

case "$target" in
    universal-apple-darwin)
        build_one aarch64-apple-darwin
        arm_bin=$built_bin
        build_one x86_64-apple-darwin
        x86_bin=$built_bin

        strip_apple_binary "$arm_bin"
        sign_apple_binary "$arm_bin"
        strip_apple_binary "$x86_bin"
        sign_apple_binary "$x86_bin"

        bin="target/universal-apple-darwin/release/emacette"
        mkdir -p "$(dirname "$bin")"
        lipo -create -output "$bin" "$arm_bin" "$x86_bin" ||
            die "lipo failed"
        chmod 755 "$bin"
        sign_apple_binary "$bin"
        ;;
    *-apple-darwin)
        build_one "$target"
        bin=$built_bin
        strip_apple_binary "$bin"
        sign_apple_binary "$bin"
        ;;
    "$host")
        build_one "$target"
        bin=$built_bin
        strip_host_binary "$bin"
        ;;
    *)
        build_one "$target"
        bin=$built_bin
        printf 'note: not stripping %s (cross build)\n' "$target" >&2
        ;;
esac

[ -f "$bin" ] || die "binary not found: $bin"
verify_arch "$bin"

status=0
version_out=$(run_binary 2>/dev/null) || status=$?
if [ "$status" -eq 0 ]; then
    case "$version_out" in
        "emacette $version"*) ;;
        *)
            die "binary reports '$version_out', expected 'emacette $version'"
            ;;
    esac
    printf 'Verified: %s\n' "$version_out"
elif [ "$status" -eq 2 ]; then
    printf 'note: cannot execute %s on this host; skipping the runtime check\n' \
        "$target" >&2
else
    printf 'release.sh: the packaged binary failed to run; output follows\n' >&2
    run_binary >&2 2>&1 || true
    die "binary failed to run: $bin"
fi

tmp=$(mktemp -d "${TMPDIR:-/tmp}/emacette-release.XXXXXX") ||
    die "cannot create a temporary directory"
trap 'rm -rf "$tmp"' EXIT HUP INT TERM

stage="$tmp/package"
mkdir -p "$stage"
cp "$bin" "$stage/emacette"
cp README.md LICENSE "$stage/"
mkdir -p "$stage/examples"
cp examples/config examples/README.md "$stage/examples/"
chmod 755 "$stage/emacette"

archive="emacette-${tag}-${target}.tar.gz"
mkdir -p "$output"
COPYFILE_DISABLE=1 tar -czf "$output/$archive" -C "$stage" emacette README.md LICENSE examples/config

if command -v sha256sum >/dev/null 2>&1; then
    (cd "$output" && sha256sum "$archive" > "$archive.sha256")
elif command -v shasum >/dev/null 2>&1; then
    (cd "$output" && shasum -a 256 "$archive" > "$archive.sha256")
else
    die "a SHA-256 tool is required (sha256sum or shasum)"
fi

printf 'Created %s\n' "$output/$archive"
printf 'Created %s\n' "$output/$archive.sha256"
case "$target" in
    *-apple-darwin)
        printf 'macOS package: ad-hoc signed and verified\n'
        ;;
esac
printf '\nPublish manually:\n'
printf '  git tag %s\n' "$tag"
printf '  git push origin %s\n' "$tag"
printf '  gh release create %s %s %s --generate-notes\n' \
    "$tag" "$output/$archive" "$output/$archive.sha256"
printf '\nOr create the release in the GitHub web UI and upload both files.\n'
