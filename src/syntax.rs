//! A small, line-oriented syntax highlighter.
//!
//! Each line is lexed on its own, carrying a [`LexState`] so that block
//! comments, multi-line strings and Markdown fences survive line breaks.
//! The renderer only asks for the lines it is about to draw, so opening a
//! huge file stays fast.

use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Plain,
    Rust,
    Go,
    C,
    Cpp,
    Java,
    Kotlin,
    Swift,
    Zig,
    Dart,
    Python,
    JavaScript,
    TypeScript,
    Ruby,
    Php,
    Lua,
    Haskell,
    Elixir,
    Sql,
    Shell,
    Markdown,
    Latex,
    Rst,
    AsciiDoc,
    Json,
    Toml,
    Yaml,
    Ini,
    Hcl,
    Nix,
    Lisp,
    Cmake,
    Dockerfile,
    Makefile,
    Protobuf,
    Html,
    Css,
    Vue,
    Svelte,
    Diff,
    GitCommit,
}

impl Lang {
    pub fn name(self) -> &'static str {
        match self {
            Lang::Plain => "Plain",
            Lang::Rust => "Rust",
            Lang::Go => "Go",
            Lang::C => "C",
            Lang::Cpp => "C++",
            Lang::Java => "Java",
            Lang::Kotlin => "Kotlin",
            Lang::Swift => "Swift",
            Lang::Zig => "Zig",
            Lang::Dart => "Dart",
            Lang::Python => "Python",
            Lang::JavaScript => "JavaScript",
            Lang::TypeScript => "TypeScript",
            Lang::Ruby => "Ruby",
            Lang::Php => "PHP",
            Lang::Lua => "Lua",
            Lang::Haskell => "Haskell",
            Lang::Elixir => "Elixir",
            Lang::Sql => "SQL",
            Lang::Shell => "Shell",
            Lang::Markdown => "Markdown",
            Lang::Latex => "LaTeX",
            Lang::Rst => "reST",
            Lang::AsciiDoc => "AsciiDoc",
            Lang::Json => "JSON",
            Lang::Toml => "TOML",
            Lang::Yaml => "YAML",
            Lang::Ini => "INI",
            Lang::Hcl => "HCL",
            Lang::Nix => "Nix",
            Lang::Lisp => "Lisp",
            Lang::Cmake => "CMake",
            Lang::Dockerfile => "Dockerfile",
            Lang::Makefile => "Makefile",
            Lang::Protobuf => "Protobuf",
            Lang::Html => "HTML",
            Lang::Css => "CSS",
            Lang::Vue => "Vue",
            Lang::Svelte => "Svelte",
            Lang::Diff => "Diff",
            Lang::GitCommit => "GitCommit",
        }
    }

    /// Stable key used for per-language configuration sections.
    pub fn config_key(self) -> &'static str {
        match self {
            Lang::Plain => "plain",
            Lang::Rust => "rust",
            Lang::Go => "go",
            Lang::C => "c",
            Lang::Cpp => "cpp",
            Lang::Java => "java",
            Lang::Kotlin => "kotlin",
            Lang::Swift => "swift",
            Lang::Zig => "zig",
            Lang::Dart => "dart",
            Lang::Python => "python",
            Lang::JavaScript => "javascript",
            Lang::TypeScript => "typescript",
            Lang::Ruby => "ruby",
            Lang::Php => "php",
            Lang::Lua => "lua",
            Lang::Haskell => "haskell",
            Lang::Elixir => "elixir",
            Lang::Sql => "sql",
            Lang::Shell => "shell",
            Lang::Markdown => "markdown",
            Lang::Latex => "latex",
            Lang::Rst => "rst",
            Lang::AsciiDoc => "asciidoc",
            Lang::Json => "json",
            Lang::Toml => "toml",
            Lang::Yaml => "yaml",
            Lang::Ini => "ini",
            Lang::Hcl => "hcl",
            Lang::Nix => "nix",
            Lang::Lisp => "lisp",
            Lang::Cmake => "cmake",
            Lang::Dockerfile => "dockerfile",
            Lang::Makefile => "makefile",
            Lang::Protobuf => "protobuf",
            Lang::Html => "html",
            Lang::Css => "css",
            Lang::Vue => "vue",
            Lang::Svelte => "svelte",
            Lang::Diff => "diff",
            Lang::GitCommit => "gitcommit",
        }
    }

    /// Common aliases accepted as configuration section names.
    pub fn config_aliases(self) -> &'static [&'static str] {
        match self {
            Lang::Plain => &["plain", "plaintext", "text"],
            Lang::Cpp => &["cpp", "c++"],
            Lang::JavaScript => &["javascript", "js"],
            Lang::TypeScript => &["typescript", "ts"],
            Lang::Markdown => &["markdown", "md"],
            Lang::Latex => &["latex", "tex"],
            Lang::Rst => &["rst", "rest", "restructuredtext"],
            Lang::AsciiDoc => &["asciidoc", "adoc"],
            Lang::Shell => &["shell", "sh"],
            Lang::GitCommit => &["gitcommit", "git"],
            Lang::Hcl => &["hcl", "terraform"],
            Lang::Lisp => &["lisp", "elisp", "scheme", "clojure", "racket"],
            Lang::Protobuf => &["protobuf", "proto"],
            _ => &[],
        }
    }

    /// Default line-comment prefix for `M-;` and similar features.
    pub fn default_comment_start(self) -> &'static str {
        match self {
            Lang::Plain
            | Lang::Markdown
            | Lang::Json
            | Lang::Html
            | Lang::Css
            | Lang::Vue
            | Lang::Svelte
            | Lang::Diff => "",
            Lang::Rust
            | Lang::Go
            | Lang::C
            | Lang::Cpp
            | Lang::Java
            | Lang::Kotlin
            | Lang::Swift
            | Lang::Zig
            | Lang::Dart
            | Lang::JavaScript
            | Lang::TypeScript
            | Lang::Php
            | Lang::AsciiDoc
            | Lang::Protobuf => "//",
            Lang::Python
            | Lang::Ruby
            | Lang::Elixir
            | Lang::Shell
            | Lang::Toml
            | Lang::Yaml
            | Lang::Hcl
            | Lang::Nix
            | Lang::Cmake
            | Lang::Dockerfile
            | Lang::Makefile => "#",
            Lang::Lua | Lang::Haskell | Lang::Sql => "--",
            Lang::Lisp => ";",
            Lang::Latex => "%",
            Lang::Rst => "..",
            Lang::Ini => ";",
            Lang::GitCommit => "#",
        }
    }

    /// Default LSP `languageId` for servers that expect one.
    pub fn lsp_language_id(self) -> &'static str {
        match self {
            Lang::Plain | Lang::GitCommit => "plaintext",
            Lang::Rust => "rust",
            Lang::Go => "go",
            Lang::C => "c",
            Lang::Cpp => "cpp",
            Lang::Java => "java",
            Lang::Kotlin => "kotlin",
            Lang::Swift => "swift",
            Lang::Zig => "zig",
            Lang::Dart => "dart",
            Lang::Python => "python",
            Lang::JavaScript => "javascript",
            Lang::TypeScript => "typescript",
            Lang::Ruby => "ruby",
            Lang::Php => "php",
            Lang::Lua => "lua",
            Lang::Haskell => "haskell",
            Lang::Elixir => "elixir",
            Lang::Sql => "sql",
            Lang::Shell => "shellscript",
            Lang::Markdown => "markdown",
            Lang::Latex => "latex",
            Lang::Rst => "restructuredtext",
            Lang::AsciiDoc => "asciidoc",
            Lang::Json => "json",
            Lang::Toml => "toml",
            Lang::Yaml => "yaml",
            Lang::Ini => "ini",
            Lang::Hcl => "hcl",
            Lang::Nix => "nix",
            Lang::Lisp => "lisp",
            Lang::Cmake => "cmake",
            Lang::Dockerfile => "dockerfile",
            Lang::Makefile => "makefile",
            Lang::Protobuf => "proto",
            Lang::Html => "html",
            Lang::Css => "css",
            Lang::Vue => "vue",
            Lang::Svelte => "svelte",
            Lang::Diff => "diff",
        }
    }

    /// Programming-language modes where completion at point is useful.
    pub fn is_code(self) -> bool {
        matches!(
            self,
            Lang::Rust
                | Lang::Go
                | Lang::C
                | Lang::Cpp
                | Lang::Java
                | Lang::Kotlin
                | Lang::Swift
                | Lang::Zig
                | Lang::Dart
                | Lang::Python
                | Lang::JavaScript
                | Lang::TypeScript
                | Lang::Ruby
                | Lang::Php
                | Lang::Lua
                | Lang::Haskell
                | Lang::Elixir
                | Lang::Nix
                | Lang::Lisp
                | Lang::Sql
                | Lang::Shell
        )
    }

    pub fn is_lisp(self) -> bool {
        matches!(self, Lang::Lisp)
    }

    /// Modes where prose-oriented features such as spell checking make sense.
    pub fn is_prose(self) -> bool {
        matches!(
            self,
            Lang::Plain
                | Lang::Markdown
                | Lang::Latex
                | Lang::Rst
                | Lang::AsciiDoc
                | Lang::GitCommit
        )
    }

    pub fn detect(path: Option<&Path>, first_line: Option<&str>) -> Lang {
        if let Some(path) = path {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_default();
            match name.as_str() {
                "commit_editmsg" | "merge_msg" | "tag_editmsg" | "squash_msg" => {
                    return Lang::GitCommit
                }
                "makefile" | "gnumakefile" | "bsdmakefile" => return Lang::Makefile,
                "cmakelists.txt" => return Lang::Cmake,
                "dockerfile" | "containerfile" => return Lang::Dockerfile,
                "gemfile" | "rakefile" | "vagrantfile" | "brewfile" | "podfile" | "fastfile"
                | "thorfile" | "berksfile" | "guardfile" | "capfile" => return Lang::Ruby,
                ".gitconfig" | ".editorconfig" => return Lang::Ini,
                ".bashrc" | ".bash_profile" | ".bash_login" | ".bash_aliases" | ".zshrc"
                | ".zprofile" | ".zshenv" | ".profile" => return Lang::Shell,
                _ => {}
            }
            if name == ".env" || name.starts_with(".env.") {
                return Lang::Ini;
            }
            if name.ends_with(".mk") {
                return Lang::Makefile;
            }
            if name.starts_with("dockerfile.") || name.ends_with(".dockerfile") {
                return Lang::Dockerfile;
            }
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase())
                .unwrap_or_default();
            return match ext.as_str() {
                "rs" => Lang::Rust,
                "go" => Lang::Go,
                "c" | "h" => Lang::C,
                "cc" | "cpp" | "cxx" | "hpp" | "hh" | "hxx" | "ino" => Lang::Cpp,
                "java" => Lang::Java,
                "kt" | "kts" => Lang::Kotlin,
                "swift" => Lang::Swift,
                "zig" => Lang::Zig,
                "dart" => Lang::Dart,
                "py" | "pyw" | "pyi" => Lang::Python,
                "js" | "mjs" | "cjs" | "jsx" => Lang::JavaScript,
                "ts" | "tsx" | "mts" | "cts" => Lang::TypeScript,
                "rb" | "rake" | "gemspec" | "ru" => Lang::Ruby,
                "php" | "phtml" | "php3" | "php4" | "php5" | "phps" => Lang::Php,
                "lua" => Lang::Lua,
                "hs" | "lhs" | "hsc" => Lang::Haskell,
                "ex" | "exs" | "eex" | "heex" => Lang::Elixir,
                "lisp" | "lsp" | "cl" | "el" | "scm" | "ss" | "sls" | "sld" | "rkt" | "clj"
                | "cljs" | "cljc" | "edn" => Lang::Lisp,
                "sql" | "psql" => Lang::Sql,
                "sh" | "bash" | "zsh" | "fish" | "ksh" => Lang::Shell,
                "md" | "markdown" | "mdx" => Lang::Markdown,
                "tex" | "sty" | "cls" | "ltx" => Lang::Latex,
                "rst" => Lang::Rst,
                "adoc" | "asciidoc" => Lang::AsciiDoc,
                "json" | "jsonc" => Lang::Json,
                "toml" => Lang::Toml,
                "yaml" | "yml" => Lang::Yaml,
                "ini" | "cfg" | "conf" | "properties" | "desktop" => Lang::Ini,
                "tf" | "tfvars" | "hcl" | "nomad" => Lang::Hcl,
                "nix" => Lang::Nix,
                "cmake" => Lang::Cmake,
                "proto" => Lang::Protobuf,
                "html" | "htm" | "xhtml" => Lang::Html,
                "css" | "scss" | "sass" | "less" => Lang::Css,
                "vue" => Lang::Vue,
                "svelte" => Lang::Svelte,
                "diff" | "patch" => Lang::Diff,
                _ => Lang::Plain,
            };
        }
        if let Some(line) = first_line {
            if let Some(rest) = line.strip_prefix("#!") {
                let cmd = rest.trim();
                if cmd.contains("python") {
                    return Lang::Python;
                }
                if cmd.contains("node") || cmd.contains("deno") || cmd.contains("bun") {
                    return Lang::JavaScript;
                }
                if cmd.contains("ruby") {
                    return Lang::Ruby;
                }
                if cmd.contains("php") {
                    return Lang::Php;
                }
                if cmd.contains("lua") {
                    return Lang::Lua;
                }
                if cmd.contains("elixir") {
                    return Lang::Elixir;
                }
                if cmd.contains("runhaskell") || cmd.contains("runghc") {
                    return Lang::Haskell;
                }
                if cmd.contains("dart") {
                    return Lang::Dart;
                }
                if cmd.contains("sbcl")
                    || cmd.contains("clisp")
                    || cmd.contains("scheme")
                    || cmd.contains("racket")
                    || cmd.contains("guile")
                    || cmd.contains("clojure")
                {
                    return Lang::Lisp;
                }
                if cmd.contains("nix-shell") || cmd.contains("nix") {
                    return Lang::Nix;
                }
                if cmd.contains("bash")
                    || cmd.contains("zsh")
                    || cmd.contains("sh")
                    || cmd.contains("fish")
                    || cmd.contains("ksh")
                {
                    return Lang::Shell;
                }
            }
        }
        Lang::Plain
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Tok {
    Keyword,
    Type,
    Constant,
    Function,
    String,
    Comment,
    Number,
    Operator,
    Preproc,
    Heading,
    Emphasis,
    Link,
    DiffAdd,
    DiffDel,
    DiffHunk,
    Meta,
}

pub const TOK_COUNT: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub kind: Tok,
}

impl Span {
    pub const fn new(start: usize, end: usize, kind: Tok) -> Self {
        Self { start, end, kind }
    }
}

/// State carried from the end of one line to the beginning of the next.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LexState {
    pub comment: u8,
    pub string: u8,
    pub triple: bool,
    /// Number of `#` in a Rust raw string when `string == b'r'`.
    pub raw: u8,
    pub fence: u8,
    pub fence_len: u8,
    pub html_comment: bool,
    pub seen_text: bool,
    /// FNV-1a hash of the heredoc terminator while `string == b'<'`.
    pub heredoc_hash: u64,
}

pub fn lex(lang: Lang, chars: &[char], state: LexState) -> (Vec<Span>, LexState) {
    match lang {
        Lang::Plain => (Vec::new(), LexState::default()),
        Lang::Rust => lex_code(chars, state, &RUST),
        Lang::Go => lex_code(chars, state, &GO),
        Lang::C => lex_code(chars, state, &C),
        Lang::Cpp => lex_code(chars, state, &CPP),
        Lang::Java => lex_code(chars, state, &JAVA),
        Lang::Kotlin => lex_code(chars, state, &KOTLIN),
        Lang::Swift => lex_code(chars, state, &SWIFT),
        Lang::Zig => lex_code(chars, state, &ZIG),
        Lang::Dart => lex_code(chars, state, &DART),
        Lang::Python => lex_code(chars, state, &PYTHON),
        Lang::JavaScript => lex_code(chars, state, &JS),
        Lang::TypeScript => lex_code(chars, state, &TS),
        Lang::Ruby => lex_code(chars, state, &RUBY),
        Lang::Php => lex_code(chars, state, &PHP),
        Lang::Lua => lex_code(chars, state, &LUA),
        Lang::Haskell => lex_code(chars, state, &HASKELL),
        Lang::Elixir => lex_code(chars, state, &ELIXIR),
        Lang::Sql => lex_code(chars, state, &SQL),
        Lang::Shell => lex_code(chars, state, &SHELL),
        Lang::Json => lex_structured(chars, state, Structured::Json),
        Lang::Toml => lex_structured(chars, state, Structured::Toml),
        Lang::Yaml => lex_structured(chars, state, Structured::Yaml),
        Lang::Markdown => lex_markdown(chars, state),
        Lang::Latex => lex_latex(chars, state),
        Lang::Rst => lex_rst(chars, state),
        Lang::AsciiDoc => lex_asciidoc(chars, state),
        Lang::Ini => lex_ini(chars, state),
        Lang::Hcl => lex_code(chars, state, &HCL),
        Lang::Nix => lex_code(chars, state, &NIX),
        Lang::Lisp => lex_code(chars, state, &LISP),
        Lang::Cmake => lex_code(chars, state, &CMAKE),
        Lang::Dockerfile => lex_dockerfile(chars, state),
        Lang::Makefile => lex_makefile(chars, state),
        Lang::Protobuf => lex_code(chars, state, &PROTOBUF),
        Lang::Html | Lang::Vue | Lang::Svelte => lex_html(chars, state),
        Lang::Css => lex_css(chars, state),
        Lang::Diff => (lex_diff(chars), state),
        Lang::GitCommit => lex_gitcommit(chars, state),
    }
}

