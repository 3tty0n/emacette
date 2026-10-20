#!/usr/bin/env python3
"""Measure time-to-first-paint for emacette and friends.

This is a deliberately simple benchmark: spawn the editor on a pty, measure
how long it takes to write its first byte, then kill it.  It captures the
"can I start typing yet?" impression rather than a full workflow.

Usage: python3 scripts/bench.py [--runs N] [binary]
"""

import os
import shutil
import statistics
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, HERE)

from ptyharness import time_to_first_output  # noqa: E402

RUNS = 9


def parse_args():
    global RUNS
    args = sys.argv[1:]
    binary = os.path.join(ROOT, "target", "release", "emacette")
    while args:
        arg = args.pop(0)
        if arg in ("-n", "--runs"):
            RUNS = max(1, int(args.pop(0)))
        elif arg in ("-h", "--help"):
            print(__doc__)
            sys.exit(0)
        else:
            binary = os.path.abspath(arg)
    return binary


def sample_file():
    directory = tempfile.mkdtemp(prefix="emacette-bench-")
    path = os.path.join(directory, "sample.rs")
    lines = ["// benchmark sample", "fn main() {"]
    for i in range(40):
        lines.append(f'    let value_{i} = "line {i}";')
    lines += ["    println!(\"{:?}\", value_0);", "}", ""]
    with open(path, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines))
    return directory, path


def measure(label, argv):
    times = []
    for _ in range(RUNS):
        elapsed = time_to_first_output(argv)
        if elapsed is None:
            print(f"{label}: did not start")
            return None
        times.append(elapsed * 1000.0)
    times.sort()
    median = statistics.median(times)
    print(
        f"{label:<16} median {median:7.2f} ms   min {times[0]:7.2f} ms   "
        f"max {times[-1]:7.2f} ms   ({RUNS} runs, first paint)"
    )
    return median


def main():
    binary = parse_args()
    if not os.path.exists(binary):
        print(f"binary not found: {binary}\nrun `cargo build --release` first")
        return 2
    directory, sample = sample_file()
    print(f"binary: {binary}")
    print(f"sample: {sample} (44 lines)")
    print()
    try:
        measure("emacette", [binary, sample])
        for label, command in [
            ("nano", ["nano", sample]),
            ("vim", ["vim", "-u", "NONE", "-n", sample]),
            ("emacs -nw -Q", ["emacs", "-nw", "-Q", sample]),
        ]:
            executable = shutil.which(command[0])
            if executable:
                measure(label, [executable] + command[1:])
            else:
                print(f"{label:<16} not installed")
    finally:
        shutil.rmtree(directory, ignore_errors=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
