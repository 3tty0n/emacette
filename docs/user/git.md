# Git integration

Part of the [emacette user guide](README.md).

## Set as the default editor

```sh
git config --global core.editor emacette
git config --global core.editor "$HOME/.local/bin/emacette"     # not on PATH
GIT_EDITOR=emacette git commit                                   # one-off
```

## Commit workflow

- `C-c C-c` saves the message and exits.
- `C-c C-k` exits without saving; git aborts the commit. A prompt appears
  only when the message text has changed.
- `C-x C-s` followed by `C-x C-c` also saves and exits.
- `C-c C-c` asks for confirmation when the message is empty or contains only
  comments.

## Details

- The same editor serves `git commit --amend`, `git tag -a`, `git rebase -i`,
  and every other command that invokes `$EDITOR`.
- Files such as `COMMIT_EDITMSG`, `MERGE_MSG`, `TAG_EDITMSG`, and `SQUASH_MSG`
  open in Git commit mode.
- Set `comment_char` to match `core.commentChar` when it is not `#`. See
  [Configuration](configuration.md).
- The prefix can also be set per language, for example
  `[gitcommit] comment_start = ";"`.
