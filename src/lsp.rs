//! A deliberately small LSP client over stdio.
//!
//! It supports what the editor currently needs: initialize, full-text
//! synchronization, `publishDiagnostics`, and completion. Everything runs on
//! a reader thread so the UI never blocks on a language server.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub row: usize,
    pub start: usize,
    pub end: usize,
    pub severity: Severity,
    pub message: String,
    pub source: Option<String>,
}

#[derive(Clone, Debug)]
pub enum LspEvent {
    Diagnostics(Vec<Diagnostic>),
    Completion { request_id: i64, items: Vec<String> },
    Error(String),
}

enum ReaderEvent {
    Message(Value),
    Error(String),
}

pub struct LspClient {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<ReaderEvent>,
    next_id: i64,
    init_id: i64,
    completion_id: Option<i64>,
    initialized: bool,
    opened: bool,
    version: i64,
    uri: String,
    language_id: String,
    root: PathBuf,
    lines: Vec<Vec<char>>,
    text: String,
    closed: bool,
}

impl LspClient {
    pub fn start(
        command_line: &str,
        language_id: &str,
        path: &Path,
        text: &str,
    ) -> io::Result<Self> {
        let mut parts = split_command(command_line);
        if parts.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "empty LSP command",
            ));
        }
        let program = parts.remove(0);
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };
        let root = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        let mut child = Command::new(&program)
            .args(&parts)
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "LSP stdin unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "LSP stdout unavailable"))?;
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || reader_loop(stdout, tx));

        let mut client = Self {
            child,
            stdin,
            rx,
            next_id: 2,
            init_id: 1,
            completion_id: None,
            initialized: false,
            opened: false,
            version: 1,
            uri: path_to_uri(&path),
            language_id: language_id.to_string(),
            root,
            lines: text_lines(text),
            text: text.to_string(),
            closed: false,
        };
        client.send_initialize()?;
        Ok(client)
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn did_change(&mut self, text: &str) {
        self.version += 1;
        self.text = text.to_string();
        self.lines = text_lines(text);
        if !self.initialized || !self.opened {
            return;
        }
        let params = json!({
            "textDocument": {"uri": self.uri, "version": self.version},
            "contentChanges": [{"text": text}]
        });
        let _ = self.send_notification("textDocument/didChange", params);
    }

    pub fn request_completion(&mut self, row: usize, col: usize) -> Option<i64> {
        if !self.initialized {
            return None;
        }
        if let Some(previous) = self.completion_id.take() {
            let _ = self.send_notification("$/cancelRequest", json!({"id": previous}));
        }
        let line = self.lines.get(row)?;
        let character = char_col_to_utf16(line, col);
        let id = self.next_id;
        self.next_id += 1;
        self.completion_id = Some(id);
        let params = json!({
            "textDocument": {"uri": self.uri},
            "position": {"line": row, "character": character}
        });
        if self
            .send_request(id, "textDocument/completion", params)
            .is_err()
        {
            self.completion_id = None;
            return None;
        }
        Some(id)
    }

    pub fn poll(&mut self) -> Vec<LspEvent> {
        let mut events = Vec::new();
        if self.closed {
            return events;
        }
        loop {
            match self.rx.try_recv() {
                Ok(ReaderEvent::Message(value)) => self.handle_message(value, &mut events),
                Ok(ReaderEvent::Error(message)) => {
                    self.closed = true;
                    events.push(LspEvent::Error(plain_text(&message)));
                    break;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.closed = true;
                    events.push(LspEvent::Error("LSP server exited".into()));
                    break;
                }
            }
        }
        events
    }

    fn handle_message(&mut self, value: Value, events: &mut Vec<LspEvent>) {
        if let Some(id) = value.get("id").and_then(Value::as_i64) {
            if id == self.init_id {
                if let Some(error) = value.get("error") {
                    events.push(LspEvent::Error(format!(
                        "LSP initialize failed: {}",
                        plain_text(&error.to_string())
                    )));
                    return;
                }
                self.initialized = true;
                self.send_notification("initialized", json!({})).ok();
                self.send_did_open();
                return;
            }
            if self.completion_id == Some(id) {
                self.completion_id = None;
                let items = value
                    .get("result")
                    .map(parse_completion)
                    .unwrap_or_default();
                events.push(LspEvent::Completion {
                    request_id: id,
                    items,
                });
                return;
            }
        }

        let Some(method) = value.get("method").and_then(Value::as_str) else {
            return;
        };
        match method {
            "textDocument/publishDiagnostics" => {
                let params = value.get("params").cloned().unwrap_or(Value::Null);
                let uri = params.get("uri").and_then(Value::as_str).unwrap_or("");
                if uri != self.uri {
                    return;
                }
                let diagnostics = parse_diagnostics(&params, &self.lines);
                events.push(LspEvent::Diagnostics(diagnostics));
            }
            _ => {
                if let Some(id) = value.get("id").and_then(Value::as_i64) {
                    let result = if method == "workspace/configuration" {
                        let count = value
                            .get("params")
                            .and_then(|params| params.get("items"))
                            .and_then(Value::as_array)
                            .map(Vec::len)
                            .unwrap_or(0);
                        Value::Array(vec![Value::Null; count])
                    } else if method == "workspace/applyEdit" {
                        json!({"applied": false})
                    } else {
                        Value::Null
                    };
                    self.send_response(id, result).ok();
                }
            }
        }
    }

    fn send_initialize(&mut self) -> io::Result<()> {
        let root_uri = path_to_uri(&self.root);
        let params = json!({
            "processId": std::process::id(),
            "clientInfo": {
                "name": "emacette",
                "version": env!("CARGO_PKG_VERSION")
            },
            "rootUri": root_uri,
            "rootPath": self.root.display().to_string(),
            "capabilities": {
                "textDocument": {
                    "synchronization": {
                        "dynamicRegistration": false,
                        "didSave": false
                    },
                    "completion": {
                        "dynamicRegistration": false,
                        "completionItem": {
                            "snippetSupport": false,
                            "documentationFormat": ["plaintext"]
                        }
                    },
                    "publishDiagnostics": {
                        "relatedInformation": false,
                        "versionSupport": true
                    }
                },
                "workspace": {
                    "configuration": false,
                    "workspaceFolders": false
                }
            },
            "workspaceFolders": Value::Null
        });
        self.send_request(self.init_id, "initialize", params)
    }

    fn send_did_open(&mut self) {
        let params = json!({
            "textDocument": {
                "uri": self.uri,
                "languageId": self.language_id,
                "version": self.version,
                "text": self.text
            }
        });
        if self
            .send_notification("textDocument/didOpen", params)
            .is_ok()
        {
            self.opened = true;
        }
    }

    fn send_request(&mut self, id: i64, method: &str, params: Value) -> io::Result<()> {
        let message = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });
        write_message(&mut self.stdin, &message)
    }

    fn send_notification(&mut self, method: &str, params: Value) -> io::Result<()> {
        let message = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        write_message(&mut self.stdin, &message)
    }

    fn send_response(&mut self, id: i64, result: Value) -> io::Result<()> {
        let message = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        });
        write_message(&mut self.stdin, &message)
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = write_message(
            &mut self.stdin,
            &json!({
                "jsonrpc": "2.0",
                "id": self.next_id,
                "method": "shutdown",
                "params": Value::Null
            }),
        );
        let _ = write_message(
            &mut self.stdin,
            &json!({
                "jsonrpc": "2.0",
                "method": "exit",
                "params": Value::Null
            }),
        );
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn reader_loop(stdout: std::process::ChildStdout, tx: mpsc::Sender<ReaderEvent>) {
    let mut reader = BufReader::new(stdout);
    loop {
        let mut content_length = None;
        loop {
            let mut header = String::new();
            match reader.read_line(&mut header) {
                Ok(0) | Err(_) => return,
                Ok(_) => {}
            }
            let line = header.trim_end_matches(['\r', '\n']);
            if line.is_empty() {
                break;
            }
            if let Some(value) = line
                .strip_prefix("Content-Length:")
                .and_then(|value| value.trim().parse::<usize>().ok())
            {
                content_length = Some(value);
            }
        }
        let Some(length) = content_length else {
            continue;
        };
        let mut body = vec![0u8; length];
        if reader.read_exact(&mut body).is_err() {
            return;
        }
        match serde_json::from_slice::<Value>(&body) {
            Ok(value) => {
                if tx.send(ReaderEvent::Message(value)).is_err() {
                    return;
                }
            }
            Err(error) => {
                let _ = tx.send(ReaderEvent::Error(format!("LSP JSON error: {error}")));
                return;
            }
        }
    }
}

