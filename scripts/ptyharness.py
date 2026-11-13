"""Tiny PTY harness used by the e2e tests and the benchmark.

The functions here spawn a program on a pseudo-terminal of a known size,
feed it bytes and collect the output.  Nothing in this module is imported
by the editor itself; it exists purely for development and CI.
"""

import fcntl
import os
import pty
import select
import signal
import struct
import termios
import time


def spawn(argv, cols=80, rows=24):
    """Fork/exec `argv` on a new pty.  Returns (pid, fd)."""
    pid, fd = pty.fork()
    if pid == 0:  # child
        try:
            os.execvp(argv[0], argv)
        finally:
            os._exit(127)
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))
    return pid, fd


def read_available(fd, timeout=0.05):
    r, _, _ = select.select([fd], [], [], timeout)
    if not r:
        return b""
    try:
        return os.read(fd, 65536)
    except OSError:
        return b""


def drain_quiet(fd, out, first_output, start, quiet=0.03, max_wait=0.3):
    """Read until the program has been quiet for a short moment.

    The editor coalesces buffered keys and paints once, so a fixed
    ``time.sleep`` between keys is not enough on a slow runner: a prompt can
    be set up and replaced before it is ever rendered.  Draining after each
    key gives the next frame a chance to appear and keeps the captured output
    deterministic.
    """
    deadline = time.monotonic() + max_wait
    quiet_since = time.monotonic()
    while time.monotonic() < deadline:
        chunk = read_available(fd, 0.01)
        now = time.monotonic()
        if chunk:
            out += chunk
            if first_output is None:
                first_output = now - start
            quiet_since = now
        elif now - quiet_since >= quiet:
            break
    return first_output


def run(
    argv,
    keys,
    cols=80,
    rows=24,
    settle=0.35,
    timeout=10,
    key_delay=0.015,
    expect=None,
    expect_timeout=6.0,
):
    """Run `argv`, wait until it is ready, send `keys`, wait for exit.

    Readiness is either "no output for `settle` seconds" or, when `expect`
    is given, the output containing those bytes (useful when a wrapper such
    as git starts the editor).
    Returns (exit_code, captured_output, seconds_until_first_output).
    """
    start = time.monotonic()
    pid, fd = spawn(argv, cols, rows)
    first_output = None
    out = bytearray()

    deadline = start + timeout
    expect_deadline = start + expect_timeout
    # Drain the initial frame(s) until the program is ready.
    quiet_since = time.monotonic()
    while time.monotonic() < deadline:
        chunk = read_available(fd, 0.02)
        now = time.monotonic()
        if chunk:
            out += chunk
            if first_output is None:
                first_output = now - start
            quiet_since = now
        if expect is not None:
            if expect in out:
                break
            if now > expect_deadline:
                break
        elif now - quiet_since > settle:
            break
        # Did it exit early?
        wpid, status = os.waitpid(pid, os.WNOHANG)
        if wpid == pid:
            _drain_final(fd, out)
            return os.waitstatus_to_exitcode(status), bytes(out), first_output

    for key in keys:
        try:
            os.write(fd, key)
        except OSError:
            break
        time.sleep(key_delay)
        first_output = drain_quiet(fd, out, first_output, start)

    # The per-key drains can take a few hundred milliseconds in total.  Keep
    # at least a short window for the program to finish after the last key.
    deadline = max(deadline, time.monotonic() + 2.0)

    status = None
    while time.monotonic() < deadline:
        chunk = read_available(fd, 0.05)
        if chunk:
            out += chunk
            if first_output is None:
                first_output = time.monotonic() - start
        wpid, st = os.waitpid(pid, os.WNOHANG)
        if wpid == pid:
            status = st
            break
    if status is None:
        os.kill(pid, signal.SIGKILL)
        os.waitpid(pid, 0)
        raise RuntimeError("timeout waiting for %r" % (argv,))
    _drain_final(fd, out)
    os.close(fd)
    return os.waitstatus_to_exitcode(status), bytes(out), first_output


def _drain_final(fd, out):
    try:
        while True:
            chunk = read_available(fd, 0.02)
            if not chunk:
                break
            out += chunk
    except OSError:
        pass


def time_to_first_output(argv, cols=80, rows=24, timeout=10):
    """Seconds from spawn until the program first writes to the terminal."""
    start = time.monotonic()
    pid, fd = spawn(argv, cols, rows)
    deadline = start + timeout
    try:
        while time.monotonic() < deadline:
            chunk = read_available(fd, 0.05)
            if chunk:
                return time.monotonic() - start
            wpid, _ = os.waitpid(pid, os.WNOHANG)
            if wpid == pid:
                return None
        return None
    finally:
        try:
            os.kill(pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        try:
            os.waitpid(pid, 0)
        except ChildProcessError:
            pass
        os.close(fd)