// ---------------------------------------------------------------------------
// helpers

fn starts_with(chars: &[char], i: usize, pat: &str) -> bool {
    for (j, pc) in (i..).zip(pat.chars()) {
        if j >= chars.len() || chars[j] != pc {
            return false;
        }
    }
    true
}

fn find_seq(chars: &[char], from: usize, pat: &str) -> Option<usize> {
    let plen = pat.chars().count();
    let mut i = from;
    while i + plen <= chars.len() {
        if starts_with(chars, i, pat) {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn scan_ident(chars: &[char], start: usize) -> usize {
    let mut j = start;
    while j < chars.len() && is_ident_continue(chars[j]) {
        j += 1;
    }
    j
}

fn next_non_space(chars: &[char], from: usize) -> Option<char> {
    chars[from..].iter().copied().find(|c| !c.is_whitespace())
}

fn scan_number(chars: &[char], start: usize) -> usize {
    let n = chars.len();
    let mut j = start;
    if j + 1 < n && chars[j] == '0' && matches!(chars[j + 1], 'x' | 'X' | 'b' | 'B' | 'o' | 'O') {
        j += 2;
        while j < n && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
            j += 1;
        }
        return j;
    }
    while j < n && (chars[j].is_ascii_digit() || chars[j] == '_') {
        j += 1;
    }
    if j < n && chars[j] == '.' && j + 1 < n && chars[j + 1].is_ascii_digit() {
        j += 1;
        while j < n && (chars[j].is_ascii_digit() || chars[j] == '_') {
            j += 1;
        }
    }
    if j < n && matches!(chars[j], 'e' | 'E') {
        let mut k = j + 1;
        if k < n && matches!(chars[k], '+' | '-') {
            k += 1;
        }
        if k < n && chars[k].is_ascii_digit() {
            j = k;
            while j < n && chars[j].is_ascii_digit() {
                j += 1;
            }
        }
    }
    j
}

fn scan_string(
    chars: &[char],
    start: usize,
    quote: char,
    escapes: bool,
    triple: bool,
) -> (usize, bool) {
    let n = chars.len();
    let mut j = start;
    if triple {
        while j + 3 <= n {
            if chars[j] == quote && chars[j + 1] == quote && chars[j + 2] == quote {
                return (j + 3, true);
            }
            j += 1;
        }
        return (n, false);
    }
    while j < n {
        let c = chars[j];
        if escapes && c == '\\' {
            j += 2;
            continue;
        }
        if c == quote {
            return (j + 1, true);
        }
        j += 1;
    }
    (n, false)
}

fn scan_raw_string(chars: &[char], start: usize, hashes: u8) -> (usize, bool) {
    let n = chars.len();
    let mut j = start;
    while j < n {
        if chars[j] == '"' {
            let mut k = j + 1;
            let mut seen = 0;
            while k < n && chars[k] == '#' && seen < hashes {
                seen += 1;
                k += 1;
            }
            if seen == hashes {
                return (k, true);
            }
        }
        j += 1;
    }
    (n, false)
}

fn scan_block(
    chars: &[char],
    start: usize,
    open: &str,
    close: &str,
    nested: bool,
    depth0: u8,
) -> (usize, bool, u8) {
    let n = chars.len();
    let mut depth = depth0.max(1) as u32;
    let mut j = start;
    while j < n {
        if starts_with(chars, j, close) {
            depth -= 1;
            j += close.chars().count();
            if depth == 0 {
                return (j, true, 0);
            }
            continue;
        }
        if nested && starts_with(chars, j, open) {
            depth += 1;
            j += open.chars().count();
            continue;
        }
        j += 1;
    }
    (n, false, depth.min(255) as u8)
}

// ---------------------------------------------------------------------------
// generic code lexer

#[derive(Clone, Copy)]
struct Quote {
    ch: char,
    multiline: bool,
    escapes: bool,
}

const DOUBLE: Quote = Quote {
    ch: '"',
    multiline: false,
    escapes: true,
};
const SINGLE: Quote = Quote {
    ch: '\'',
    multiline: false,
    escapes: true,
};
const SHELL_SINGLE: Quote = Quote {
    ch: '\'',
    multiline: false,
    escapes: false,
};
const SQL_SINGLE: Quote = Quote {
    ch: '\'',
    multiline: false,
    escapes: false,
};
const BACKTICK: Quote = Quote {
    ch: '`',
    multiline: true,
    escapes: false,
};

#[derive(Clone, Copy)]
struct CodeSpec {
    line_comments: &'static [&'static str],
    block_comment: Option<(&'static str, &'static str, bool)>,
    quotes: &'static [Quote],
    char_literal: bool,
    triple_quotes: bool,
    keywords: &'static [&'static str],
    types: &'static [&'static str],
    constants: &'static [&'static str],
    preproc: bool,
    rust_raw: bool,
    python: bool,
    shell: bool,
    hash_needs_space: bool,
    /// Match keywords and constants without regard to case (SQL, PHP, CMake).
    case_insensitive: bool,
    /// Highlight any identifier that starts with an uppercase letter as a type.
    upper_types: bool,
    /// Give the name after `def`/`class`/`module` a distinct colour.
    def_pending: bool,
    /// Highlight `:symbol` / `:atom` as a constant.
    symbols: bool,
    /// Highlight `@name` as a preprocessor-ish token (Ruby ivars, Zig builtins).
    at_word: bool,
    /// Lua long brackets: `[[...]]`, `--[[...]]` and `[=[...]=]`.
    lua: bool,
    /// Nix indented strings: `''...''`.
    nix: bool,
    /// Zig multiline strings (`\\` at the start of a line).
    zig: bool,
    /// HCL/Dockerfile heredocs: `<<EOF` ... `EOF`.
    heredoc: bool,
}

impl CodeSpec {
    const DEFAULT: Self = Self {
        line_comments: &[],
        block_comment: None,
        quotes: &[],
        char_literal: false,
        triple_quotes: false,
        keywords: &[],
        types: &[],
        constants: &[],
        preproc: false,
        rust_raw: false,
        python: false,
        shell: false,
        hash_needs_space: false,
        case_insensitive: false,
        upper_types: false,
        def_pending: false,
        symbols: false,
        at_word: false,
        lua: false,
        nix: false,
        zig: false,
        heredoc: false,
    };
}

const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "union",
];
const RUST_TYPES: &[&str] = &[
    "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64",
    "i128", "isize", "f32", "f64", "String", "Vec", "Option", "Result", "Box", "Rc", "Arc",
    "HashMap", "HashSet", "BTreeMap", "BTreeSet", "Path", "PathBuf", "Cow", "Rc",
];
const RUST_CONSTANTS: &[&str] = &["None", "Some", "Ok", "Err", "true", "false"];

const GO_KEYWORDS: &[&str] = &[
    "break",
    "case",
    "chan",
    "const",
    "continue",
    "default",
    "defer",
    "else",
    "fallthrough",
    "for",
    "func",
    "go",
    "goto",
    "if",
    "import",
    "interface",
    "map",
    "package",
    "range",
    "return",
    "select",
    "struct",
    "switch",
    "type",
    "var",
];
const GO_TYPES: &[&str] = &[
    "bool",
    "byte",
    "complex64",
    "complex128",
    "error",
    "float32",
    "float64",
    "int",
    "int8",
    "int16",
    "int32",
    "int64",
    "rune",
    "string",
    "uint",
    "uint8",
    "uint16",
    "uint32",
    "uint64",
    "uintptr",
    "any",
];
const GO_CONSTANTS: &[&str] = &["true", "false", "iota", "nil"];

const C_KEYWORDS: &[&str] = &[
    "auto",
    "break",
    "case",
    "const",
    "continue",
    "default",
    "do",
    "else",
    "enum",
    "extern",
    "for",
    "goto",
    "if",
    "inline",
    "register",
    "restrict",
    "return",
    "sizeof",
    "static",
    "struct",
    "switch",
    "typedef",
    "union",
    "volatile",
    "while",
    "_Bool",
    "_Atomic",
    "_Static_assert",
    "_Thread_local",
    "alignas",
    "alignof",
    "bool",
    "constexpr",
    "nullptr",
    "static_assert",
    "thread_local",
    "class",
    "namespace",
    "template",
    "typename",
    "public",
    "private",
    "protected",
    "virtual",
    "override",
    "final",
    "new",
    "delete",
    "this",
    "using",
    "try",
    "catch",
    "throw",
    "decltype",
    "noexcept",
    "const_cast",
    "static_cast",
    "dynamic_cast",
    "reinterpret_cast",
    "concept",
    "requires",
    "co_await",
    "co_return",
    "co_yield",
    "export",
    "import",
    "module",
];
const C_TYPES: &[&str] = &[
    "char",
    "double",
    "float",
    "int",
    "long",
    "short",
    "signed",
    "unsigned",
    "void",
    "wchar_t",
    "size_t",
    "ssize_t",
    "int8_t",
    "int16_t",
    "int32_t",
    "int64_t",
    "uint8_t",
    "uint16_t",
    "uint32_t",
    "uint64_t",
    "intptr_t",
    "uintptr_t",
    "FILE",
    "auto",
    "string",
    "vector",
    "map",
    "set",
    "pair",
    "unique_ptr",
    "shared_ptr",
    "size_type",
    "std",
];
const C_CONSTANTS: &[&str] = &["NULL", "true", "false", "nullptr", "EOF"];

const PY_KEYWORDS: &[&str] = &[
    "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del", "elif",
    "else", "except", "finally", "for", "from", "global", "if", "import", "in", "is", "lambda",
    "nonlocal", "not", "or", "pass", "raise", "return", "try", "while", "with", "yield", "match",
    "case",
];
const PY_TYPES: &[&str] = &[
    "bool",
    "bytes",
    "bytearray",
    "complex",
    "dict",
    "float",
    "frozenset",
    "int",
    "list",
    "object",
    "set",
    "str",
    "tuple",
    "type",
    "Exception",
    "ValueError",
    "TypeError",
    "KeyError",
    "RuntimeError",
    "Optional",
    "List",
    "Dict",
    "Tuple",
    "Set",
    "Sequence",
    "Iterable",
    "Any",
];
const PY_CONSTANTS: &[&str] = &["True", "False", "None", "NotImplemented", "Ellipsis"];

const JS_KEYWORDS: &[&str] = &[
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "export",
    "extends",
    "finally",
    "for",
    "from",
    "function",
    "get",
    "if",
    "import",
    "in",
    "instanceof",
    "let",
    "new",
    "of",
    "return",
    "set",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
    "enum",
    "implements",
    "interface",
    "private",
    "protected",
    "public",
    "readonly",
    "declare",
    "namespace",
    "abstract",
    "as",
    "satisfies",
    "keyof",
    "infer",
    "is",
    "never",
    "unknown",
    "any",
    "type",
];
const JS_TYPES: &[&str] = &[
    "Array",
    "Boolean",
    "Date",
    "Error",
    "Function",
    "JSON",
    "Map",
    "Math",
    "Number",
    "Object",
    "Promise",
    "RegExp",
    "Set",
    "String",
    "Symbol",
    "WeakMap",
    "WeakSet",
    "BigInt",
    "Record",
    "Partial",
    "Readonly",
    "Pick",
    "Omit",
    "Exclude",
    "Extract",
    "NonNullable",
];
const JS_CONSTANTS: &[&str] = &[
    "true",
    "false",
    "null",
    "undefined",
    "NaN",
    "Infinity",
    "console",
    "window",
    "document",
    "globalThis",
    "process",
];

const SHELL_KEYWORDS: &[&str] = &[
    "if", "then", "else", "elif", "fi", "for", "while", "until", "do", "done", "case", "esac",
    "in", "function", "select", "time", "return", "break", "continue", "local", "export",
    "readonly", "declare", "typeset", "unset", "shift", "exit", "eval", "exec", "trap", "set",
];
const SHELL_CONSTANTS: &[&str] = &[
    "true", "false", "null", "PATH", "HOME", "PWD", "SHELL", "USER", "EDITOR",
];

const JAVA_KEYWORDS: &[&str] = &[
    "abstract",
    "assert",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "default",
    "do",
    "else",
    "enum",
    "extends",
    "final",
    "finally",
    "for",
    "goto",
    "if",
    "implements",
    "import",
    "instanceof",
    "interface",
    "module",
    "native",
    "new",
    "non-sealed",
    "open",
    "opens",
    "package",
    "permits",
    "private",
    "protected",
    "provides",
    "public",
    "record",
    "requires",
    "return",
    "sealed",
    "static",
    "strictfp",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "to",
    "transient",
    "transitive",
    "try",
    "uses",
    "var",
    "volatile",
    "while",
    "with",
    "yield",
];
const JAVA_TYPES: &[&str] = &[
    "boolean",
    "byte",
    "char",
    "double",
    "float",
    "int",
    "long",
    "short",
    "void",
    "String",
    "Object",
    "Integer",
    "Long",
    "Double",
    "Float",
    "Boolean",
    "Character",
    "Byte",
    "Short",
    "List",
    "ArrayList",
    "Map",
    "HashMap",
    "Set",
    "HashSet",
    "Optional",
    "Stream",
    "Exception",
    "RuntimeException",
    "Runnable",
    "Thread",
    "Math",
    "System",
];
const JAVA_CONSTANTS: &[&str] = &["true", "false", "null"];

const KOTLIN_KEYWORDS: &[&str] = &[
    "as",
    "break",
    "class",
    "continue",
    "do",
    "else",
    "false",
    "for",
    "fun",
    "if",
    "in",
    "interface",
    "is",
    "null",
    "object",
    "package",
    "return",
    "super",
    "this",
    "throw",
    "true",
    "try",
    "typealias",
    "typeof",
    "val",
    "var",
    "when",
    "while",
    "by",
    "catch",
    "constructor",
    "delegate",
    "dynamic",
    "field",
    "file",
    "finally",
    "get",
    "import",
    "init",
    "param",
    "property",
    "receiver",
    "set",
    "setparam",
    "where",
    "actual",
    "abstract",
    "annotation",
    "companion",
    "const",
    "crossinline",
    "data",
    "enum",
    "expect",
    "external",
    "final",
    "infix",
    "inline",
    "inner",
    "internal",
    "lateinit",
    "noinline",
    "open",
    "operator",
    "out",
    "override",
    "private",
    "protected",
    "public",
    "reified",
    "sealed",
    "suspend",
    "tailrec",
    "vararg",
    "value",
];
const KOTLIN_TYPES: &[&str] = &[
    "Any",
    "Unit",
    "Nothing",
    "Boolean",
    "Byte",
    "Short",
    "Int",
    "Long",
    "Float",
    "Double",
    "Char",
    "String",
    "Array",
    "List",
    "MutableList",
    "Map",
    "MutableMap",
    "Set",
    "MutableSet",
    "Pair",
    "Triple",
    "Sequence",
    "IntArray",
    "LongArray",
    "BooleanArray",
    "Result",
];
const KOTLIN_CONSTANTS: &[&str] = &["true", "false", "null"];

const SWIFT_KEYWORDS: &[&str] = &[
    "associatedtype",
    "class",
    "deinit",
    "enum",
    "extension",
    "fileprivate",
    "func",
    "import",
    "init",
    "inout",
    "internal",
    "let",
    "open",
    "operator",
    "private",
    "precedencegroup",
    "protocol",
    "public",
    "rethrows",
    "static",
    "struct",
    "subscript",
    "typealias",
    "var",
    "break",
    "case",
    "catch",
    "continue",
    "default",
    "defer",
    "do",
    "else",
    "fallthrough",
    "for",
    "guard",
    "if",
    "in",
    "repeat",
    "return",
    "throw",
    "switch",
    "where",
    "while",
    "as",
    "async",
    "await",
    "is",
    "super",
    "self",
    "Self",
    "throws",
    "try",
    "some",
    "any",
    "actor",
    "nonisolated",
    "isolated",
    "consume",
    "borrowing",
    "consuming",
    "macro",
    "package",
];
const SWIFT_TYPES: &[&str] = &[
    "Int",
    "Double",
    "Float",
    "String",
    "Bool",
    "Character",
    "Void",
    "Array",
    "Dictionary",
    "Set",
    "Optional",
    "Result",
    "Error",
    "Any",
    "AnyObject",
    "Data",
    "Date",
    "URL",
];
const SWIFT_CONSTANTS: &[&str] = &["true", "false", "nil"];

