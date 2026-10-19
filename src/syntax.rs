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
    Python,
    JavaScript,
    TypeScript,
    Shell,
    Markdown,
    Json,
    Toml,
    Yaml,
    Makefile,
    Html,
    Css,
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
            Lang::Python => "Python",
            Lang::JavaScript => "JavaScript",
            Lang::TypeScript => "TypeScript",
            Lang::Shell => "Shell",
            Lang::Markdown => "Markdown",
            Lang::Json => "JSON",
            Lang::Toml => "TOML",
            Lang::Yaml => "YAML",
            Lang::Makefile => "Makefile",
            Lang::Html => "HTML",
            Lang::Css => "CSS",
            Lang::Diff => "Diff",
            Lang::GitCommit => "GitCommit",
        }
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
                _ => {}
            }
            if name.ends_with(".mk") {
                return Lang::Makefile;
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
                "cc" | "cpp" | "cxx" | "hpp" | "hh" | "hxx" => Lang::Cpp,
                "py" | "pyw" | "pyi" => Lang::Python,
                "js" | "mjs" | "cjs" | "jsx" => Lang::JavaScript,
                "ts" | "tsx" | "mts" | "cts" => Lang::TypeScript,
                "sh" | "bash" | "zsh" | "fish" | "ksh" => Lang::Shell,
                "md" | "markdown" | "mdx" => Lang::Markdown,
                "json" | "jsonc" => Lang::Json,
                "toml" => Lang::Toml,
                "yaml" | "yml" => Lang::Yaml,
                "html" | "htm" | "xhtml" => Lang::Html,
                "css" | "scss" | "sass" | "less" => Lang::Css,
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
                if cmd.contains("bash")
                    || cmd.contains("zsh")
                    || cmd.contains("sh")
                    || cmd.contains("fish")
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
}

pub fn lex(lang: Lang, chars: &[char], state: LexState) -> (Vec<Span>, LexState) {
    match lang {
        Lang::Plain => (Vec::new(), LexState::default()),
        Lang::Rust => lex_code(chars, state, &RUST),
        Lang::Go => lex_code(chars, state, &GO),
        Lang::C => lex_code(chars, state, &C),
        Lang::Cpp => lex_code(chars, state, &CPP),
        Lang::Python => lex_code(chars, state, &PYTHON),
        Lang::JavaScript => lex_code(chars, state, &JS),
        Lang::TypeScript => lex_code(chars, state, &TS),
        Lang::Shell => lex_code(chars, state, &SHELL),
        Lang::Json => lex_structured(chars, state, Structured::Json),
        Lang::Toml => lex_structured(chars, state, Structured::Toml),
        Lang::Yaml => lex_structured(chars, state, Structured::Yaml),
        Lang::Markdown => lex_markdown(chars, state),
        Lang::Makefile => lex_makefile(chars, state),
        Lang::Html => lex_html(chars, state),
        Lang::Css => lex_css(chars, state),
        Lang::Diff => (lex_diff(chars), state),
        Lang::GitCommit => lex_gitcommit(chars, state),
    }
}

// ---------------------------------------------------------------------------
// helpers

