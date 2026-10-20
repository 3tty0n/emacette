# Spelling

Part of [nano-emacs](../README.md).

Spell checking is English-only for now. It highlights misspelled words in
prose modes and can suggest corrections.

## Supported modes

- Plain text
- Markdown
- LaTeX
- reStructuredText
- AsciiDoc
- Git commit messages

Code, comments, math, and markup commands are skipped using syntax spans.
LaTeX arguments for `label`, `ref`, `cite`, `url`, `includegraphics`, and
similar commands are also skipped.

## Dictionary discovery

The checker uses the first available source:

- `spell_dictionary` from the configuration file.
- `$NANO_EMACS_DICT`.
- `/usr/share/dict/words`.
- `/usr/share/dict/american-english`.
- `/usr/share/dict/british-english`.
- `/usr/share/hunspell/en_US.dic`.
- `/usr/share/hunspell/en_GB.dic`.
- `/usr/share/myspell/dicts/en_US.dic`.
- `/usr/share/myspell/dicts/en_GB.dic`.
- `/Library/Spelling/en_US.dic`.
- `/Library/Spelling/en_GB.dic`.
- `~/.local/share/hunspell/en_US.dic` or `en_GB.dic`.

A plain word list uses one word per line. A hunspell `.dic` file uses its base
words; affix expansions are not applied, so inflected forms may be flagged.

## Configuration

- `spell_check = true` — enable spell checking at startup.
- `spell_dictionary = /path/to/words` — use an explicit dictionary.
- Both keys can be overridden in a language section, for example
  `[markdown]` or `[latex]`. See [Configuration](configuration.md).
- `NANO_EMACS_DICT=/path/to/words nemacs FILE` — one-off dictionary override.

See [Configuration](configuration.md) for the rest of the options.

## Commands

- `M-x flyspell-mode` — toggle highlighting.
- `M-x spell-check-mode` — alias for `flyspell-mode`.
- `M-$` or `M-x ispell-word` — prompt to replace the word at point.
- `M-x ispell-next` — jump to the next misspelled word.
- `M-x ispell-previous` — jump to the previous misspelled word.
- `M-x ispell-buffer` — check the whole buffer and report the count.

## Display

- Misspelled words are underlined in the terminal.
- `M-$` pre-fills the first suggestion; edit it and press `RET` to replace, or `C-g` to cancel.
- The mode line shows `Spell` while checking is enabled.
- If no dictionary is found, the editor reports an error instead of enabling
  the mode.

## Limitations

- English dictionaries only.
- No personal dictionary or ignore list yet.
- Hunspell affixes are not expanded.
- Contractions such as `don't` are skipped rather than checked.
- Acronyms written entirely in uppercase are skipped.
- The checker is synchronous; very large buffers may pause while checking.