const ZIG_KEYWORDS: &[&str] = &[
    "addrspace",
    "align",
    "allowzero",
    "and",
    "anyframe",
    "anytype",
    "asm",
    "async",
    "await",
    "break",
    "callconv",
    "catch",
    "comptime",
    "const",
    "continue",
    "defer",
    "else",
    "enum",
    "errdefer",
    "error",
    "export",
    "extern",
    "fn",
    "for",
    "if",
    "inline",
    "linksection",
    "noalias",
    "noinline",
    "nosuspend",
    "opaque",
    "or",
    "orelse",
    "packed",
    "pub",
    "resume",
    "return",
    "struct",
    "suspend",
    "switch",
    "test",
    "threadlocal",
    "try",
    "union",
    "unreachable",
    "usingnamespace",
    "var",
    "volatile",
    "while",
];
const ZIG_TYPES: &[&str] = &[
    "i8",
    "i16",
    "i32",
    "i64",
    "i128",
    "u8",
    "u16",
    "u32",
    "u64",
    "u128",
    "isize",
    "usize",
    "f16",
    "f32",
    "f64",
    "f80",
    "f128",
    "bool",
    "void",
    "noreturn",
    "type",
    "anyerror",
    "comptime_int",
    "comptime_float",
    "c_int",
    "c_uint",
    "c_long",
    "c_ulong",
    "c_longlong",
    "c_ulonglong",
];
const ZIG_CONSTANTS: &[&str] = &["true", "false", "null", "undefined"];

const DART_KEYWORDS: &[&str] = &[
    "abstract",
    "as",
    "assert",
    "async",
    "await",
    "base",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "covariant",
    "default",
    "deferred",
    "do",
    "dynamic",
    "else",
    "enum",
    "export",
    "extends",
    "extension",
    "external",
    "factory",
    "final",
    "finally",
    "for",
    "function",
    "get",
    "hide",
    "if",
    "implements",
    "import",
    "in",
    "interface",
    "is",
    "late",
    "library",
    "mixin",
    "new",
    "on",
    "operator",
    "part",
    "required",
    "rethrow",
    "return",
    "sealed",
    "set",
    "show",
    "static",
    "super",
    "switch",
    "sync",
    "this",
    "throw",
    "try",
    "typedef",
    "var",
    "void",
    "when",
    "while",
    "with",
    "yield",
];
const DART_TYPES: &[&str] = &[
    "bool",
    "double",
    "int",
    "num",
    "String",
    "Object",
    "List",
    "Map",
    "Set",
    "Future",
    "Stream",
    "Iterable",
    "Iterator",
    "Duration",
    "DateTime",
    "Uri",
    "BigInt",
    "Exception",
    "Error",
    "Function",
];
const DART_CONSTANTS: &[&str] = &["true", "false", "null"];

const RUBY_KEYWORDS: &[&str] = &[
    "BEGIN",
    "END",
    "alias",
    "and",
    "begin",
    "break",
    "case",
    "class",
    "def",
    "defined?",
    "do",
    "else",
    "elsif",
    "end",
    "ensure",
    "for",
    "if",
    "in",
    "module",
    "next",
    "not",
    "or",
    "redo",
    "rescue",
    "retry",
    "return",
    "self",
    "super",
    "then",
    "undef",
    "unless",
    "until",
    "when",
    "while",
    "yield",
    "require",
    "require_relative",
    "include",
    "extend",
    "prepend",
    "attr_reader",
    "attr_writer",
    "attr_accessor",
    "private",
    "protected",
    "public",
    "raise",
    "lambda",
    "proc",
];
const RUBY_TYPES: &[&str] = &[
    "String",
    "Integer",
    "Float",
    "Array",
    "Hash",
    "Symbol",
    "Range",
    "Regexp",
    "Time",
    "Proc",
    "Lambda",
    "Object",
    "Class",
    "Module",
    "Struct",
    "Exception",
    "StandardError",
    "RuntimeError",
    "TrueClass",
    "FalseClass",
    "NilClass",
    "Numeric",
    "Comparable",
    "Enumerable",
    "IO",
    "File",
];
const RUBY_CONSTANTS: &[&str] = &[
    "true", "false", "nil", "__FILE__", "__LINE__", "__dir__", "ENV", "ARGV", "STDIN", "STDOUT",
    "STDERR",
];

const PHP_KEYWORDS: &[&str] = &[
    "abstract",
    "and",
    "array",
    "as",
    "break",
    "callable",
    "case",
    "catch",
    "class",
    "clone",
    "const",
    "continue",
    "declare",
    "default",
    "do",
    "echo",
    "else",
    "elseif",
    "empty",
    "enddeclare",
    "endfor",
    "endforeach",
    "endif",
    "endswitch",
    "endwhile",
    "enum",
    "eval",
    "exit",
    "extends",
    "final",
    "finally",
    "fn",
    "for",
    "foreach",
    "from",
    "function",
    "global",
    "goto",
    "if",
    "implements",
    "include",
    "include_once",
    "instanceof",
    "insteadof",
    "interface",
    "isset",
    "list",
    "match",
    "namespace",
    "new",
    "or",
    "print",
    "private",
    "protected",
    "public",
    "readonly",
    "require",
    "require_once",
    "return",
    "static",
    "switch",
    "throw",
    "trait",
    "try",
    "unset",
    "use",
    "var",
    "while",
    "xor",
    "yield",
];
const PHP_TYPES: &[&str] = &[
    "int", "float", "string", "bool", "array", "object", "callable", "iterable", "void", "mixed",
    "never", "null", "false", "true", "self", "parent", "integer", "boolean", "double",
];
const PHP_CONSTANTS: &[&str] = &[
    "true",
    "false",
    "null",
    "PHP_EOL",
    "PHP_INT_MAX",
    "PHP_INT_MIN",
    "PHP_VERSION",
    "DIRECTORY_SEPARATOR",
];

const LUA_KEYWORDS: &[&str] = &[
    "and", "break", "do", "else", "elseif", "end", "for", "function", "goto", "if", "in", "local",
    "not", "or", "repeat", "return", "then", "until", "while",
];
const LUA_TYPES: &[&str] = &[
    "string", "number", "boolean", "table", "function", "nil", "userdata", "thread", "any",
];
const LUA_CONSTANTS: &[&str] = &["true", "false", "nil", "_G", "_VERSION", "self"];

const HASKELL_KEYWORDS: &[&str] = &[
    "case",
    "class",
    "data",
    "default",
    "deriving",
    "do",
    "else",
    "foreign",
    "if",
    "import",
    "in",
    "infix",
    "infixl",
    "infixr",
    "instance",
    "let",
    "module",
    "newtype",
    "of",
    "then",
    "type",
    "where",
    "qualified",
    "as",
    "hiding",
    "forall",
    "mdo",
    "rec",
    "proc",
    "pattern",
    "static",
    "stock",
    "anyclass",
    "via",
    "family",
    "role",
];
const HASKELL_TYPES: &[&str] = &[
    "Int",
    "Integer",
    "Float",
    "Double",
    "Bool",
    "Char",
    "String",
    "Maybe",
    "Either",
    "IO",
    "Ordering",
    "Eq",
    "Ord",
    "Show",
    "Read",
    "Enum",
    "Bounded",
    "Num",
    "Real",
    "Integral",
    "Fractional",
    "Floating",
    "Functor",
    "Applicative",
    "Monad",
    "MonadIO",
    "Foldable",
    "Traversable",
    "Semigroup",
    "Monoid",
    "Text",
    "ByteString",
    "Word",
    "Word8",
    "Word16",
    "Word32",
    "Word64",
    "Int8",
    "Int16",
    "Int32",
    "Int64",
];
const HASKELL_CONSTANTS: &[&str] = &[
    "True", "False", "Nothing", "Just", "Left", "Right", "LT", "EQ", "GT",
];

const ELIXIR_KEYWORDS: &[&str] = &[
    "after",
    "alias",
    "and",
    "case",
    "catch",
    "cond",
    "def",
    "defp",
    "defmodule",
    "defmacro",
    "defmacrop",
    "defdelegate",
    "defguard",
    "defguardp",
    "defimpl",
    "defprotocol",
    "defstruct",
    "defexception",
    "do",
    "else",
    "end",
    "fn",
    "for",
    "if",
    "import",
    "in",
    "not",
    "or",
    "quote",
    "raise",
    "receive",
    "require",
    "rescue",
    "try",
    "unless",
    "unquote",
    "unquote_splicing",
    "use",
    "when",
    "with",
];
const ELIXIR_TYPES: &[&str] = &[
    "String",
    "Integer",
    "Float",
    "List",
    "Map",
    "Set",
    "Tuple",
    "Atom",
    "Function",
    "PID",
    "Port",
    "Reference",
    "Struct",
    "Enum",
    "Stream",
    "GenServer",
    "Supervisor",
    "Agent",
    "Task",
    "Keyword",
    "Date",
    "DateTime",
    "NaiveDateTime",
    "Time",
    "URI",
    "File",
    "IO",
];
const ELIXIR_CONSTANTS: &[&str] = &["true", "false", "nil"];

const SQL_KEYWORDS: &[&str] = &[
    "SELECT",
    "FROM",
    "WHERE",
    "INSERT",
    "INTO",
    "VALUES",
    "UPDATE",
    "SET",
    "DELETE",
    "CREATE",
    "TABLE",
    "INDEX",
    "VIEW",
    "DROP",
    "ALTER",
    "ADD",
    "COLUMN",
    "PRIMARY",
    "KEY",
    "FOREIGN",
    "REFERENCES",
    "UNIQUE",
    "NOT",
    "NULL",
    "AND",
    "OR",
    "AS",
    "ON",
    "JOIN",
    "INNER",
    "LEFT",
    "RIGHT",
    "FULL",
    "OUTER",
    "CROSS",
    "GROUP",
    "BY",
    "ORDER",
    "HAVING",
    "LIMIT",
    "OFFSET",
    "DISTINCT",
    "UNION",
    "ALL",
    "EXCEPT",
    "INTERSECT",
    "CASE",
    "WHEN",
    "THEN",
    "ELSE",
    "END",
    "IN",
    "IS",
    "LIKE",
    "ILIKE",
    "BETWEEN",
    "EXISTS",
    "WITH",
    "RECURSIVE",
    "ASC",
    "DESC",
    "NULLS",
    "FIRST",
    "LAST",
    "CONSTRAINT",
    "DEFAULT",
    "CHECK",
    "CASCADE",
    "RESTRICT",
    "BEGIN",
    "COMMIT",
    "ROLLBACK",
    "TRANSACTION",
    "GRANT",
    "REVOKE",
    "EXPLAIN",
    "ANALYZE",
    "VACUUM",
    "TRUNCATE",
    "RETURNING",
    "OVER",
    "PARTITION",
    "WINDOW",
    "USING",
    "NATURAL",
    "LATERAL",
    "MATERIALIZED",
    "TEMPORARY",
    "IF",
    "REPLACE",
    "DATABASE",
    "SCHEMA",
    "SEQUENCE",
    "TRIGGER",
    "FUNCTION",
    "PROCEDURE",
    "DECLARE",
    "CURSOR",
    "FETCH",
    "OPEN",
    "CLOSE",
    "FOR",
    "LOOP",
    "WHILE",
    "DO",
    "CALL",
    "MERGE",
    "MATCHED",
    "ROWS",
    "RANGE",
    "PRECEDING",
    "FOLLOWING",
    "CURRENT",
    "ROW",
    "UNBOUNDED",
    "FILTER",
    "WITHIN",
];
const SQL_TYPES: &[&str] = &[
    "INT",
    "INTEGER",
    "SMALLINT",
    "BIGINT",
    "DECIMAL",
    "NUMERIC",
    "REAL",
    "DOUBLE",
    "PRECISION",
    "FLOAT",
    "CHAR",
    "VARCHAR",
    "TEXT",
    "BOOLEAN",
    "BOOL",
    "DATE",
    "TIME",
    "TIMESTAMP",
    "INTERVAL",
    "JSON",
    "JSONB",
    "UUID",
    "BYTEA",
    "BLOB",
    "CLOB",
    "SERIAL",
    "BIGSERIAL",
    "MONEY",
    "XML",
    "ARRAY",
];
const SQL_CONSTANTS: &[&str] = &[
    "TRUE",
    "FALSE",
    "NULL",
    "CURRENT_DATE",
    "CURRENT_TIME",
    "CURRENT_TIMESTAMP",
    "NOW",
    "DEFAULT",
];

const CMAKE_KEYWORDS: &[&str] = &[
    "if",
    "elseif",
    "else",
    "endif",
    "foreach",
    "endforeach",
    "while",
    "endwhile",
    "function",
    "endfunction",
    "macro",
    "endmacro",
    "break",
    "continue",
    "return",
    "include",
    "set",
    "unset",
    "option",
    "message",
    "project",
    "cmake_minimum_required",
    "add_executable",
    "add_library",
    "add_subdirectory",
    "target_link_libraries",
    "target_include_directories",
    "target_compile_options",
    "target_compile_definitions",
    "target_sources",
    "install",
    "find_package",
    "find_library",
    "find_path",
    "configure_file",
    "file",
    "execute_process",
    "add_custom_command",
    "add_custom_target",
    "add_test",
    "enable_testing",
    "list",
    "string",
    "math",
    "get_filename_component",
    "get_property",
    "set_property",
    "set_target_properties",
    "mark_as_advanced",
    "include_directories",
    "link_directories",
    "link_libraries",
    "source_group",
    "set_directory_properties",
    "set_source_files_properties",
];
const CMAKE_CONSTANTS: &[&str] = &[
    "ON",
    "OFF",
    "TRUE",
    "FALSE",
    "YES",
    "NO",
    "Y",
    "N",
    "IGNORE",
    "NOTFOUND",
    "ALL",
    "COMMAND",
    "DEPENDS",
    "WORKING_DIRECTORY",
    "OUTPUT",
    "INPUT",
    "MAIN_DEPENDENCY",
    "BYPRODUCTS",
    "VERBATIM",
    "USES_TERMINAL",
];

const HCL_KEYWORDS: &[&str] = &[
    "resource",
    "data",
    "variable",
    "output",
    "module",
    "provider",
    "terraform",
    "locals",
    "dynamic",
    "content",
    "for",
    "in",
    "if",
    "else",
    "endif",
    "for_each",
    "count",
    "depends_on",
    "lifecycle",
    "create_before_destroy",
    "prevent_destroy",
    "ignore_changes",
    "replace_triggered_by",
    "precondition",
    "postcondition",
    "validation",
    "condition",
    "error_message",
    "sensitive",
    "nullable",
    "default",
    "type",
    "description",
    "source",
    "version",
    "required_providers",
    "required_version",
    "backend",
    "cloud",
    "workspace",
    "provisioner",
    "connection",
    "when",
    "on_failure",
    "timeouts",
    "create",
    "update",
    "delete",
];
const HCL_TYPES: &[&str] = &[
    "string", "number", "bool", "list", "map", "set", "object", "tuple", "any", "optional",
];
const HCL_CONSTANTS: &[&str] = &["true", "false", "null"];

const NIX_KEYWORDS: &[&str] = &[
    "assert", "else", "if", "in", "inherit", "let", "or", "rec", "then", "with",
];
const NIX_TYPES: &[&str] = &[
    "builtins",
    "derivation",
    "lib",
    "pkgs",
    "stdenv",
    "fetchurl",
    "fetchFromGitHub",
    "mkDerivation",
    "attrs",
    "list",
    "path",
    "package",
    "function",
];
const NIX_CONSTANTS: &[&str] = &["true", "false", "null"];

const PROTOBUF_KEYWORDS: &[&str] = &[
    "syntax",
    "package",
    "import",
    "option",
    "message",
    "enum",
    "service",
    "rpc",
    "returns",
    "stream",
    "repeated",
    "optional",
    "required",
    "oneof",
    "map",
    "reserved",
    "extend",
    "extensions",
    "group",
    "weak",
    "public",
    "edition",
];
const PROTOBUF_TYPES: &[&str] = &[
    "double", "float", "int32", "int64", "uint32", "uint64", "sint32", "sint64", "fixed32",
    "fixed64", "sfixed32", "sfixed64", "bool", "string", "bytes",
];
const PROTOBUF_CONSTANTS: &[&str] = &["true", "false"];

