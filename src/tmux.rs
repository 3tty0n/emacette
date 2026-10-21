//! tmux integration: give emacette's key bindings priority inside a pane.
//!
//! tmux handles its root and prefix key tables before forwarding input to the
//! pane.  A root binding such as `C-y` (often used for clipboard paste) would
//! otherwise make the editor's own `C-y` unreachable.  While emacette runs we
//! temporarily wrap root-table bindings so that they are forwarded to the
//! active pane when that pane is running emacette, and leave the original
//! binding in effect elsewhere.  The original bindings and prefix settings are
//! restored when the editor exits.

use std::env;
use std::process::{Command, Output};

const MARKER: &str = "emacette-key-forward";
const FIELD_SEPARATOR: char = '\u{1f}';

#[derive(Clone, Debug, PartialEq, Eq)]
struct RootBinding {
    key: String,
    repeat: bool,
    note: String,
    command: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SavedPrefix {
    option: &'static str,
    local_value: Option<String>,
}

#[derive(Debug, Default)]
pub struct KeyGuard {
    bindings: Vec<RootBinding>,
    prefixes: Vec<SavedPrefix>,
}

impl KeyGuard {
    /// Install temporary forwarding bindings when running inside tmux.
    ///
    /// Failures are deliberately ignored: tmux may be unavailable from the
    /// pane, or the server may reject a particular binding, and that must not
    /// prevent the editor from starting.
    pub fn install() -> Self {
        let mut guard = Self::default();
        if env::var_os("TMUX").is_none() || keys_disabled() {
            return guard;
        }

        if let Some(pane_command) = current_pane_command() {
            if !is_current_process(&pane_command) {
                return guard;
            }
            if let Some(bindings) = root_bindings() {
                let condition = forwarding_condition(&pane_command);
                for binding in bindings {
                    // A crashed previous instance may have left a forwarding
                    // binding behind.  It already does the right thing, so do
                    // not wrap it again.
                    if binding.command.contains(MARKER) {
                        continue;
                    }
                    let forward = forward_command(&binding.key);
                    let mut args = vec!["bind-key"];
                    if binding.repeat {
                        args.push("-r");
                    }
                    args.extend([
                        "-n",
                        binding.key.as_str(),
                        "if-shell",
                        "-F",
                        condition.as_str(),
                        forward.as_str(),
                        binding.command.as_str(),
                    ]);
                    if tmux(&args).is_some() {
                        guard.bindings.push(binding);
                    }
                }
            }
        }

        for option in ["prefix", "prefix2"] {
            let Some((local_value, effective)) = option_value(option) else {
                continue;
            };
            if !prefix_conflicts_with_editor(&effective) {
                continue;
            }
            if tmux(&["set-option", option, "None"]).is_some() {
                guard.prefixes.push(SavedPrefix {
                    option,
                    local_value,
                });
            }
        }

        guard
    }
}

impl Drop for KeyGuard {
    fn drop(&mut self) {
        for binding in &self.bindings {
            let mut args = vec!["bind-key"];
            if binding.repeat {
                args.push("-r");
            }
            if !binding.note.is_empty() {
                args.extend(["-N", binding.note.as_str()]);
            }
            args.extend(["-n", binding.key.as_str(), binding.command.as_str()]);
            let _ = tmux(&args);
        }
        for prefix in &self.prefixes {
            match &prefix.local_value {
                Some(value) => {
                    let _ = tmux(&["set-option", prefix.option, value]);
                }
                None => {
                    let _ = tmux(&["set-option", "-u", prefix.option]);
                }
            }
        }
    }
}

fn keys_disabled() -> bool {
    match env::var("EMACETTE_TMUX_KEYS") {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "0" | "off" | "false" | "no"
        ),
        Err(_) => false,
    }
}

fn tmux(args: &[&str]) -> Option<Output> {
    let output = Command::new("tmux").args(args).output().ok()?;
    output.status.success().then_some(output)
}

fn tmux_text(args: &[&str]) -> Option<String> {
    let output = tmux(args)?;
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn current_pane_command() -> Option<String> {
    let command = tmux_text(&["display-message", "-p", "#{pane_current_command}"])?;
    (!command.is_empty()).then_some(command)
}

fn is_current_process(pane_command: &str) -> bool {
    let mut names = Vec::new();
    if let Ok(exe) = env::current_exe() {
        if let Some(name) = exe.file_name() {
            names.push(name.to_string_lossy().into_owned());
        }
    }
    if let Some(arg0) = env::args().next() {
        if let Some(name) = std::path::Path::new(&arg0).file_name() {
            names.push(name.to_string_lossy().into_owned());
        }
    }
    names.iter().any(|name| name == pane_command)
}

fn root_bindings() -> Option<Vec<RootBinding>> {
    let format = format!(
        "#{{key_string}}{sep}#{{key_repeat}}{sep}#{{key_note}}{sep}#{{key_command}}",
        sep = FIELD_SEPARATOR
    );
    let output = tmux_text(&["list-keys", "-T", "root", "-F", &format])?;
    Some(parse_root_bindings(&output))
}

fn parse_root_bindings(output: &str) -> Vec<RootBinding> {
    output
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(4, FIELD_SEPARATOR);
            let key = fields.next()?.to_string();
            let repeat = fields.next()? == "1";
            let note = fields.next()?.to_string();
            let command = fields.next()?.to_string();
            if key.is_empty() || is_mouse_key(&key) {
                return None;
            }
            Some(RootBinding {
                key,
                repeat,
                note,
                command,
            })
        })
        .collect()
}

