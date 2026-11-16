"""Bump the package version in Cargo.toml and Cargo.lock together."""

import argparse
from pathlib import Path
import re


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("part", choices=("major", "minor", "patch"))
    args = parser.parse_args()
    manifest = Path("Cargo.toml")
    lock = Path("Cargo.lock")
    source = manifest.read_text()
    locked = lock.read_text()
    pattern = r'(\[package\]\s+name = "emacette"\s+version = ")(\d+\.\d+\.\d+)(")'
    match = re.search(pattern, source)
    if match is None:
        parser.error("expected an emacette package with an X.Y.Z version")
    old = match[2]
    parts = list(map(int, old.split(".")))
    index = ("major", "minor", "patch").index(args.part)
    parts[index] += 1
    parts[index + 1:] = [0] * (2 - index)
    new = ".".join(map(str, parts))
    entry = f'name = "emacette"\nversion = "{old}"'
    if locked.count(entry) != 1:
        parser.error("Cargo.lock must contain the same emacette version")
    manifest.write_text(source[:match.start(2)] + new + source[match.end(2):])
    lock.write_text(locked.replace(entry, f'name = "emacette"\nversion = "{new}"'))
    print(f"{old} -> {new}")


if __name__ == "__main__":
    main()