const RUST: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", true)),
    quotes: &[DOUBLE],
    char_literal: true,
    keywords: RUST_KEYWORDS,
    types: RUST_TYPES,
    constants: RUST_CONSTANTS,
    rust_raw: true,
    ..CodeSpec::DEFAULT
};
const GO: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", false)),
    quotes: &[DOUBLE, BACKTICK],
    char_literal: true,
    keywords: GO_KEYWORDS,
    types: GO_TYPES,
    constants: GO_CONSTANTS,
    ..CodeSpec::DEFAULT
};
const C: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", false)),
    quotes: &[DOUBLE],
    char_literal: true,
    keywords: C_KEYWORDS,
    types: C_TYPES,
    constants: C_CONSTANTS,
    preproc: true,
    ..CodeSpec::DEFAULT
};
const CPP: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", false)),
    quotes: &[DOUBLE],
    char_literal: true,
    keywords: C_KEYWORDS,
    types: C_TYPES,
    constants: C_CONSTANTS,
    preproc: true,
    ..CodeSpec::DEFAULT
};
const JAVA: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", false)),
    quotes: &[DOUBLE],
    char_literal: true,
    triple_quotes: true,
    keywords: JAVA_KEYWORDS,
    types: JAVA_TYPES,
    constants: JAVA_CONSTANTS,
    ..CodeSpec::DEFAULT
};
const KOTLIN: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", true)),
    quotes: &[DOUBLE],
    char_literal: true,
    triple_quotes: true,
    keywords: KOTLIN_KEYWORDS,
    types: KOTLIN_TYPES,
    constants: KOTLIN_CONSTANTS,
    ..CodeSpec::DEFAULT
};
const SWIFT: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", true)),
    quotes: &[DOUBLE],
    triple_quotes: true,
    keywords: SWIFT_KEYWORDS,
    types: SWIFT_TYPES,
    constants: SWIFT_CONSTANTS,
    ..CodeSpec::DEFAULT
};
const ZIG: CodeSpec = CodeSpec {
    line_comments: &["//"],
    quotes: &[DOUBLE],
    char_literal: true,
    keywords: ZIG_KEYWORDS,
    types: ZIG_TYPES,
    constants: ZIG_CONSTANTS,
    at_word: true,
    zig: true,
    ..CodeSpec::DEFAULT
};
const DART: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", true)),
    quotes: &[DOUBLE, SINGLE],
    triple_quotes: true,
    keywords: DART_KEYWORDS,
    types: DART_TYPES,
    constants: DART_CONSTANTS,
    shell: true,
    ..CodeSpec::DEFAULT
};
const PYTHON: CodeSpec = CodeSpec {
    line_comments: &["#"],
    quotes: &[DOUBLE, SINGLE],
    triple_quotes: true,
    keywords: PY_KEYWORDS,
    types: PY_TYPES,
    constants: PY_CONSTANTS,
    python: true,
    ..CodeSpec::DEFAULT
};
const JS: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", false)),
    quotes: &[DOUBLE, SINGLE, BACKTICK],
    keywords: JS_KEYWORDS,
    types: JS_TYPES,
    constants: JS_CONSTANTS,
    ..CodeSpec::DEFAULT
};
const TS: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", false)),
    quotes: &[DOUBLE, SINGLE, BACKTICK],
    keywords: JS_KEYWORDS,
    types: JS_TYPES,
    constants: JS_CONSTANTS,
    ..CodeSpec::DEFAULT
};
const RUBY: CodeSpec = CodeSpec {
    line_comments: &["#"],
    block_comment: Some(("=begin", "=end", false)),
    quotes: &[DOUBLE, SINGLE, BACKTICK],
    keywords: RUBY_KEYWORDS,
    types: RUBY_TYPES,
    constants: RUBY_CONSTANTS,
    def_pending: true,
    symbols: true,
    at_word: true,
    ..CodeSpec::DEFAULT
};
const PHP: CodeSpec = CodeSpec {
    line_comments: &["//", "#"],
    block_comment: Some(("/*", "*/", false)),
    quotes: &[DOUBLE, SINGLE],
    keywords: PHP_KEYWORDS,
    types: PHP_TYPES,
    constants: PHP_CONSTANTS,
    shell: true,
    hash_needs_space: true,
    case_insensitive: true,
    heredoc: true,
    ..CodeSpec::DEFAULT
};
const LUA: CodeSpec = CodeSpec {
    line_comments: &["--"],
    quotes: &[DOUBLE, SINGLE],
    keywords: LUA_KEYWORDS,
    types: LUA_TYPES,
    constants: LUA_CONSTANTS,
    lua: true,
    ..CodeSpec::DEFAULT
};
const HASKELL: CodeSpec = CodeSpec {
    line_comments: &["--"],
    block_comment: Some(("{-", "-}", true)),
    quotes: &[DOUBLE],
    char_literal: true,
    keywords: HASKELL_KEYWORDS,
    types: HASKELL_TYPES,
    constants: HASKELL_CONSTANTS,
    upper_types: true,
    ..CodeSpec::DEFAULT
};
const ELIXIR: CodeSpec = CodeSpec {
    line_comments: &["#"],
    quotes: &[DOUBLE, SINGLE],
    triple_quotes: true,
    keywords: ELIXIR_KEYWORDS,
    types: ELIXIR_TYPES,
    constants: ELIXIR_CONSTANTS,
    upper_types: true,
    def_pending: true,
    symbols: true,
    ..CodeSpec::DEFAULT
};
const SQL: CodeSpec = CodeSpec {
    line_comments: &["--"],
    block_comment: Some(("/*", "*/", false)),
    quotes: &[SQL_SINGLE, DOUBLE],
    keywords: SQL_KEYWORDS,
    types: SQL_TYPES,
    constants: SQL_CONSTANTS,
    case_insensitive: true,
    ..CodeSpec::DEFAULT
};
const SHELL: CodeSpec = CodeSpec {
    line_comments: &["#"],
    quotes: &[DOUBLE, SHELL_SINGLE],
    keywords: SHELL_KEYWORDS,
    constants: SHELL_CONSTANTS,
    shell: true,
    hash_needs_space: true,
    ..CodeSpec::DEFAULT
};
const CMAKE: CodeSpec = CodeSpec {
    line_comments: &["#"],
    block_comment: Some(("#[[", "]]", false)),
    quotes: &[DOUBLE],
    keywords: CMAKE_KEYWORDS,
    constants: CMAKE_CONSTANTS,
    shell: true,
    case_insensitive: true,
    ..CodeSpec::DEFAULT
};
const HCL: CodeSpec = CodeSpec {
    line_comments: &["#", "//"],
    block_comment: Some(("/*", "*/", false)),
    quotes: &[DOUBLE],
    keywords: HCL_KEYWORDS,
    types: HCL_TYPES,
    constants: HCL_CONSTANTS,
    shell: true,
    heredoc: true,
    ..CodeSpec::DEFAULT
};
const NIX: CodeSpec = CodeSpec {
    line_comments: &["#"],
    block_comment: Some(("/*", "*/", false)),
    quotes: &[DOUBLE],
    keywords: NIX_KEYWORDS,
    types: NIX_TYPES,
    constants: NIX_CONSTANTS,
    shell: true,
    nix: true,
    ..CodeSpec::DEFAULT
};
const LISP_KEYWORDS: &[&str] = &[
    "defun",
    "defmacro",
    "defvar",
    "defparameter",
    "defconstant",
    "defclass",
    "defmethod",
    "defgeneric",
    "defpackage",
    "in-package",
    "lambda",
    "let",
    "let*",
    "letrec",
    "if",
    "cond",
    "when",
    "unless",
    "progn",
    "prog1",
    "prog2",
    "setq",
    "setf",
    "quote",
    "function",
    "loop",
    "do",
    "dolist",
    "dotimes",
    "case",
    "typecase",
    "handler-case",
    "unwind-protect",
    "and",
    "or",
    "not",
    "car",
    "cdr",
    "cons",
    "list",
    "apply",
    "funcall",
    "mapcar",
    "mapc",
    "format",
    "princ",
    "print",
    "require",
    "provide",
    "eval-when",
    "declare",
    "declaim",
    "fn",
    "defn",
    "def",
    "ns",
];
const LISP_CONSTANTS: &[&str] = &["nil", "t", "true", "false"];

const LISP: CodeSpec = CodeSpec {
    line_comments: &[";"],
    block_comment: Some(("#|", "|#", true)),
    quotes: &[DOUBLE],
    keywords: LISP_KEYWORDS,
    constants: LISP_CONSTANTS,
    symbols: true,
    ..CodeSpec::DEFAULT
};

const PROTOBUF: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", false)),
    quotes: &[DOUBLE],
    keywords: PROTOBUF_KEYWORDS,
    types: PROTOBUF_TYPES,
    constants: PROTOBUF_CONSTANTS,
    ..CodeSpec::DEFAULT
};

fn lex_code(chars: &[char], mut state: LexState, spec: &CodeSpec) -> (Vec<Span>, LexState) {
    let n = chars.len();
    let mut spans: Vec<Span> = Vec::new();
    let mut i = 0usize;

    // Resume a heredoc body.
    if state.string == b'<' {
        if heredoc_terminator(chars, state) {
            spans.push(Span::new(0, n, Tok::Meta));
            state.string = 0;
            state.raw = 0;
            state.heredoc_hash = 0;
        } else {
            spans.push(Span::new(0, n, Tok::String));
        }
        return (spans, state);
    }

    // Resume a Lua long-bracket string.
    if spec.lua && state.string == b'[' {
        if let Some(end) = find_long_close(chars, 0, state.raw) {
            spans.push(Span::new(0, end, Tok::String));
            state.string = 0;
            state.raw = 0;
            i = end;
        } else {
            spans.push(Span::new(0, n, Tok::String));
            return (spans, state);
        }
    }

    // Resume a Nix indented string.
    if spec.nix && state.string == b'\'' {
        let (end, closed) = scan_nix_indented(chars, 0);
        spans.push(Span::new(0, end, Tok::String));
        if !closed {
            return (spans, state);
        }
        state.string = 0;
        i = end;
    }

    // Resume a block comment from the previous line.
    if state.comment > 0 {
        if spec.lua {
            let level = state.comment - 1;
            if let Some(end) = find_long_close(chars, 0, level) {
                spans.push(Span::new(0, end, Tok::Comment));
                state.comment = 0;
                i = end;
            } else {
                spans.push(Span::new(0, n, Tok::Comment));
                return (spans, state);
            }
        } else if let Some((open, close, nested)) = spec.block_comment {
            let (end, closed, depth) = scan_block(chars, 0, open, close, nested, state.comment);
            spans.push(Span::new(0, end, Tok::Comment));
            if !closed {
                state.comment = depth;
                return (spans, state);
            }
            state.comment = 0;
            i = end;
        } else {
            state.comment = 0;
        }
    }

    // Resume a multi-line string.
    if state.string == b'r' {
        let (end, closed) = scan_raw_string(chars, 0, state.raw);
        spans.push(Span::new(0, end, Tok::String));
        if !closed {
            return (spans, state);
        }
        state.string = 0;
        state.raw = 0;
        i = end;
    } else if state.string != 0 {
        let quote = state.string as char;
        let (end, closed) = scan_string(chars, 0, quote, true, state.triple);
        spans.push(Span::new(0, end, Tok::String));
        if !closed {
            return (spans, state);
        }
        state.string = 0;
        state.triple = false;
        i = end;
    }

    let mut pending: Option<Tok> = None;

    while i < n {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Lua long comments start with `--[[`.
        if spec.lua && starts_with(chars, i, "--") {
            if let Some((level, content)) = long_bracket_open(chars, i + 2) {
                if let Some(end) = find_long_close(chars, content, level) {
                    spans.push(Span::new(i, end, Tok::Comment));
                    i = end;
                    continue;
                }
                spans.push(Span::new(i, n, Tok::Comment));
                state.comment = level + 1;
                return (spans, state);
            }
        }

        // block comment start (checked before line comments so `#[[` and
        // `--[[` style openers win over their shorter line-comment prefixes)
        if let Some((open, close, nested)) = spec.block_comment {
            let line_start = || chars[..i].iter().all(|ch| ch.is_whitespace());
            if starts_with(chars, i, open) && (!open.starts_with('=') || line_start()) {
                let (end, closed, depth) =
                    scan_block(chars, i + open.chars().count(), open, close, nested, 1);
                spans.push(Span::new(i, end, Tok::Comment));
                if !closed {
                    state.comment = depth;
                    return (spans, state);
                }
                i = end;
                continue;
            }
        }

        // line comments
        let mut line_comment = false;
        for lc in spec.line_comments {
            if starts_with(chars, i, lc) {
                if spec.hash_needs_space && *lc == "#" {
                    let preceded_by_space = i == 0 || chars[i - 1].is_whitespace();
                    let attribute = i + 1 < n && chars[i + 1] == '[';
                    if !preceded_by_space {
                        continue;
                    }
                    if attribute {
                        // PHP 8 attributes (`#[...]`) are not comments.
                        spans.push(Span::new(i, n, Tok::Preproc));
                        return (spans, state);
                    }
                }
                spans.push(Span::new(i, n, Tok::Comment));
                line_comment = true;
                break;
            }
        }
        if line_comment {
            break;
        }

        // C preprocessor
        if spec.preproc && c == '#' && chars[..i].iter().all(|ch| ch.is_whitespace()) {
            spans.push(Span::new(i, n, Tok::Preproc));
            break;
        }

        // Rust attributes
        if spec.rust_raw && c == '#' && i + 1 < n && matches!(chars[i + 1], '[' | '!') {
            spans.push(Span::new(i, n, Tok::Preproc));
            break;
        }

        // Lua long-bracket strings: [[...]], [=[...]=]
        if spec.lua && c == '[' {
            if let Some((level, content)) = long_bracket_open(chars, i) {
                if let Some(end) = find_long_close(chars, content, level) {
                    spans.push(Span::new(i, end, Tok::String));
                    i = end;
                    continue;
                }
                spans.push(Span::new(i, n, Tok::String));
                state.string = b'[';
                state.raw = level;
                return (spans, state);
            }
        }

        // Nix indented strings: ''...''
        if spec.nix && starts_with(chars, i, "''") {
            let (end, closed) = scan_nix_indented(chars, i + 2);
            spans.push(Span::new(i, end, Tok::String));
            if !closed {
                state.string = b'\'';
                return (spans, state);
            }
            i = end;
            continue;
        }

        // Zig multiline strings are made of `\\`-prefixed lines.
        if spec.zig && starts_with(chars, i, "\\\\") {
            spans.push(Span::new(i, n, Tok::String));
            return (spans, state);
        }

        // Rust raw strings: r"..." / r#"..."# / br#"..."#
        if spec.rust_raw && (c == 'r' || (c == 'b' && i + 1 < n && chars[i + 1] == 'r')) {
            let r = if c == 'b' { i + 1 } else { i };
            let mut j = r + 1;
            while j < n && chars[j] == '#' {
                j += 1;
            }
            if j < n && chars[j] == '"' {
                let hashes = (j - r - 1) as u8;
                let (end, closed) = scan_raw_string(chars, j + 1, hashes);
                spans.push(Span::new(i, end, Tok::String));
                if !closed {
                    state.string = b'r';
                    state.raw = hashes;
                    return (spans, state);
                }
                i = end;
                continue;
            }
        }

        // char literals (C/Rust/Go/Zig/Haskell)
        if spec.char_literal && c == '\'' {
            if let Some(end) = scan_char_literal(chars, i) {
                spans.push(Span::new(i, end, Tok::String));
                i = end;
                continue;
            }
        }

        // strings
        if let Some(q) = spec.quotes.iter().find(|q| q.ch == c) {
            let triple = spec.triple_quotes
                && (c == '"' || c == '\'')
                && i + 2 < n
                && chars[i + 1] == c
                && chars[i + 2] == c;
            let multiline = triple || q.multiline;
            let escapes = q.escapes;
            let start = if triple { i + 3 } else { i + 1 };
            let (end, closed) = scan_string(chars, start, c, escapes, triple);
            spans.push(Span::new(i, end, Tok::String));
            if !closed && multiline {
                state.string = c as u8;
                state.triple = triple;
                return (spans, state);
            }
            i = end;
            continue;
        }

        // HCL / Dockerfile heredocs: <<EOF ... EOF and <<-EOF ... EOF.
        if spec.heredoc {
            if let Some((hash, indented)) = start_heredoc(chars, i) {
                spans.push(Span::new(i, n, Tok::String));
                state.string = b'<';
                state.raw = indented as u8;
                state.heredoc_hash = hash;
                return (spans, state);
            }
        }

        // Python decorators
        if spec.python && c == '@' {
            let mut j = i + 1;
            while j < n && (is_ident_continue(chars[j]) || chars[j] == '.') {
                j += 1;
            }
            if j > i + 1 {
                spans.push(Span::new(i, j, Tok::Function));
                i = j;
                continue;
            }
        }

        // Ruby instance variables / Zig builtins: @name
        if spec.at_word && c == '@' {
            let mut j = i + 1;
            if j < n && chars[j] == '@' {
                j += 1;
            }
            let start = j;
            j = scan_ident(chars, j);
            if j > start {
                spans.push(Span::new(i, j, Tok::Preproc));
                i = j;
                continue;
            }
        }

        // Ruby symbols / Elixir atoms: :name
        if spec.symbols && c == ':' && i + 1 < n && is_ident_start(chars[i + 1]) {
            let end = scan_ident(chars, i + 1);
            spans.push(Span::new(i, end, Tok::Constant));
            i = end;
            continue;
        }

        // shell variables
        if spec.shell && c == '$' {
            if let Some(end) = scan_dollar_var(chars, i) {
                spans.push(Span::new(i, end, Tok::Preproc));
                i = end;
                continue;
            }
        }

        // numbers
        if c.is_ascii_digit() {
            let end = scan_number(chars, i);
            spans.push(Span::new(i, end, Tok::Number));
            i = end;
            continue;
        }

        // identifiers / keywords
        if is_ident_start(c) {
            let end = scan_ident(chars, i);
            let word: String = chars[i..end].iter().copied().collect();
            let kind = if let Some(k) = pending.take() {
                if k == Tok::Function && word == "self" && next_non_space(chars, end) == Some('.') {
                    pending = Some(k);
                    Some(Tok::Keyword)
                } else {
                    Some(k)
                }
            } else if word_in(spec.keywords, &word, spec.case_insensitive) {
                if spec.python {
                    if word == "def" {
                        pending = Some(Tok::Function);
                    } else if word == "class" {
                        pending = Some(Tok::Type);
                    }
                } else if spec.def_pending {
                    match word.as_str() {
                        "def" | "defp" | "defmacro" | "defmacrop" | "defguard" | "defguardp"
                        | "defdelegate" => pending = Some(Tok::Function),
                        "defmodule" | "defprotocol" | "defimpl" | "class" | "module" => {
                            pending = Some(Tok::Type)
                        }
                        _ => {}
                    }
                }
                Some(Tok::Keyword)
            } else if word_in(spec.types, &word, spec.case_insensitive) {
                Some(Tok::Type)
            } else if word_in(spec.constants, &word, spec.case_insensitive) {
                Some(Tok::Constant)
            } else if spec.upper_types
                && word
                    .chars()
                    .next()
                    .is_some_and(|first| first.is_uppercase())
            {
                Some(Tok::Type)
            } else if next_non_space(chars, end) == Some('(') {
                Some(Tok::Function)
            } else {
                None
            };
            if let Some(kind) = kind {
                spans.push(Span::new(i, end, kind));
            }
            i = end;
            continue;
        }

        // operators (grouped)
        if is_operator(c) {
            let mut j = i + 1;
            while j < n && is_operator(chars[j]) {
                j += 1;
            }
            spans.push(Span::new(i, j, Tok::Operator));
            i = j;
            continue;
        }

        i += 1;
    }

    (spans, state)
}