fn write_message(writer: &mut impl Write, value: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}

pub fn split_command(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    for c in input.chars() {
        if escaped {
            current.push(c);
            escaped = false;
            continue;
        }
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                } else if c == '\\' && q == '"' {
                    escaped = true;
                } else {
                    current.push(c);
                }
            }
            None => {
                if matches!(c, '"' | '\'') {
                    quote = Some(c);
                } else if c == '\\' {
                    escaped = true;
                } else if c.is_whitespace() {
                    if !current.is_empty() {
                        parts.push(std::mem::take(&mut current));
                    }
                } else {
                    current.push(c);
                }
            }
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

fn plain_text(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

fn text_lines(text: &str) -> Vec<Vec<char>> {
    text.split('\n')
        .map(|line| line.chars().collect())
        .collect()
}

fn path_to_uri(path: &Path) -> String {
    let mut uri = String::from("file://");
    for byte in path.to_string_lossy().as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_' | b'.' | b'~') {
            uri.push(*byte as char);
        } else {
            uri.push_str(&format!("%{byte:02X}"));
        }
    }
    uri
}

fn utf16_to_char_col(line: &[char], units: usize) -> usize {
    let mut width = 0usize;
    for (index, c) in line.iter().enumerate() {
        if width >= units {
            return index;
        }
        width += c.len_utf16();
    }
    line.len()
}

