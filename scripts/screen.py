"""Reconstruct a text screen from the ANSI output of a PTY session.

Used by `make e2e` to make the end-to-end test failures readable.
"""

import re
import unicodedata

CSI = re.compile(r"\x1b\[([0-9;?]*)([A-Za-z~])")


def char_width(ch):
    if unicodedata.combining(ch):
        return 0
    return 2 if unicodedata.east_asian_width(ch) in ("W", "F") else 1


def screen_from_ansi(data, cols=80, rows=24):
    screen = [[" "] * cols for _ in range(rows)]
    y = x = 0
    text = data.decode("utf-8", "replace")
    i = 0
    while i < len(text):
        ch = text[i]
        if ch == "\x1b":
            match = CSI.match(text, i)
            if not match:
                i += 1
                continue
            params, command = match.group(1), match.group(2)
            if command == "H":
                parts = params.split(";") if params else []
                y = max(0, (int(parts[0]) if parts and parts[0] else 1) - 1)
                x = max(0, (int(parts[1]) if len(parts) > 1 and parts[1] else 1) - 1)
            elif command == "K":
                for xx in range(x, cols):
                    screen[y][xx] = " "
            elif command == "J":
                for yy in range(y + 1, rows):
                    screen[yy] = [" "] * cols
                for xx in range(x, cols):
                    screen[y][xx] = " "
            i = match.end()
            continue
        if ch == "\r":
            x = 0
            i += 1
            continue
        if ch == "\n":
            y = min(rows - 1, y + 1)
            i += 1
            continue
        width = char_width(ch)
        if 0 <= y < rows and 0 <= x < cols:
            screen[y][x] = ch
            if width == 2 and x + 1 < cols:
                screen[y][x + 1] = ""
        x += width
        i += 1

    lines = []
    for row in screen:
        lines.append("".join(cell for cell in row if cell != "").rstrip())
    return "\n".join(lines)