fn word_in(list: &[&str], word: &str, case_insensitive: bool) -> bool {
    if case_insensitive {
        list.iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(word))
    } else {
        list.contains(&word)
    }
}

fn long_bracket_open(chars: &[char], start: usize) -> Option<(u8, usize)> {
    if start >= chars.len() || chars[start] != '[' {
        return None;
    }
    let mut j = start + 1;
    while j < chars.len() && chars[j] == '=' {
        j += 1;
    }
    if j >= chars.len() || chars[j] != '[' {
        return None;
    }
    let level = (j - start - 1).min(254) as u8;
    Some((level, j + 1))
}

fn find_long_close(chars: &[char], from: usize, level: u8) -> Option<usize> {
    let mut i = from;
    while i < chars.len() {
        if chars[i] == ']' {
            let mut j = i + 1;
            let mut seen = 0u8;
            while j < chars.len() && chars[j] == '=' && seen < level {
                seen += 1;
                j += 1;
            }
            if seen == level && j < chars.len() && chars[j] == ']' {
                return Some(j + 1);
            }
        }
        i += 1;
    }
    None
}

fn scan_nix_indented(chars: &[char], start: usize) -> (usize, bool) {
    let n = chars.len();
    let mut j = start;
    while j < n {
        if starts_with(chars, j, "''${") {
            j += 4;
            continue;
        }
        if starts_with(chars, j, "'''") {
            j += 3;
            continue;
        }
        if starts_with(chars, j, "''") {
            return (j + 2, true);
        }
        j += 1;
    }
    (n, false)
}

fn heredoc_hash(chars: &[char]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for &c in chars {
        for byte in (c as u32).to_le_bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
    }
    hash
}

/// Parse `<<EOF` / `<<-EOF`, returning the terminator hash and whether the
/// closing marker may be indented.
/// Parse `<<EOF` / `<<-EOF` / `<<<EOF` (PHP heredoc), returning the
/// terminator hash and whether the closing marker may be indented.
fn start_heredoc(chars: &[char], start: usize) -> Option<(u64, bool)> {
    if !starts_with(chars, start, "<<") {
        return None;
    }
    let n = chars.len();
    let mut j = start + 2;
    let mut indented = false;
    if j < n && chars[j] == '<' {
        // PHP heredoc / nowdoc.
        indented = true;
        j += 1;
    }
    if j < n && chars[j] == '-' {
        indented = true;
        j += 1;
    }
    let quote = if j < n && matches!(chars[j], '"' | '\'') {
        let quote = chars[j];
        j += 1;
        Some(quote)
    } else {
        None
    };
    let ident_start = j;
    j = scan_ident(chars, j);
    if j == ident_start {
        return None;
    }
    if let Some(quote) = quote {
        if j >= n || chars[j] != quote {
            return None;
        }
    }
    Some((heredoc_hash(&chars[ident_start..j]), indented))
}

fn heredoc_terminator(chars: &[char], state: LexState) -> bool {
    let n = chars.len();
    let mut start = 0;
    if state.raw != 0 {
        while start < n && chars[start].is_whitespace() {
            start += 1;
        }
    }
    let end = scan_ident(chars, start);
    end > start
        && (end == n || chars[end].is_whitespace())
        && heredoc_hash(&chars[start..end]) == state.heredoc_hash
}

fn scan_char_literal(chars: &[char], start: usize) -> Option<usize> {
    let n = chars.len();
    let mut j = start + 1;
    if j >= n {
        return None;
    }
    if chars[j] == '\\' {
        j += 2;
    } else {
        j += 1;
    }
    if j < n && chars[j] == '\'' {
        Some(j + 1)
    } else {
        None
    }
}

fn scan_dollar_var(chars: &[char], start: usize) -> Option<usize> {
    let n = chars.len();
    let mut j = start + 1;
    if j < n && chars[j] == '{' {
        if let Some(close) = find_seq(chars, j + 1, "}") {
            j = close + 1;
        }
    } else if j < n && chars[j] == '(' {
        if let Some(close) = find_seq(chars, j + 1, ")") {
            j = close + 1;
        }
    } else {
        while j < n && is_ident_continue(chars[j]) {
            j += 1;
        }
    }
    if j > start + 1 {
        Some(j)
    } else {
        None
    }
}

fn is_operator(c: char) -> bool {
    matches!(
        c,
        '+' | '-'
            | '*'
            | '/'
            | '%'
            | '='
            | '<'
            | '>'
            | '!'
            | '&'
            | '|'
            | '^'
            | '~'
            | '?'
            | ':'
            | '.'
    )
}

// ---------------------------------------------------------------------------
// Markdown

fn lex_markdown(chars: &[char], mut state: LexState) -> (Vec<Span>, LexState) {
    let n = chars.len();
    let text: String = chars.iter().copied().collect();
    let trimmed = text.trim_start();
    let indent = text.chars().count() - trimmed.chars().count();
    let mut spans = Vec::new();

    if state.fence != 0 {
        let fence_char = state.fence as char;
        let run = trimmed.chars().take_while(|c| *c == fence_char).count();
        if run >= state.fence_len as usize {
            spans.push(Span::new(indent, n, Tok::Meta));
            state.fence = 0;
            state.fence_len = 0;
        } else {
            spans.push(Span::new(0, n, Tok::String));
        }
        return (spans, state);
    }

    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        let fence_char = trimmed.chars().next().unwrap_or('`');
        let run = trimmed.chars().take_while(|c| *c == fence_char).count();
        state.fence = fence_char as u8;
        state.fence_len = run.min(255) as u8;
        spans.push(Span::new(indent, n, Tok::Meta));
        return (spans, state);
    }

    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    if (1..=6).contains(&hashes)
        && (trimmed.chars().nth(hashes).is_none() || trimmed.chars().nth(hashes) == Some(' '))
    {
        spans.push(Span::new(0, n, Tok::Heading));
        return (spans, state);
    }

    if trimmed.starts_with('>') || trimmed == "---" || trimmed == "***" || trimmed == "___" {
        spans.push(Span::new(indent, n, Tok::Meta));
        return (spans, state);
    }

    // list bullets
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        spans.push(Span::new(indent, indent + 1, Tok::Meta));
    } else {
        let digits = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
        if digits > 0 && trimmed.chars().nth(digits) == Some('.') {
            spans.push(Span::new(indent, indent + digits + 1, Tok::Meta));
        }
    }

    // inline markup
    let mut i = 0;
    while i < n {
        match chars[i] {
            '`' => {
                let run = chars[i..].iter().take_while(|c| **c == '`').count();
                let marker = "`".repeat(run);
                if let Some(close) = find_seq(chars, i + run, &marker) {
                    spans.push(Span::new(i, close + run, Tok::String));
                    i = close + run;
                } else {
                    i += run;
                }
            }
            '*' | '_' => {
                let c = chars[i];
                let double = i + 1 < n && chars[i + 1] == c;
                let marker: String = if double {
                    c.to_string().repeat(2)
                } else {
                    c.to_string()
                };
                let from = i + marker.chars().count();
                if let Some(close) = find_seq(chars, from, &marker) {
                    if close > from {
                        spans.push(Span::new(i, close + marker.chars().count(), Tok::Emphasis));
                        i = close + marker.chars().count();
                        continue;
                    }
                }
                i += 1;
            }
            '[' => {
                if let Some(bracket) = find_seq(chars, i + 1, "]") {
                    if bracket + 1 < n && chars[bracket + 1] == '(' {
                        if let Some(paren) = find_seq(chars, bracket + 2, ")") {
                            spans.push(Span::new(i, paren + 1, Tok::Link));
                            i = paren + 1;
                            continue;
                        }
                    }
                }
                i += 1;
            }
            _ => i += 1,
        }
    }

    state.seen_text = true;
    (spans, state)
}

// ---------------------------------------------------------------------------
// Git commit messages and diffs

fn lex_gitcommit(chars: &[char], mut state: LexState) -> (Vec<Span>, LexState) {
    let n = chars.len();
    let text: String = chars.iter().copied().collect();
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return (Vec::new(), state);
    }
    if trimmed.starts_with('#') {
        let kind = if text.contains(">8") {
            Tok::Meta
        } else {
            Tok::Comment
        };
        return (vec![Span::new(0, n, kind)], state);
    }

    let mut spans = Vec::new();
    if !state.seen_text {
        spans.push(Span::new(0, n, Tok::Heading));
    } else if let Some((byte_colon, _)) = text.char_indices().find(|(_, c)| *c == ':') {
        // Highlight trailers such as "Signed-off-by:".
        let head = &text[..byte_colon];
        if head.chars().count() < 40
            && head
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            let colon = head.chars().count();
            spans.push(Span::new(0, colon + 1, Tok::Meta));
        }
    }
    state.seen_text = true;
    (spans, state)
}

