//! Static help text, shown by F1 and by `--help`.

pub enum HelpLine {
    Heading(&'static str),
    Key(&'static str, &'static str),
    Text(&'static str),
    Blank,
}

pub static HELP: &[HelpLine] = &[
    HelpLine::Heading("nano-emacs - a tiny Emacs-flavoured editor"),
    HelpLine::Blank,
    HelpLine::Text("F1 / q / C-g close this help.  C-n, C-p, SPC scroll."),
    HelpLine::Blank,
    HelpLine::Heading("Files and git"),
    HelpLine::Key("C-x C-s", "save buffer"),
    HelpLine::Key("C-x C-w", "write buffer to a file (save as)"),
    HelpLine::Key("C-x C-f", "find file"),
    HelpLine::Key("C-x C-c", "quit, asking before discarding changes"),
    HelpLine::Key("C-c C-c", "save and quit (finish a git commit)"),
    HelpLine::Key("C-c C-k", "abort a git commit without saving"),
    HelpLine::Blank,
    HelpLine::Heading("Moving"),
    HelpLine::Key("C-f  C-b", "forward / backward one character"),
    HelpLine::Key("C-n  C-p", "next / previous line"),
    HelpLine::Key("M-f  M-b", "forward / backward one word"),
    HelpLine::Key("C-a  C-e", "beginning / end of line"),
    HelpLine::Key("M-<  M->", "beginning / end of buffer"),
    HelpLine::Key("C-v  M-v", "page down / page up"),
    HelpLine::Key("M-g g", "go to line"),
    HelpLine::Key("C-l", "recentre the screen"),
    HelpLine::Blank,
    HelpLine::Heading("Editing"),
    HelpLine::Key("C-d  DEL", "delete char; delete active region"),
    HelpLine::Key("M-d  M-DEL", "kill word forward / backward"),
    HelpLine::Key("C-k", "kill to end of line"),
    HelpLine::Key("C-y  M-y", "yank / yank-pop"),
    HelpLine::Key("C-SPC", "set mark / start a region"),
    HelpLine::Key("C-w  M-w", "kill / copy region"),
    HelpLine::Key("C-x C-x", "exchange point and mark"),
    HelpLine::Key("C-x h", "mark whole buffer"),
    HelpLine::Key("C-g", "deactivate mark / cancel"),
    HelpLine::Key("C-t", "transpose characters"),
    HelpLine::Key("M-u M-l M-c", "upcase / downcase / capitalise word"),
    HelpLine::Key("C-o", "open a line below point"),
    HelpLine::Key("C-/ or C-_", "undo"),
    HelpLine::Key("TAB", "indent (insert spaces to the next tab stop)"),
    HelpLine::Blank,
    HelpLine::Heading("Searching"),
    HelpLine::Key("C-s", "incremental search forward"),
    HelpLine::Key("C-r", "incremental search backward"),
    HelpLine::Key("C-s again", "next match (C-r: previous)"),
    HelpLine::Key("RET", "accept the match (mark is set)"),
    HelpLine::Key("C-g", "abort the search"),
    HelpLine::Blank,
    HelpLine::Heading("Minibuffer"),
    HelpLine::Key("M-x", "run a command by name"),
    HelpLine::Key("TAB  S-TAB", "complete / cycle candidates"),
    HelpLine::Key("C-g", "cancel"),
    HelpLine::Blank,
    HelpLine::Heading("Notes"),
    HelpLine::Text("Alt (Meta) works as ESC followed by the key."),
    HelpLine::Text("M-x auto-fill-mode / toggle-truncate-lines"),
    HelpLine::Text("M-x kill-region, kill-ring-save, delete-region"),
    HelpLine::Text("Configuration: ~/.config/nano-emacs/config"),
    HelpLine::Text("  auto_fill, fill_column, truncate_lines, ..."),
];

/// Plain text version for `--help`.
pub fn plain_text() -> String {
    let mut out = String::new();
    for line in HELP {
        match line {
            HelpLine::Heading(h) => {
                out.push('\n');
                out.push_str(h);
                out.push('\n');
            }
            HelpLine::Key(k, d) => {
                out.push_str(&format!("  {:<14} {}\n", k, d));
            }
            HelpLine::Text(t) => {
                out.push_str("  ");
                out.push_str(t);
                out.push('\n');
            }
            HelpLine::Blank => out.push('\n'),
        }
    }
    out.trim_start_matches('\n').to_string()
}