fn char_col_to_utf16(line: &[char], col: usize) -> usize {
    line.iter()
        .take(col.min(line.len()))
        .map(|c| c.len_utf16())
        .sum()
}

fn parse_diagnostics(params: &Value, lines: &[Vec<char>]) -> Vec<Diagnostic> {
    let Some(items) = params.get("diagnostics").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut diagnostics = Vec::new();
    for item in items {
        let Some(start) = item.get("range").and_then(|range| range.get("start")) else {
            continue;
        };
        let Some(end) = item.get("range").and_then(|range| range.get("end")) else {
            continue;
        };
        let start_row = start.get("line").and_then(Value::as_u64).unwrap_or(0) as usize;
        let end_row = (end.get("line").and_then(Value::as_u64).unwrap_or(0) as usize)
            .min(lines.len().saturating_sub(1));
        let start_character = start.get("character").and_then(Value::as_u64).unwrap_or(0) as usize;
        let end_character = end.get("character").and_then(Value::as_u64).unwrap_or(0) as usize;
        let severity = match item.get("severity").and_then(Value::as_u64) {
            Some(1) => Severity::Error,
            Some(3) => Severity::Information,
            Some(4) => Severity::Hint,
            _ => Severity::Warning,
        };
        let message = plain_text(item.get("message").and_then(Value::as_str).unwrap_or(""));
        let source = item.get("source").and_then(Value::as_str).map(plain_text);
        for row in start_row..=end_row {
            let Some(line) = lines.get(row) else {
                break;
            };
            let start_col = if row == start_row {
                utf16_to_char_col(line, start_character)
            } else {
                0
            };
            let end_col = if row == end_row {
                utf16_to_char_col(line, end_character)
            } else {
                line.len()
            };
            diagnostics.push(Diagnostic {
                row,
                start: start_col.min(end_col),
                end: end_col.max(start_col),
                severity,
                message: message.clone(),
                source: source.clone(),
            });
        }
    }
    diagnostics.sort_by_key(|diagnostic| (diagnostic.row, diagnostic.start));
    diagnostics
}

fn parse_completion(result: &Value) -> Vec<String> {
    let items = if let Some(items) = result.as_array() {
        items
    } else if let Some(items) = result.get("items").and_then(Value::as_array) {
        items
    } else {
        return Vec::new();
    };
    let mut words = Vec::new();
    for item in items {
        let Some(label) = item.get("label").and_then(Value::as_str) else {
            continue;
        };
        let text_edit = item
            .get("textEdit")
            .and_then(|edit| edit.get("newText"))
            .and_then(Value::as_str);
        let insert_text = item.get("insertText").and_then(Value::as_str);
        let mut text = text_edit.or(insert_text).unwrap_or(label);
        if text.contains(['\n', '\r', '$']) {
            text = label;
        }
        let text = plain_text(text);
        if text.is_empty() || words.contains(&text) {
            continue;
        }
        words.push(text);
        if words.len() >= 200 {
            break;
        }
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_commands_with_quotes() {
        assert_eq!(
            split_command("rust-analyzer --stdio"),
            vec!["rust-analyzer", "--stdio"]
        );
        assert_eq!(
            split_command("server --log '/tmp/a b' \"--root=/tmp/c d\""),
            vec!["server", "--log", "/tmp/a b", "--root=/tmp/c d"]
        );
    }

    #[test]
    fn converts_utf16_diagnostic_positions() {
        let lines = vec![vec!['a', 'é', 'b', '😀', 'c']];
        let params = json!({
            "diagnostics": [{
                "range": {
                    "start": {"line": 0, "character": 1},
                    "end": {"line": 0, "character": 5}
                },
                "severity": 1,
                "message": "bad\nthing",
                "source": "test"
            }]
        });
        let diagnostics = parse_diagnostics(&params, &lines);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].row, 0);
        assert_eq!(diagnostics[0].start, 1);
        assert_eq!(diagnostics[0].end, 4);
        assert_eq!(diagnostics[0].severity, Severity::Error);
        assert_eq!(diagnostics[0].message, "bad thing");
    }

    #[test]
    fn extracts_completion_items() {
        let result = json!({
            "items": [
                {"label": "alpha", "insertText": "alpha"},
                {"label": "beta", "textEdit": {"newText": "beta"}},
                {"label": "snippet", "insertText": "snippet($1)"},
                {"label": "alpha"}
            ]
        });
        assert_eq!(
            parse_completion(&result),
            vec![
                "alpha".to_string(),
                "beta".to_string(),
                "snippet".to_string()
            ]
        );
    }

    #[test]
    fn file_uri_percent_encodes_spaces() {
        assert_eq!(
            path_to_uri(Path::new("/tmp/a b/é.rs")),
            "file:///tmp/a%20b/%C3%A9.rs"
        );
    }
}