fn lex_diff(chars: &[char]) -> Vec<Span> {
    let n = chars.len();
    if n == 0 {
        return Vec::new();
    }
    let text: String = chars.iter().copied().collect();
    let kind = if text.starts_with("+++") || text.starts_with("---") {
        Some(Tok::Meta)
    } else if text.starts_with("@@") {
        Some(Tok::DiffHunk)
    } else if text.starts_with('+') {
        Some(Tok::DiffAdd)
    } else if text.starts_with('-') {
        Some(Tok::DiffDel)
    } else if text.starts_with("diff ")
        || text.starts_with("index ")
        || text.starts_with("new file")
        || text.starts_with("deleted file")
        || text.starts_with("similarity index")
        || text.starts_with("rename ")
        || text.starts_with("Binary files")
    {
        Some(Tok::Meta)
    } else if text.starts_with('\\') {
        Some(Tok::Comment)
    } else {
        None
    };
    kind.map(|k| vec![Span::new(0, n, k)]).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// JSON / TOML / YAML

#[derive(Clone, Copy)]
enum Structured {
    Json,
    Toml,
    Yaml,
}

fn lex_structured(chars: &[char], state: LexState, kind: Structured) -> (Vec<Span>, LexState) {
    let n = chars.len();
    let mut spans = Vec::new();
    let text: String = chars.iter().copied().collect();
    let trimmed_start = text.chars().count() - text.trim_start().chars().count();

    if matches!(kind, Structured::Yaml) {
        if let Some((byte_hash, _)) = text.char_indices().find(|(_, c)| *c == '#') {
            if byte_hash == 0
                || text[..byte_hash]
                    .chars()
                    .last()
                    .map_or(true, char::is_whitespace)
            {
                let hash = text[..byte_hash].chars().count();
                spans.push(Span::new(hash, n, Tok::Comment));
            }
        }
    }

    let comment_at = spans.first().map(|s| s.start).unwrap_or(n);
    let mut i = 0;

    // TOML table header
    if matches!(kind, Structured::Toml) {
        let t = text.trim_start();
        if t.starts_with('[') {
            if let Some(close) = find_seq(chars, trimmed_start + 1, "]") {
                spans.push(Span::new(trimmed_start, close + 1, Tok::Heading));
                i = close + 1;
            }
        }
    }
    while i < comment_at {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '"' || (c == '\'' && !matches!(kind, Structured::Json)) {
            let (end, _closed) = scan_string(chars, i + 1, c, true, false);
            let end = end.min(comment_at);
            let is_key = next_non_space(chars, end) == Some(':');
            spans.push(Span::new(
                i,
                end,
                if is_key && !matches!(kind, Structured::Toml) {
                    Tok::Type
                } else {
                    Tok::String
                },
            ));
            i = end;
            continue;
        }
        if matches!(c, '{' | '}' | '[' | ']' | ',' | ':') {
            spans.push(Span::new(i, i + 1, Tok::Operator));
            i += 1;
            continue;
        }
        if matches!(c, '-' | '?') && i + 1 < n && chars[i + 1] == ' ' {
            spans.push(Span::new(i, i + 1, Tok::Operator));
            i += 1;
            continue;
        }
        if c == '&' || c == '*' || c == '!' {
            let end = scan_ident(chars, i + 1);
            spans.push(Span::new(i, end, Tok::Preproc));
            i = end;
            continue;
        }
        if c.is_ascii_digit() {
            let end = scan_number(chars, i).min(comment_at);
            spans.push(Span::new(i, end, Tok::Number));
            i = end;
            continue;
        }
        if is_ident_start(c) {
            let end = scan_ident(chars, i);
            let word: String = chars[i..end].iter().copied().collect();
            let next = next_non_space(chars, end);
            let tok = if matches!(
                word.as_str(),
                "true" | "false" | "null" | "True" | "False" | "None" | "yes" | "no" | "on" | "off"
            ) {
                Some(Tok::Constant)
            } else if next == Some(':') || next == Some('=') {
                Some(Tok::Type)
            } else {
                None
            };
            if let Some(tok) = tok {
                spans.push(Span::new(i, end, tok));
            }
            i = end;
            continue;
        }
        i += 1;
    }

    (spans, state)
}

// ---------------------------------------------------------------------------
// Makefile

fn lex_makefile(chars: &[char], state: LexState) -> (Vec<Span>, LexState) {
    let n = chars.len();
    let mut spans = Vec::new();
    let mut i = 0;

    // Comment (a '#' starts a comment unless it is inside a variable reference).
    while i < n {
        if chars[i] == '#' {
            spans.push(Span::new(i, n, Tok::Comment));
            break;
        }
        if chars[i] == '$' && i + 1 < n && matches!(chars[i + 1], '(' | '{') {
            let open = chars[i + 1];
            let close = if open == '(' { ")" } else { "}" };
            if let Some(end) = find_seq(chars, i + 1, close) {
                spans.push(Span::new(i, end + 1, Tok::Preproc));
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }
    let limit = spans
        .last()
        .filter(|s| s.kind == Tok::Comment)
        .map(|s| s.start)
        .unwrap_or(n);

    if chars.first() == Some(&'\t') {
        return (spans, state);
    }

    // Variable assignment?
    if let Some(eq) = find_char(chars, '=', limit) {
        let lhs: String = chars[..eq].iter().copied().collect();
        let lhs_trim = lhs.trim();
        if !lhs_trim.is_empty()
            && lhs_trim
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '-')
        {
            spans.push(Span::new(0, lhs.len(), Tok::Constant));
            spans.push(Span::new(eq, eq + 1, Tok::Operator));
            return (spans, state);
        }
    }

    // Target line: "name: deps"
    if let Some(colon) = find_char(chars, ':', limit) {
        let target = text_of(chars, 0, colon);
        let trimmed = target.trim();
        if !trimmed.is_empty() && !trimmed.contains('=') {
            let start = target.chars().count() - target.trim_start().chars().count();
            spans.push(Span::new(
                start,
                start + trimmed.chars().count(),
                Tok::Function,
            ));
            spans.push(Span::new(colon, colon + 1, Tok::Operator));
        }
    }

    spans.sort_by_key(|s| s.start);
    (spans, state)
}

fn find_char(chars: &[char], needle: char, limit: usize) -> Option<usize> {
    chars.iter().take(limit).position(|c| *c == needle)
}

fn text_of(chars: &[char], start: usize, end: usize) -> String {
    let end = end.min(chars.len());
    if start >= end {
        return String::new();
    }
    chars[start..end].iter().copied().collect()
}

// ---------------------------------------------------------------------------
// HTML / CSS

fn lex_html(chars: &[char], mut state: LexState) -> (Vec<Span>, LexState) {
    let n = chars.len();
    let mut spans = Vec::new();
    let mut i = 0;

    if state.html_comment {
        if let Some(end) = find_seq(chars, 0, "-->") {
            spans.push(Span::new(0, end + 3, Tok::Comment));
            state.html_comment = false;
            i = end + 3;
        } else {
            spans.push(Span::new(0, n, Tok::Comment));
            return (spans, state);
        }
    }

    while i < n {
        if starts_with(chars, i, "<!--") {
            if let Some(end) = find_seq(chars, i + 4, "-->") {
                spans.push(Span::new(i, end + 3, Tok::Comment));
                i = end + 3;
            } else {
                spans.push(Span::new(i, n, Tok::Comment));
                state.html_comment = true;
                return (spans, state);
            }
            continue;
        }
        if chars[i] == '<' {
            let end = find_seq(chars, i + 1, ">").map(|e| e + 1).unwrap_or(n);
            lex_tag(chars, i, end, &mut spans);
            i = end;
            continue;
        }
        if chars[i] == '&' {
            if let Some(end) = find_seq(chars, i + 1, ";") {
                if end - i <= 12 {
                    spans.push(Span::new(i, end + 1, Tok::Constant));
                    i = end + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
    (spans, state)
}

fn lex_tag(chars: &[char], start: usize, end: usize, spans: &mut Vec<Span>) {
    let mut i = start;
    if i < end && chars[i] == '<' {
        i += 1;
        if i < end && chars[i] == '/' {
            i += 1;
        }
        let name_start = i;
        while i < end && (is_ident_continue(chars[i]) || chars[i] == '-' || chars[i] == ':') {
            i += 1;
        }
        if i > name_start {
            spans.push(Span::new(name_start, i, Tok::Type));
        }
    }
    while i < end {
        let c = chars[i];
        if c == '"' || c == '\'' {
            let (s_end, _closed) = scan_string(chars, i + 1, c, true, false);
            let s_end = s_end.min(end);
            spans.push(Span::new(i, s_end, Tok::String));
            i = s_end;
            continue;
        }
        if is_ident_start(c) {
            let attr_end = scan_ident(chars, i);
            spans.push(Span::new(i, attr_end, Tok::Keyword));
            i = attr_end;
            continue;
        }
        i += 1;
    }
}

fn lex_css(chars: &[char], mut state: LexState) -> (Vec<Span>, LexState) {
    let n = chars.len();
    let mut spans = Vec::new();
    let mut i = 0;

    if state.comment > 0 {
        if let Some(end) = find_seq(chars, 0, "*/") {
            spans.push(Span::new(0, end + 2, Tok::Comment));
            state.comment = 0;
            i = end + 2;
        } else {
            spans.push(Span::new(0, n, Tok::Comment));
            return (spans, state);
        }
    }

    while i < n {
        if starts_with(chars, i, "/*") {
            if let Some(end) = find_seq(chars, i + 2, "*/") {
                spans.push(Span::new(i, end + 2, Tok::Comment));
                i = end + 2;
            } else {
                spans.push(Span::new(i, n, Tok::Comment));
                state.comment = 1;
                return (spans, state);
            }
            continue;
        }
        let c = chars[i];
        if c == '"' || c == '\'' {
            let (end, _closed) = scan_string(chars, i + 1, c, true, false);
            spans.push(Span::new(i, end, Tok::String));
            i = end;
            continue;
        }
        if c == '@' {
            let end = scan_ident(chars, i + 1);
            spans.push(Span::new(i, end, Tok::Preproc));
            i = end;
            continue;
        }
        if is_ident_start(c) || c == '-' || c == '.' || c == '#' {
            let mut j = i + 1;
            while j < n && (is_ident_continue(chars[j]) || matches!(chars[j], '-' | '_' | '.')) {
                j += 1;
            }
            let next = next_non_space(chars, j);
            let kind = if next == Some(':') {
                Tok::Keyword
            } else if next == Some('(') {
                Tok::Function
            } else {
                Tok::Type
            };
            spans.push(Span::new(i, j, kind));
            i = j;
            continue;
        }
        if c.is_ascii_digit() {
            let end = scan_number(chars, i);
            spans.push(Span::new(i, end, Tok::Number));
            i = end;
            continue;
        }
        i += 1;
    }
    (spans, state)
}

// ---------------------------------------------------------------------------
// Dockerfile

const DOCKERFILE_INSTRUCTIONS: &[&str] = &[
    "FROM",
    "RUN",
    "CMD",
    "LABEL",
    "MAINTAINER",
    "EXPOSE",
    "ENV",
    "ADD",
    "COPY",
    "ENTRYPOINT",
    "VOLUME",
    "USER",
    "WORKDIR",
    "ARG",
    "ONBUILD",
    "STOPSIGNAL",
    "HEALTHCHECK",
    "SHELL",
];

fn lex_dockerfile(chars: &[char], mut state: LexState) -> (Vec<Span>, LexState) {
    let n = chars.len();
    let mut spans = Vec::new();

    if state.string == b'<' {
        if heredoc_terminator(chars, state) {
            spans.push(Span::new(0, n, Tok::Meta));
            state.string = 0;
            state.raw = 0;
            state.heredoc_hash = 0;
        } else {
            spans.push(Span::new(0, n, Tok::String));
        }
        return (spans, state);
    }

    let continuation = state.seen_text;
    state.seen_text = false;

    let mut start = 0;
    while start < n && chars[start].is_whitespace() {
        start += 1;
    }
    if start == n {
        return (spans, state);
    }
    if chars[start] == '#' {
        spans.push(Span::new(start, n, Tok::Comment));
        state.seen_text = continuation;
        return (spans, state);
    }

    let mut i = start;
    if !continuation {
        let end = scan_ident(chars, start);
        if end > start {
            let word: String = chars[start..end].iter().copied().collect();
            if DOCKERFILE_INSTRUCTIONS
                .iter()
                .any(|instruction| instruction.eq_ignore_ascii_case(&word))
            {
                spans.push(Span::new(start, end, Tok::Keyword));
            }
            i = end;
        }
    }

    while i < n {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '#' {
            i += 1;
            continue;
        }
        if c == '"' || c == '\'' {
            let (end, _closed) = scan_string(chars, i + 1, c, true, false);
            spans.push(Span::new(i, end, Tok::String));
            i = end;
            continue;
        }
        if c == '$' {
            if let Some(end) = scan_dollar_var(chars, i) {
                spans.push(Span::new(i, end, Tok::Preproc));
                i = end;
                continue;
            }
        }
        if starts_with(chars, i, "<<") {
            if let Some((hash, indented)) = start_heredoc(chars, i) {
                spans.push(Span::new(i, n, Tok::String));
                state.string = b'<';
                state.raw = indented as u8;
                state.heredoc_hash = hash;
                return (spans, state);
            }
        }
        if starts_with(chars, i, "--") {
            let mut j = i + 2;
            while j < n && !chars[j].is_whitespace() && chars[j] != '=' {
                j += 1;
            }
            spans.push(Span::new(i, j, Tok::Preproc));
            i = j;
            continue;
        }
        if c == '=' {
            spans.push(Span::new(i, i + 1, Tok::Operator));
            i += 1;
            continue;
        }
        if c.is_ascii_digit() {
            let end = scan_number(chars, i);
            spans.push(Span::new(i, end, Tok::Number));
            i = end;
            continue;
        }
        if is_ident_start(c) {
            let end = scan_ident(chars, i);
            let word: String = chars[i..end].iter().copied().collect();
            if word.eq_ignore_ascii_case("as") || word.eq_ignore_ascii_case("onbuild") {
                spans.push(Span::new(i, end, Tok::Keyword));
            }
            i = end;
            continue;
        }
        i += 1;
    }

    let mut last = n;
    while last > 0 && chars[last - 1].is_whitespace() {
        last -= 1;
    }
    if last > 0 && chars[last - 1] == '\\' {
        state.seen_text = true;
    }
    (spans, state)
}

// ---------------------------------------------------------------------------
// INI / .env

fn lex_ini(chars: &[char], state: LexState) -> (Vec<Span>, LexState) {
    let n = chars.len();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < n && chars[i].is_whitespace() {
        i += 1;
    }
    if i == n {
        return (spans, state);
    }
    if chars[i] == ';' || chars[i] == '#' {
        spans.push(Span::new(i, n, Tok::Comment));
        return (spans, state);
    }
    if starts_with(chars, i, "export ") {
        spans.push(Span::new(i, i + 6, Tok::Keyword));
        i += 7;
        while i < n && chars[i].is_whitespace() {
            i += 1;
        }
    }

    if i < n && chars[i] == '[' {
        if let Some(close) = find_seq(chars, i + 1, "]") {
            spans.push(Span::new(i, close + 1, Tok::Heading));
            i = close + 1;
        }
    } else if i < n {
        let key_start = i;
        let mut delimiter = i;
        while delimiter < n && chars[delimiter] != '=' && chars[delimiter] != ':' {
            delimiter += 1;
        }
        if delimiter < n {
            let mut key_end = delimiter;
            while key_end > key_start && chars[key_end - 1].is_whitespace() {
                key_end -= 1;
            }
            if key_end > key_start {
                spans.push(Span::new(key_start, key_end, Tok::Type));
            }
            spans.push(Span::new(delimiter, delimiter + 1, Tok::Operator));
            i = delimiter + 1;
        }
    }

    while i < n {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if (c == ';' || c == '#') && (i == 0 || chars[i - 1].is_whitespace()) {
            spans.push(Span::new(i, n, Tok::Comment));
            break;
        }
        if c == '"' || c == '\'' {
            let (end, _closed) = scan_string(chars, i + 1, c, true, false);
            spans.push(Span::new(i, end, Tok::String));
            i = end;
            continue;
        }
        if c == '$' {
            if let Some(end) = scan_dollar_var(chars, i) {
                spans.push(Span::new(i, end, Tok::Preproc));
                i = end;
                continue;
            }
        }
        if c.is_ascii_digit() {
            let end = scan_number(chars, i);
            spans.push(Span::new(i, end, Tok::Number));
            i = end;
            continue;
        }
        if is_ident_start(c) {
            let end = scan_ident(chars, i);
            let word: String = chars[i..end].iter().copied().collect();
            if ["true", "false", "yes", "no", "on", "off", "null", "none"]
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(&word))
            {
                spans.push(Span::new(i, end, Tok::Constant));
            }
            i = end;
            continue;
        }
        if matches!(c, '=' | ':' | ',' | '\\') {
            spans.push(Span::new(i, i + 1, Tok::Operator));
        }
        i += 1;
    }
    (spans, state)
}

// ---------------------------------------------------------------------------
// LaTeX

fn is_escaped(chars: &[char], index: usize) -> bool {
    let mut backslashes = 0;
    let mut i = index;
    while i > 0 && chars[i - 1] == '\\' {
        backslashes += 1;
        i -= 1;
    }
    backslashes % 2 == 1
}

fn is_latex_section(name: &str) -> bool {
    matches!(
        name,
        "part"
            | "chapter"
            | "section"
            | "subsection"
            | "subsubsection"
            | "paragraph"
            | "subparagraph"
            | "title"
            | "author"
            | "date"
            | "maketitle"
    )
}

fn is_latex_verbatim(env: &str) -> bool {
    matches!(
        env,
        "verbatim" | "Verbatim" | "lstlisting" | "minted" | "alltt" | "comment"
    )
}

fn lex_latex(chars: &[char], mut state: LexState) -> (Vec<Span>, LexState) {
    let n = chars.len();
    let mut spans = Vec::new();
    let mut i = 0;

    // Verbatim-like environments continue until \end{...}.
    if state.string == b'v' {
        if let Some(start) = find_seq(chars, 0, "\\end{") {
            let mut end = start + 5;
            if let Some(close) = find_seq(chars, end, "}") {
                end = close + 1;
            }
            spans.push(Span::new(0, end, Tok::String));
            state.string = 0;
            i = end;
        } else {
            spans.push(Span::new(0, n, Tok::String));
            return (spans, state);
        }
    }

    // Display math carried over from the previous line.
    if state.comment > 0 {
        let close = match state.comment {
            1 => "$$",
            2 => "\\]",
            _ => "\\)",
        };
        if let Some(end) = find_seq(chars, 0, close) {
            let end = end + close.chars().count();
            spans.push(Span::new(0, end, Tok::String));
            state.comment = 0;
            i = end;
        } else {
            spans.push(Span::new(0, n, Tok::String));
            return (spans, state);
        }
    }

    while i < n {
        let c = chars[i];
        if c == '%' && !is_escaped(chars, i) {
            spans.push(Span::new(i, n, Tok::Comment));
            break;
        }
        if c == '\\' {
            if starts_with(chars, i, "\\\\") {
                spans.push(Span::new(i, i + 2, Tok::Operator));
                i += 2;
                continue;
            }
            if starts_with(chars, i, "\\[") {
                if let Some(end) = find_seq(chars, i + 2, "\\]") {
                    spans.push(Span::new(i, end + 2, Tok::String));
                    i = end + 2;
                } else {
                    spans.push(Span::new(i, n, Tok::String));
                    state.comment = 2;
                    return (spans, state);
                }
                continue;
            }
            if starts_with(chars, i, "\\(") {
                if let Some(end) = find_seq(chars, i + 2, "\\)") {
                    spans.push(Span::new(i, end + 2, Tok::String));
                    i = end + 2;
                } else {
                    spans.push(Span::new(i, n, Tok::String));
                    state.comment = 3;
                    return (spans, state);
                }
                continue;
            }
            if i + 1 < n && chars[i + 1].is_alphabetic() {
                let mut j = i + 1;
                while j < n && chars[j].is_alphabetic() {
                    j += 1;
                }
                let name: String = chars[i + 1..j].iter().copied().collect();
                let kind = if is_latex_section(&name) {
                    Tok::Heading
                } else {
                    Tok::Function
                };
                spans.push(Span::new(i, j, kind));

                if (name == "begin" || name == "end") && j < n && chars[j] == '{' {
                    if let Some(close) = find_seq(chars, j + 1, "}") {
                        let env: String = chars[j + 1..close].iter().copied().collect();
                        spans.push(Span::new(j, j + 1, Tok::Operator));
                        spans.push(Span::new(j + 1, close, Tok::Type));
                        spans.push(Span::new(close, close + 1, Tok::Operator));
                        i = close + 1;
                        if name == "begin" && is_latex_verbatim(&env) {
                            let end_tag = format!("\\end{{{env}}}");
                            let tag_len = end_tag.chars().count();
                            if let Some(end) = find_seq(chars, close + 1, &end_tag) {
                                spans.push(Span::new(close + 1, end + tag_len, Tok::String));
                                i = end + tag_len;
                                continue;
                            }
                            if close + 1 < n {
                                spans.push(Span::new(close + 1, n, Tok::String));
                            }
                            state.string = b'v';
                            return (spans, state);
                        }
                        continue;
                    }
                }

                if name == "verb" && j < n {
                    let delimiter = chars[j];
                    let marker: String = delimiter.to_string();
                    if let Some(end) = find_seq(chars, j + 1, &marker) {
                        spans.push(Span::new(j, end + 1, Tok::String));
                        i = end + 1;
                        continue;
                    }
                }
                i = j;
                continue;
            }
            if i + 1 < n {
                spans.push(Span::new(i, i + 2, Tok::Operator));
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if c == '$' {
            if starts_with(chars, i, "$$") {
                if let Some(end) = find_seq(chars, i + 2, "$$") {
                    spans.push(Span::new(i, end + 2, Tok::String));
                    i = end + 2;
                } else {
                    spans.push(Span::new(i, n, Tok::String));
                    state.comment = 1;
                    return (spans, state);
                }
            } else {
                let mut j = i + 1;
                let mut close = None;
                while j < n {
                    if chars[j] == '$' && !is_escaped(chars, j) {
                        close = Some(j);
                        break;
                    }
                    j += 1;
                }
                if let Some(end) = close {
                    spans.push(Span::new(i, end + 1, Tok::String));
                    i = end + 1;
                } else {
                    spans.push(Span::new(i, n, Tok::String));
                    i = n;
                }
            }
            continue;
        }
        if matches!(c, '{' | '}' | '[' | ']' | '&' | '~') {
            spans.push(Span::new(i, i + 1, Tok::Operator));
            i += 1;
            continue;
        }
        if c == '#' {
            spans.push(Span::new(i, i + 1, Tok::Preproc));
            i += 1;
            continue;
        }
        if c.is_ascii_digit() {
            let end = scan_number(chars, i);
            spans.push(Span::new(i, end, Tok::Number));
            i = end;
            continue;
        }
        i += 1;
    }
    (spans, state)
}

// ---------------------------------------------------------------------------
// reStructuredText and AsciiDoc

fn lex_rst(chars: &[char], state: LexState) -> (Vec<Span>, LexState) {
    let n = chars.len();
    let text: String = chars.iter().copied().collect();
    let trimmed = text.trim_start();
    let indent = n - trimmed.chars().count();
    let mut spans = Vec::new();
    if trimmed.is_empty() {
        return (spans, state);
    }

    // Section underlines and overlines are runs of a single punctuation char.
    let adornments = "=-~^\"'`#*+:.!";
    let first = trimmed.chars().next().unwrap_or(' ');
    if trimmed.chars().count() >= 2
        && adornments.contains(first)
        && trimmed.chars().all(|c| c == first)
    {
        spans.push(Span::new(indent, n, Tok::Heading));
        return (spans, state);
    }

    let mut inline_start = indent;
    if trimmed.starts_with("..") {
        let mut name_start = indent + 2;
        while name_start < n && chars[name_start].is_whitespace() {
            name_start += 1;
        }
        if let Some(colon) = find_seq(chars, name_start, "::") {
            spans.push(Span::new(indent, colon + 2, Tok::Preproc));
            inline_start = colon + 2;
        } else {
            spans.push(Span::new(indent, n, Tok::Comment));
            return (spans, state);
        }
    }

    if chars.get(indent) == Some(&'|') {
        spans.push(Span::new(indent, indent + 1, Tok::Meta));
    }

    // Field lists: `:name: value`.
    if chars.get(indent) == Some(&':') {
        if let Some(close) = find_seq(chars, indent + 1, ":") {
            spans.push(Span::new(indent, close + 1, Tok::Type));
            inline_start = close + 1;
        }
    }

    let mut i = inline_start;
    while i < n {
        match chars[i] {
            '`' => {
                let run = chars[i..].iter().take_while(|c| **c == '`').count();
                let marker = "`".repeat(run);
                if let Some(close) = find_seq(chars, i + run, &marker) {
                    let mut end = close + run;
                    let kind = if run >= 2 {
                        Tok::String
                    } else {
                        if end < n && chars[end] == '_' {
                            end += 1;
                        }
                        Tok::Link
                    };
                    spans.push(Span::new(i, end, kind));
                    i = end;
                } else {
                    i += run;
                }
            }
            '*' | '_' => {
                let c = chars[i];
                let double = i + 1 < n && chars[i + 1] == c;
                let marker: String = if double {
                    c.to_string().repeat(2)
                } else {
                    c.to_string()
                };
                let from = i + marker.chars().count();
                if let Some(close) = find_seq(chars, from, &marker) {
                    if close > from {
                        spans.push(Span::new(i, close + marker.chars().count(), Tok::Emphasis));
                        i = close + marker.chars().count();
                        continue;
                    }
                }
                i += 1;
            }
            ':' => {
                if i + 1 < n && is_ident_start(chars[i + 1]) {
                    let mut j = i + 1;
                    while j < n && (is_ident_continue(chars[j]) || chars[j] == '-') {
                        j += 1;
                    }
                    if j < n && chars[j] == ':' {
                        spans.push(Span::new(i, j + 1, Tok::Preproc));
                        i = j + 1;
                        continue;
                    }
                }
                i += 1;
            }
            'h' if starts_with(chars, i, "http://") || starts_with(chars, i, "https://") => {
                let end = scan_url(chars, i);
                spans.push(Span::new(i, end, Tok::Link));
                i = end;
            }
            _ => i += 1,
        }
    }
    (spans, state)
}

fn lex_asciidoc(chars: &[char], mut state: LexState) -> (Vec<Span>, LexState) {
    let n = chars.len();
    let text: String = chars.iter().copied().collect();
    let trimmed = text.trim_start();
    let indent = n - trimmed.chars().count();
    let mut spans = Vec::new();
    if trimmed.is_empty() {
        return (spans, state);
    }

    if state.comment > 0 {
        if trimmed == "////" {
            spans.push(Span::new(indent, n, Tok::Meta));
            state.comment = 0;
        } else {
            spans.push(Span::new(0, n, Tok::Comment));
        }
        return (spans, state);
    }
    if trimmed == "////" {
        spans.push(Span::new(indent, n, Tok::Meta));
        state.comment = 1;
        return (spans, state);
    }
    if trimmed.starts_with("//") {
        spans.push(Span::new(indent, n, Tok::Comment));
        return (spans, state);
    }

    let equals = trimmed.chars().take_while(|c| *c == '=').count();
    if (1..=6).contains(&equals) && trimmed.chars().nth(equals) == Some(' ') {
        spans.push(Span::new(indent, n, Tok::Heading));
        return (spans, state);
    }
    if ["----", "....", "++++", "****", "____", "====", "|===", "--"].contains(&trimmed) {
        spans.push(Span::new(indent, n, Tok::Meta));
        return (spans, state);
    }
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        spans.push(Span::new(indent, n, Tok::Preproc));
        return (spans, state);
    }

    let mut inline_start = indent;
    if chars.get(indent) == Some(&':') {
        if let Some(close) = find_seq(chars, indent + 1, ":") {
            spans.push(Span::new(indent, close + 1, Tok::Type));
            inline_start = close + 1;
        }
    }

    // List bullets.
    let bullet_end = {
        let c = chars.get(indent).copied().unwrap_or(' ');
        if matches!(c, '*' | '-' | '.' | '+') && chars.get(indent + 1) == Some(&' ') {
            Some(indent + 1)
        } else {
            let digits = chars[indent..]
                .iter()
                .take_while(|c| c.is_ascii_digit())
                .count();
            if digits > 0 && chars.get(indent + digits) == Some(&'.') {
                Some(indent + digits + 1)
            } else {
                None
            }
        }
    };
    if let Some(end) = bullet_end {
        spans.push(Span::new(indent, end, Tok::Meta));
        inline_start = inline_start.max(end);
    }

    let mut i = inline_start;
    while i < n {
        match chars[i] {
            '`' => {
                let run = chars[i..].iter().take_while(|c| **c == '`').count();
                let marker = "`".repeat(run);
                if let Some(close) = find_seq(chars, i + run, &marker) {
                    spans.push(Span::new(i, close + run, Tok::String));
                    i = close + run;
                } else {
                    i += run;
                }
            }
            '*' | '_' => {
                let c = chars[i];
                let marker: String = c.to_string();
                if let Some(close) = find_seq(chars, i + 1, &marker) {
                    if close > i + 1 {
                        spans.push(Span::new(i, close + 1, Tok::Emphasis));
                        i = close + 1;
                        continue;
                    }
                }
                i += 1;
            }
            'h' if starts_with(chars, i, "http://") || starts_with(chars, i, "https://") => {
                let end = scan_url(chars, i);
                spans.push(Span::new(i, end, Tok::Link));
                i = end;
            }
            'l' if starts_with(chars, i, "link:") || starts_with(chars, i, "xref:") => {
                if let Some(end) = scan_asciidoc_link(chars, i) {
                    spans.push(Span::new(i, end, Tok::Link));
                    i = end;
                } else {
                    i += 1;
                }
            }
            'i' if starts_with(chars, i, "image:") => {
                if let Some(end) = scan_asciidoc_link(chars, i) {
                    spans.push(Span::new(i, end, Tok::Preproc));
                    i = end;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    (spans, state)
}

fn scan_url(chars: &[char], start: usize) -> usize {
    let n = chars.len();
    let mut i = start;
    while i < n && !chars[i].is_whitespace() && !matches!(chars[i], '<' | '>' | '[' | ']' | ')') {
        i += 1;
    }
    i
}

fn scan_asciidoc_link(chars: &[char], start: usize) -> Option<usize> {
    let bracket = find_seq(chars, start, "[")?;
    let close = find_seq(chars, bracket + 1, "]")?;
    Some(close + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(line: &str, lang: Lang) -> (Vec<(String, Tok)>, LexState) {
        let chars: Vec<char> = line.chars().collect();
        let (spans, state) = lex(lang, &chars, LexState::default());
        let out = spans
            .iter()
            .map(|s| (chars[s.start..s.end].iter().collect::<String>(), s.kind))
            .collect();
        (out, state)
    }

    fn has(spans: &[(String, Tok)], text: &str, kind: Tok) -> bool {
        spans.iter().any(|(t, k)| t == text && *k == kind)
    }

    #[test]
    fn detects_more_languages() {
        let cases = [
            ("A.java", Lang::Java),
            ("a.kt", Lang::Kotlin),
            ("a.swift", Lang::Swift),
            ("a.zig", Lang::Zig),
            ("a.dart", Lang::Dart),
            ("a.rb", Lang::Ruby),
            ("a.php", Lang::Php),
            ("a.lua", Lang::Lua),
            ("a.hs", Lang::Haskell),
            ("a.ex", Lang::Elixir),
            ("a.sql", Lang::Sql),
            ("a.tex", Lang::Latex),
            ("a.rst", Lang::Rst),
            ("a.adoc", Lang::AsciiDoc),
            ("a.ini", Lang::Ini),
            ("a.tf", Lang::Hcl),
            ("a.nix", Lang::Nix),
            ("a.el", Lang::Lisp),
            ("a.scm", Lang::Lisp),
            ("a.clj", Lang::Lisp),
            ("a.rkt", Lang::Lisp),
            ("a.cmake", Lang::Cmake),
            ("a.proto", Lang::Protobuf),
            ("a.vue", Lang::Vue),
            ("a.svelte", Lang::Svelte),
        ];
        for (path, lang) in cases {
            assert_eq!(
                Lang::detect(Some(Path::new(path)), None),
                lang,
                "extension detection for {path}"
            );
        }
        assert_eq!(Lang::detect(Some(Path::new("Gemfile")), None), Lang::Ruby);
        assert_eq!(
            Lang::detect(Some(Path::new("CMakeLists.txt")), None),
            Lang::Cmake
        );
        assert_eq!(
            Lang::detect(Some(Path::new("Dockerfile.dev")), None),
            Lang::Dockerfile
        );
        assert_eq!(Lang::detect(Some(Path::new(".env")), None), Lang::Ini);
        assert_eq!(Lang::detect(Some(Path::new(".bashrc")), None), Lang::Shell);
        assert_eq!(Lang::detect(None, Some("#!/usr/bin/env ruby")), Lang::Ruby);
        assert_eq!(
            Lang::detect(None, Some("#!/usr/bin/env -S nix-shell")),
            Lang::Nix
        );
        assert_eq!(
            Lang::detect(None, Some("#!/usr/bin/env runhaskell")),
            Lang::Haskell
        );
    }

    #[test]
    fn java_kotlin_swift_basics() {
        let (spans, _) = kinds(r#"public class Main { String s = "x"; } // c"#, Lang::Java);
        assert!(has(&spans, "public", Tok::Keyword));
        assert!(has(&spans, "String", Tok::Type));
        assert!(has(&spans, r#""x""#, Tok::String));
        assert!(has(&spans, "// c", Tok::Comment));

        let (spans, _) = kinds("fun main() { val x = 1 } // c", Lang::Kotlin);
        assert!(has(&spans, "fun", Tok::Keyword));
        assert!(has(&spans, "main", Tok::Function));
        assert!(has(&spans, "1", Tok::Number));

        let (spans, _) = kinds(r#"func main() { let s = "hi" } // c"#, Lang::Swift);
        assert!(has(&spans, "func", Tok::Keyword));
        assert!(has(&spans, "main", Tok::Function));
        assert!(has(&spans, r#""hi""#, Tok::String));
    }

    #[test]
    fn zig_dart_basics() {
        let (spans, _) = kinds(r#"const std = @import("std"); // c"#, Lang::Zig);
        assert!(has(&spans, "const", Tok::Keyword));
        assert!(has(&spans, "@import", Tok::Preproc));
        assert!(has(&spans, r#""std""#, Tok::String));
        assert!(has(&spans, "// c", Tok::Comment));
        let (spans, _) = kinds(r"const s = \\hello", Lang::Zig);
        assert!(has(&spans, r"\\hello", Tok::String));

        let (spans, _) = kinds(r#"void main() { var s = "hi"; } // c"#, Lang::Dart);
        assert!(has(&spans, "void", Tok::Keyword));
        assert!(has(&spans, "main", Tok::Function));
        assert!(has(&spans, r#""hi""#, Tok::String));
    }

    #[test]
    fn ruby_php_basics() {
        let (spans, _) = kinds("def greet(name) # c", Lang::Ruby);
        assert!(has(&spans, "def", Tok::Keyword));
        assert!(has(&spans, "greet", Tok::Function));
        assert!(has(&spans, "# c", Tok::Comment));
        let (spans, _) = kinds("x = :sym", Lang::Ruby);
        assert!(has(&spans, ":sym", Tok::Constant));
        let (spans, _) = kinds("@ivar = 1", Lang::Ruby);
        assert!(has(&spans, "@ivar", Tok::Preproc));
        let (spans, _) = kinds("def self.greet; end", Lang::Ruby);
        assert!(has(&spans, "self", Tok::Keyword));
        assert!(has(&spans, "greet", Tok::Function));
        let (_, state) = kinds("=begin", Lang::Ruby);
        assert_eq!(state.comment, 1);
        let (_, state) = kinds("x=begin", Lang::Ruby);
        assert_eq!(state.comment, 0);

        let (spans, _) = kinds(r#"FUNCTION foo($x) { echo "hi"; } // c"#, Lang::Php);
        assert!(has(&spans, "FUNCTION", Tok::Keyword));
        assert!(has(&spans, "foo", Tok::Function));
        assert!(has(&spans, "$x", Tok::Preproc));
        assert!(has(&spans, r#""hi""#, Tok::String));
        let (spans, _) = kinds("#[Attribute]", Lang::Php);
        assert_eq!(spans[0].1, Tok::Preproc);
    }

    #[test]
    fn lua_haskell_elixir_sql_basics() {
        let (spans, _) = kinds(r#"local function f(x) return "a" end -- c"#, Lang::Lua);
        assert!(has(&spans, "local", Tok::Keyword));
        assert!(has(&spans, "f", Tok::Function));
        assert!(has(&spans, r#""a""#, Tok::String));
        assert!(has(&spans, "-- c", Tok::Comment));
        let (spans, _) = kinds("x = [[abc]]", Lang::Lua);
        assert!(has(&spans, "[[abc]]", Tok::String));

        let (spans, _) = kinds(r#"let x = "hi" -- c"#, Lang::Haskell);
        assert!(has(&spans, "let", Tok::Keyword));
        assert!(has(&spans, r#""hi""#, Tok::String));
        assert!(has(&spans, "-- c", Tok::Comment));
        let (spans, _) = kinds("main :: IO ()", Lang::Haskell);
        assert!(has(&spans, "IO", Tok::Type));

        let (spans, _) = kinds("defmodule Foo do", Lang::Elixir);
        assert!(has(&spans, "defmodule", Tok::Keyword));
        assert!(has(&spans, "Foo", Tok::Type));
        let (spans, _) = kinds("x = :ok # c", Lang::Elixir);
        assert!(has(&spans, ":ok", Tok::Constant));
        assert!(has(&spans, "# c", Tok::Comment));

        let (spans, _) = kinds("select * from users where id = 1; -- c", Lang::Sql);
        assert!(has(&spans, "select", Tok::Keyword));
        assert!(has(&spans, "1", Tok::Number));
        assert!(has(&spans, "-- c", Tok::Comment));
    }

    #[test]
    fn config_and_markup_basics() {
        let (spans, _) = kinds("cmake_minimum_required(VERSION 3.20) # c", Lang::Cmake);
        assert!(has(&spans, "cmake_minimum_required", Tok::Keyword));
        assert!(has(&spans, "3.20", Tok::Number));
        assert!(has(&spans, "# c", Tok::Comment));

        let (spans, _) = kinds(r#"resource "aws_instance" "web" { # c"#, Lang::Hcl);
        assert!(has(&spans, "resource", Tok::Keyword));
        assert!(has(&spans, r#""aws_instance""#, Tok::String));
        assert!(has(&spans, "# c", Tok::Comment));

        let (spans, _) = kinds(r#"let x = "hi"; in x # c"#, Lang::Nix);
        assert!(has(&spans, "let", Tok::Keyword));
        assert!(has(&spans, "in", Tok::Keyword));
        assert!(has(&spans, r#""hi""#, Tok::String));
        assert!(has(&spans, "# c", Tok::Comment));
        let (spans, _) = kinds("x = ''hello''", Lang::Nix);
        assert!(has(&spans, "''hello''", Tok::String));

        let (spans, _) = kinds("message Foo { string name = 1; } // c", Lang::Protobuf);
        assert!(has(&spans, "message", Tok::Keyword));
        assert!(has(&spans, "string", Tok::Type));
        assert!(has(&spans, "1", Tok::Number));
        assert!(has(&spans, "// c", Tok::Comment));

        let (spans, _) = kinds(
            "FROM --platform=$BUILDPLATFORM rust:1 AS build",
            Lang::Dockerfile,
        );
        assert!(has(&spans, "FROM", Tok::Keyword));
        assert!(has(&spans, "--platform", Tok::Preproc));
        assert!(has(&spans, "$BUILDPLATFORM", Tok::Preproc));
        assert!(has(&spans, "AS", Tok::Keyword));
        let (spans, _) = kinds("# comment", Lang::Dockerfile);
        assert_eq!(spans[0].1, Tok::Comment);

        let (spans, _) = kinds("[core]", Lang::Ini);
        assert_eq!(spans[0].1, Tok::Heading);
        let (spans, _) = kinds(r#"key = "value" ; c"#, Lang::Ini);
        assert!(has(&spans, "key", Tok::Type));
        assert!(has(&spans, "=", Tok::Operator));
        assert!(has(&spans, r#""value""#, Tok::String));
        assert!(has(&spans, "; c", Tok::Comment));

        let (spans, _) = kinds(r"\section{Title} % c", Lang::Latex);
        assert!(has(&spans, r"\section", Tok::Heading));
        assert!(has(&spans, "% c", Tok::Comment));
        let (spans, _) = kinds("$x^2$", Lang::Latex);
        assert!(has(&spans, "$x^2$", Tok::String));

        let (spans, _) = kinds("=====", Lang::Rst);
        assert_eq!(spans[0].1, Tok::Heading);
        let (spans, _) = kinds("*em* ``code``", Lang::Rst);
        assert!(has(&spans, "*em*", Tok::Emphasis));
        assert!(has(&spans, "``code``", Tok::String));

        let (spans, _) = kinds("= Title", Lang::AsciiDoc);
        assert_eq!(spans[0].1, Tok::Heading);
        let (spans, _) = kinds(":attr: value", Lang::AsciiDoc);
        assert!(has(&spans, ":attr:", Tok::Type));
        let (spans, _) = kinds("*bold*", Lang::AsciiDoc);
        assert!(has(&spans, "*bold*", Tok::Emphasis));
        let (spans, _) = kinds("// c", Lang::AsciiDoc);
        assert_eq!(spans[0].1, Tok::Comment);
    }

    #[test]
    fn multiline_states_carry() {
        let chars: Vec<char> = "x = 1 --[[ open".chars().collect();
        let (_, state) = lex(Lang::Lua, &chars, LexState::default());
        assert_eq!(state.comment, 1);
        let chars: Vec<char> = "still ]] y = 2".chars().collect();
        let (spans, state) = lex(Lang::Lua, &chars, state);
        assert_eq!(state.comment, 0);
        assert_eq!(spans[0].kind, Tok::Comment);
        assert!(spans.iter().any(|s| s.kind == Tok::Number));

        let chars: Vec<char> = "s = [[hello".chars().collect();
        let (_, state) = lex(Lang::Lua, &chars, LexState::default());
        assert_eq!(state.string, b'[');
        let chars: Vec<char> = "world]]".chars().collect();
        let (spans, state) = lex(Lang::Lua, &chars, state);
        assert_eq!(state.string, 0);
        assert_eq!(spans[0].kind, Tok::String);

        let chars: Vec<char> = "{- open".chars().collect();
        let (_, state) = lex(Lang::Haskell, &chars, LexState::default());
        assert_eq!(state.comment, 1);
        let chars: Vec<char> = "{- nested -}".chars().collect();
        let (_, state) = lex(Lang::Haskell, &chars, state);
        assert_eq!(state.comment, 1);
        let chars: Vec<char> = "-}".chars().collect();
        let (spans, state) = lex(Lang::Haskell, &chars, state);
        assert_eq!(state.comment, 0);
        assert_eq!(spans[0].kind, Tok::Comment);

        let chars: Vec<char> = "x = ''hello".chars().collect();
        let (_, state) = lex(Lang::Nix, &chars, LexState::default());
        assert_eq!(state.string, b'\'');
        let chars: Vec<char> = "world''".chars().collect();
        let (spans, state) = lex(Lang::Nix, &chars, state);
        assert_eq!(state.string, 0);
        assert_eq!(spans[0].kind, Tok::String);

        let chars: Vec<char> = "x = <<EOF".chars().collect();
        let (_, state) = lex(Lang::Hcl, &chars, LexState::default());
        assert_eq!(state.string, b'<');
        let chars: Vec<char> = "hello".chars().collect();
        let (spans, state) = lex(Lang::Hcl, &chars, state);
        assert_eq!(spans[0].kind, Tok::String);
        assert_eq!(state.string, b'<');
        let chars: Vec<char> = "EOF".chars().collect();
        let (spans, state) = lex(Lang::Hcl, &chars, state);
        assert_eq!(spans[0].kind, Tok::Meta);
        assert_eq!(state.string, 0);

        let chars: Vec<char> = "RUN <<-EOF".chars().collect();
        let (_, state) = lex(Lang::Dockerfile, &chars, LexState::default());
        assert_eq!(state.string, b'<');
        let chars: Vec<char> = "  body".chars().collect();
        let (spans, state) = lex(Lang::Dockerfile, &chars, state);
        assert_eq!(spans[0].kind, Tok::String);
        let chars: Vec<char> = "  EOF".chars().collect();
        let (spans, state) = lex(Lang::Dockerfile, &chars, state);
        assert_eq!(spans[0].kind, Tok::Meta);
        assert_eq!(state.string, 0);

        let chars: Vec<char> = "x = <<<EOT".chars().collect();
        let (_, state) = lex(Lang::Php, &chars, LexState::default());
        assert_eq!(state.string, b'<');
        let chars: Vec<char> = "body".chars().collect();
        let (spans, state) = lex(Lang::Php, &chars, state);
        assert_eq!(spans[0].kind, Tok::String);
        let chars: Vec<char> = "  EOT".chars().collect();
        let (spans, state) = lex(Lang::Php, &chars, state);
        assert_eq!(spans[0].kind, Tok::Meta);
        assert_eq!(state.string, 0);

        let chars: Vec<char> = r#"var s = """hello"#.chars().collect();
        let (_, state) = lex(Lang::Dart, &chars, LexState::default());
        assert_eq!(state.string, b'"');
        let chars: Vec<char> = r#"world""" + 1"#.chars().collect();
        let (spans, state) = lex(Lang::Dart, &chars, state);
        assert_eq!(state.string, 0);
        assert_eq!(spans[0].kind, Tok::String);

        let chars: Vec<char> = r"\begin{verbatim}".chars().collect();
        let (_, state) = lex(Lang::Latex, &chars, LexState::default());
        assert_eq!(state.string, b'v');
        let chars: Vec<char> = r"\LaTeX % not a comment".chars().collect();
        let (spans, state) = lex(Lang::Latex, &chars, state);
        assert_eq!(spans[0].kind, Tok::String);
        let chars: Vec<char> = r"\end{verbatim}".chars().collect();
        let (spans, state) = lex(Lang::Latex, &chars, state);
        assert_eq!(spans[0].kind, Tok::String);
        assert_eq!(state.string, 0);

        let (spans, state) = kinds(r"\begin{verbatim}x\end{verbatim}", Lang::Latex);
        assert_eq!(state.string, 0);
        assert!(has(&spans, r"x\end{verbatim}", Tok::String));

        let chars: Vec<char> = "#[[ open".chars().collect();
        let (_, state) = lex(Lang::Cmake, &chars, LexState::default());
        assert_eq!(state.comment, 1);
        let chars: Vec<char> = "still ]] x = 1".chars().collect();
        let (spans, state) = lex(Lang::Cmake, &chars, state);
        assert_eq!(state.comment, 0);
        assert_eq!(spans[0].kind, Tok::Comment);
        assert!(spans.iter().any(|s| s.kind == Tok::Number));
    }

    #[test]
    fn arbitrary_text_produces_valid_spans() {
        let langs = [
            Lang::Java,
            Lang::Kotlin,
            Lang::Swift,
            Lang::Zig,
            Lang::Dart,
            Lang::Ruby,
            Lang::Php,
            Lang::Lua,
            Lang::Haskell,
            Lang::Elixir,
            Lang::Sql,
            Lang::Latex,
            Lang::Rst,
            Lang::AsciiDoc,
            Lang::Ini,
            Lang::Hcl,
            Lang::Nix,
            Lang::Cmake,
            Lang::Dockerfile,
            Lang::Protobuf,
            Lang::Vue,
            Lang::Svelte,
        ];
        let alphabet = [
            'a', 'Z', '0', ' ', '"', '\'', '/', '*', '#', '-', '[', ']', '{', '}', '<', '>', '@',
            ':', '$', '%', '\\', '`', '=', 'あ',
        ];
        let mut seed = 0x1234_5678_9abc_def0u64;
        for lang in langs {
            for _ in 0..200 {
                let len = (seed % 40) as usize;
                let mut text = String::new();
                for _ in 0..len {
                    seed = seed
                        .wrapping_mul(6_364_136_223_846_793_005)
                        .wrapping_add(1_442_695_040_888_963_407);
                    text.push(alphabet[(seed as usize) % alphabet.len()]);
                }
                let chars: Vec<char> = text.chars().collect();
                let (spans, _) = lex(lang, &chars, LexState::default());
                let mut last = 0;
                for span in spans {
                    assert!(span.start <= span.end);
                    assert!(span.end <= chars.len());
                    assert!(span.start >= last, "overlap in {text:?} ({lang:?})");
                    last = span.end;
                }
            }
        }
    }

    #[test]
    fn spans_are_sorted_and_within_bounds() {
        let cases = [
            ("fn main() { println!(\"hi\"); }", Lang::Rust),
            ("def f(x): return {1: 'a'}", Lang::Python),
            ("SELECT * FROM x; -- c", Lang::Plain),
            ("a: 1\n", Lang::Yaml),
            ("[table]\nkey = \"v\"", Lang::Toml),
            ("public class 名前 { String s = \"あ\"; }", Lang::Java),
            ("fun 名前() = 1 // c", Lang::Kotlin),
            ("func 名前() { let s = \"あ\" }", Lang::Swift),
            ("const s = \"あ\"; // c", Lang::Zig),
            ("void 名前() { var s = \"あ\"; }", Lang::Dart),
            ("def 名前; @x = :sym; end # c", Lang::Ruby),
            ("FUNCTION 名前($x) { echo \"あ\"; }", Lang::Php),
            ("local x = [[あ]] -- c", Lang::Lua),
            ("main :: IO (); x = \"あ\" -- c", Lang::Haskell),
            ("defmodule 名前 do :ok end # c", Lang::Elixir),
            ("SELECT 名前 FROM x; -- c", Lang::Sql),
            ("cmake_minimum_required(VERSION 3.20) # あ", Lang::Cmake),
            ("resource \"x\" \"名前\" { # あ", Lang::Hcl),
            ("let x = \"あ\"; in x # c", Lang::Nix),
            ("message 名前 { string x = 1; } // あ", Lang::Protobuf),
            ("FROM x AS y # あ", Lang::Dockerfile),
            ("[名前]\nkey = \"あ\"; c", Lang::Ini),
            ("\\section{名前} % あ", Lang::Latex),
            ("名前\n=====\n*あ* ``code``", Lang::Rst),
            ("= 名前\n:attr: あ\n*bold* // c", Lang::AsciiDoc),
            ("<div class=\"名前\">あ</div>", Lang::Vue),
            ("<script>let x = \"あ\";</script>", Lang::Svelte),
        ];
        for (text, lang) in cases {
            let chars: Vec<char> = text.chars().collect();
            let (spans, _) = lex(lang, &chars, LexState::default());
            let mut last = 0;
            for s in spans {
                assert!(s.start <= s.end);
                assert!(s.end <= chars.len());
                assert!(s.start >= last, "overlap in {text:?}");
                last = s.end;
            }
        }
    }

    #[test]
    fn detects_languages() {
        assert_eq!(
            Lang::detect(Some(Path::new("/tmp/.git/COMMIT_EDITMSG")), None),
            Lang::GitCommit
        );
        assert_eq!(Lang::detect(Some(Path::new("a.rs")), None), Lang::Rust);
        assert_eq!(
            Lang::detect(Some(Path::new("Makefile")), None),
            Lang::Makefile
        );
        assert_eq!(
            Lang::detect(None, Some("#!/usr/bin/env python3")),
            Lang::Python
        );
        assert_eq!(
            Lang::detect(Some(Path::new("x.unknown")), None),
            Lang::Plain
        );
    }

    #[test]
    fn lisp_basics() {
        let (spans, _) = kinds("(defun f (x) \"; c\") ; tail", Lang::Lisp);
        assert!(spans.contains(&("defun".into(), Tok::Keyword)));
        assert!(spans.contains(&("\"; c\"".into(), Tok::String)));
        assert!(spans.contains(&("; tail".into(), Tok::Comment)));
    }

    #[test]
    fn rust_basics() {
        let (spans, _) = kinds("let x = 1; // hi", Lang::Rust);
        assert!(spans.contains(&("let".into(), Tok::Keyword)));
        assert!(spans.contains(&("1".into(), Tok::Number)));
        assert!(spans.contains(&("// hi".into(), Tok::Comment)));
    }

    #[test]
    fn rust_block_comment_carries_state() {
        let chars: Vec<char> = "a /* open".chars().collect();
        let (_, state) = lex(Lang::Rust, &chars, LexState::default());
        assert_eq!(state.comment, 1);
        let chars: Vec<char> = " still */ b".chars().collect();
        let (spans, state) = lex(Lang::Rust, &chars, state);
        assert_eq!(state.comment, 0);
        assert_eq!(spans[0].kind, Tok::Comment);
        assert_eq!(spans[0].end, 9);
    }

    #[test]
    fn rust_raw_string_is_single_token() {
        let (spans, state) = kinds(r##"let s = r#"a " b"#;"##, Lang::Rust);
        assert_eq!(state.string, 0);
        assert!(spans
            .iter()
            .any(|(t, k)| t == r##"r#"a " b"#"## && *k == Tok::String));
    }

    #[test]
    fn python_triple_string_spans_lines() {
        let chars: Vec<char> = r#"x = """hello"#.chars().collect();
        let (_, state) = lex(Lang::Python, &chars, LexState::default());
        assert_eq!(state.string, b'"');
        assert!(state.triple);
        let chars: Vec<char> = r#"world""" + 1"#.chars().collect();
        let (spans, state) = lex(Lang::Python, &chars, state);
        assert_eq!(state.string, 0);
        assert_eq!(spans[0].kind, Tok::String);
    }

    #[test]
    fn python_def_name_is_function() {
        let (spans, _) = kinds("def main():", Lang::Python);
        assert!(spans.contains(&("def".into(), Tok::Keyword)));
        assert!(spans.contains(&("main".into(), Tok::Function)));
    }

    #[test]
    fn markdown_fence_carries_state() {
        let chars: Vec<char> = "```rust".chars().collect();
        let (_, state) = lex(Lang::Markdown, &chars, LexState::default());
        assert_eq!(state.fence, b'`');
        let chars: Vec<char> = "let x = 1;".chars().collect();
        let (spans, state) = lex(Lang::Markdown, &chars, state);
        assert_eq!(spans[0].kind, Tok::String);
        assert_eq!(state.fence, b'`');
    }

    #[test]
    fn commit_message_summary_and_comments() {
        let chars: Vec<char> = "fix: something".chars().collect();
        let (spans, state) = lex(Lang::GitCommit, &chars, LexState::default());
        assert_eq!(spans[0].kind, Tok::Heading);
        let chars: Vec<char> = "# Please enter the commit message".chars().collect();
        let (spans, _) = lex(Lang::GitCommit, &chars, state);
        assert_eq!(spans[0].kind, Tok::Comment);
    }

    #[test]
    fn diff_lines() {
        let (spans, _) = kinds("+added", Lang::Diff);
        assert_eq!(spans[0].1, Tok::DiffAdd);
        let (spans, _) = kinds("@@ -1 +1 @@", Lang::Diff);
        assert_eq!(spans[0].1, Tok::DiffHunk);
    }

    #[test]
    fn cjk_identifiers_do_not_panic() {
        let (spans, _) = kinds("let 名前 = 1;", Lang::Rust);
        assert!(spans.contains(&("let".into(), Tok::Keyword)));
        assert!(spans.contains(&("1".into(), Tok::Number)));
    }
}