fn is_mouse_key(key: &str) -> bool {
    ["Mouse", "Wheel", "Click", "Drag"]
        .iter()
        .any(|part| key.contains(part))
}

fn forwarding_condition(pane_command: &str) -> String {
    format!("#{{&&:#{{==:#{{pane_current_command}},{pane_command}}},#{{==:{MARKER},{MARKER}}}}}")
}

fn forward_command(key: &str) -> String {
    if key == "Any" {
        // With no explicit key, send-keys forwards the key that triggered
        // the binding.
        "send-keys".to_string()
    } else {
        format!("send-keys {key}")
    }
}

/// Read the effective value of a session option and remember whether it was
/// set locally (rather than inherited from the global value).
fn option_value(name: &str) -> Option<(Option<String>, String)> {
    let local = tmux_text(&["show-options", "-v", name]).unwrap_or_default();
    let global = tmux_text(&["show-options", "-gv", name]).unwrap_or_default();
    let effective = if local.is_empty() {
        global
    } else {
        local.clone()
    };
    if effective.is_empty() {
        return None;
    }
    Some(((!local.is_empty()).then_some(local), effective))
}

fn normalize_key(key: &str) -> String {
    let key = key.trim().to_ascii_lowercase();
    let key = key.replace("c-spc", "c-space").replace("c-@", "c-space");
    match key.as_str() {
        "c-i" => "tab".to_string(),
        "c-m" | "return" | "enter" => "enter".to_string(),
        "c-[" | "esc" => "escape".to_string(),
        "c-?" | "bs" | "bspace" => "backspace".to_string(),
        "pgup" => "pageup".to_string(),
        "pgdn" => "pagedown".to_string(),
        _ => key,
    }
}

/// Keys handled by the editor without first pressing a prefix.  A tmux prefix
/// using one of them must be disabled while emacette is active, or the key
/// never reaches the editor.
const EDITOR_KEYS: &[&str] = &[
    "c-a",
    "c-b",
    "c-c",
    "c-d",
    "c-e",
    "c-f",
    "c-g",
    "c-h",
    "c-k",
    "c-l",
    "c-n",
    "c-o",
    "c-p",
    "c-r",
    "c-s",
    "c-t",
    "c-v",
    "c-w",
    "c-x",
    "c-y",
    "c-space",
    "c-/",
    "c-_",
    "m-<",
    "m->",
    "m-b",
    "m-c",
    "m-d",
    "m-f",
    "m-g",
    "m-l",
    "m-u",
    "m-v",
    "m-w",
    "m-x",
    "m-y",
    "m-;",
    "m-$",
    "m-left",
    "m-right",
    "up",
    "down",
    "left",
    "right",
    "home",
    "end",
    "pageup",
    "pagedown",
    "tab",
    "btab",
    "s-tab",
    "backspace",
    "delete",
    "escape",
    "enter",
    "f1",
];

fn prefix_conflicts_with_editor(prefix: &str) -> bool {
    EDITOR_KEYS.contains(&normalize_key(prefix).as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_root_bindings_and_skips_mouse_keys() {
        let sep = FIELD_SEPARATOR;
        let output = format!(
            "C-y{sep}0{sep}{sep}run-shell paste\n\
             MouseDown1Pane{sep}0{sep}{sep}select-pane\n\
             C-x{sep}1{sep}a note{sep}kill-pane"
        );
        let bindings = parse_root_bindings(&output);
        assert_eq!(
            bindings,
            vec![
                RootBinding {
                    key: "C-y".into(),
                    repeat: false,
                    note: String::new(),
                    command: "run-shell paste".into(),
                },
                RootBinding {
                    key: "C-x".into(),
                    repeat: true,
                    note: "a note".into(),
                    command: "kill-pane".into(),
                },
            ]
        );
    }

    #[test]
    fn detects_editor_prefix_conflicts() {
        assert!(prefix_conflicts_with_editor("C-b"));
        assert!(prefix_conflicts_with_editor("C-Space"));
        assert!(prefix_conflicts_with_editor("C-i"));
        assert!(prefix_conflicts_with_editor("M-x"));
        assert!(!prefix_conflicts_with_editor("C-q"));
        assert!(!prefix_conflicts_with_editor("None"));
    }

    #[test]
    fn forwarding_condition_contains_a_marker() {
        let condition = forwarding_condition("emacette");
        assert!(condition.contains("emacette"));
        assert!(condition.contains(MARKER));
    }

    #[test]
    fn any_binding_forwards_the_pressed_key() {
        assert_eq!(forward_command("Any"), "send-keys");
        assert_eq!(forward_command("C-y"), "send-keys C-y");
    }
}