fn starts_with(chars: &[char], i: usize, pat: &str) -> bool {
    let mut j = i;
    for pc in pat.chars() {
        if j >= chars.len() || chars[j] != pc {
            return false;
        }
        j += 1;
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
const BACKTICK: Quote = Quote {
    ch: '`',
    multiline: true,
    escapes: false,
};

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

const RUST: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", true)),
    quotes: &[DOUBLE],
    char_literal: true,
    triple_quotes: false,
    keywords: RUST_KEYWORDS,
    types: RUST_TYPES,
    constants: RUST_CONSTANTS,
    preproc: false,
    rust_raw: true,
    python: false,
    shell: false,
    hash_needs_space: false,
};
const GO: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", false)),
    quotes: &[DOUBLE, BACKTICK],
    char_literal: true,
    triple_quotes: false,
    keywords: GO_KEYWORDS,
    types: GO_TYPES,
    constants: GO_CONSTANTS,
    preproc: false,
    rust_raw: false,
    python: false,
    shell: false,
    hash_needs_space: false,
};
const C: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", false)),
    quotes: &[DOUBLE],
    char_literal: true,
    triple_quotes: false,
    keywords: C_KEYWORDS,
    types: C_TYPES,
    constants: C_CONSTANTS,
    preproc: true,
    rust_raw: false,
    python: false,
    shell: false,
    hash_needs_space: false,
};
const CPP: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", false)),
    quotes: &[DOUBLE],
    char_literal: true,
    triple_quotes: false,
    keywords: C_KEYWORDS,
    types: C_TYPES,
    constants: C_CONSTANTS,
    preproc: true,
    rust_raw: false,
    python: false,
    shell: false,
    hash_needs_space: false,
};
const PYTHON: CodeSpec = CodeSpec {
    line_comments: &["#"],
    block_comment: None,
    quotes: &[DOUBLE, SINGLE],
    char_literal: false,
    triple_quotes: true,
    keywords: PY_KEYWORDS,
    types: PY_TYPES,
    constants: PY_CONSTANTS,
    preproc: false,
    rust_raw: false,
    python: true,
    shell: false,
    hash_needs_space: false,
};
const JS: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", false)),
    quotes: &[DOUBLE, SINGLE, BACKTICK],
    char_literal: false,
    triple_quotes: false,
    keywords: JS_KEYWORDS,
    types: JS_TYPES,
    constants: JS_CONSTANTS,
    preproc: false,
    rust_raw: false,
    python: false,
    shell: false,
    hash_needs_space: false,
};
const TS: CodeSpec = CodeSpec {
    line_comments: &["//"],
    block_comment: Some(("/*", "*/", false)),
    quotes: &[DOUBLE, SINGLE, BACKTICK],
    char_literal: false,
    triple_quotes: false,
    keywords: JS_KEYWORDS,
    types: JS_TYPES,
    constants: JS_CONSTANTS,
    preproc: false,
    rust_raw: false,
    python: false,
    shell: false,
    hash_needs_space: false,
};
const SHELL: CodeSpec = CodeSpec {
    line_comments: &["#"],
    block_comment: None,
    quotes: &[DOUBLE, SHELL_SINGLE],
    char_literal: false,
    triple_quotes: false,
    keywords: SHELL_KEYWORDS,
    types: &[],
    constants: SHELL_CONSTANTS,
    preproc: false,
    rust_raw: false,
    python: false,
    shell: true,
    hash_needs_space: true,
};

fn lex_code(chars: &[char], mut state: LexState, spec: &CodeSpec) -> (Vec<Span>, LexState) {
    let n = chars.len();
    let mut spans: Vec<Span> = Vec::new();
    let mut i = 0usize;

    // Resume a block comment from the previous line.
    if state.comment > 0 {
        if let Some((open, close, nested)) = spec.block_comment {
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

        // line comments
        let mut line_comment = false;
        for lc in spec.line_comments {
            if starts_with(chars, i, lc) {
                if spec.hash_needs_space && *lc == "#" && i > 0 && !chars[i - 1].is_whitespace() {
                    continue;
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

        // block comment start
        if let Some((open, close, nested)) = spec.block_comment {
            if starts_with(chars, i, open) {
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

        // char literals (C/Rust/Go)
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

        // shell variables
        if spec.shell && c == '$' {
            let mut j = i + 1;
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
            if j > i + 1 {
                spans.push(Span::new(i, j, Tok::Preproc));
                i = j;
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
                Some(k)
            } else if spec.keywords.contains(&word.as_str()) {
                if spec.python {
                    if word == "def" {
                        pending = Some(Tok::Function);
                    } else if word == "class" {
                        pending = Some(Tok::Type);
                    }
                }
                Some(Tok::Keyword)
            } else if spec.types.contains(&word.as_str()) {
                Some(Tok::Type)
            } else if spec.constants.contains(&word.as_str()) {
                Some(Tok::Constant)
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

    #[test]
    fn spans_are_sorted_and_within_bounds() {
        let cases = [
            ("fn main() { println!(\"hi\"); }", Lang::Rust),
            ("def f(x): return {1: 'a'}", Lang::Python),
            ("SELECT * FROM x; -- c", Lang::Plain),
            ("a: 1\n", Lang::Yaml),
            ("[table]\nkey = \"v\"", Lang::Toml),
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
}
