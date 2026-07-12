use crate::utils::expand_path;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io;
use std::io::Write;

pub fn is_file_exists(path: &str) -> Result<bool, std::io::Error> {
    let expanded_path = expand_path(path);
    match fs::metadata(&expanded_path) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e),
    }
}
pub fn create_file(path: &str) -> Result<(bool), std::io::Error> {
    let expanded_path = expand_path(path);
    let result = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&expanded_path);

    match result {
        Ok(mut file) => {
            file.write_all(b"{}")?;
        }
        Err(e) => {
            return Err(e);
        }
    }
    Ok(true)
}
pub fn add_shortcut<W: Write>(path: &str, alias: &str, cmd: &str, writer: &mut W) {
    let expanded_path = expand_path(path);
    let content = fs::read_to_string(&expanded_path).expect("Failed to read config file");
    let mut shortcuts: HashMap<String, String> = serde_json::from_str(&content).unwrap_or_default();

    shortcuts.insert(alias.to_string(), cmd.to_string());
    if let Ok(json) = serde_json::to_string_pretty(&shortcuts) {
        if fs::write(&expanded_path, json).is_ok() {
            writeln!(writer, "Added shortcut: {} -> {}", alias, cmd).ok();
        } else {
            writeln!(writer, "Failed to write to config file").ok();
        }
    }
}

pub fn remove_shortcut<W: Write>(path: &str, alias: &str, writer: &mut W) {
    let expanded_path = expand_path(path);
    let content = fs::read_to_string(&expanded_path).expect("Failed to read config file");
    let mut shortcuts: HashMap<String, String> = serde_json::from_str(&content).unwrap_or_default();
    if shortcuts.remove(alias).is_some() {
        if let Ok(json) = serde_json::to_string_pretty(&shortcuts) {
            if fs::write(&expanded_path, json).is_ok() {
                writeln!(writer, "Removed shortcut: {}", alias).ok();
            } else {
                writeln!(writer, "Failed to write to config file").ok();
            }
        }
    } else {
        writeln!(writer, "Shortcut not found: {}", alias).ok();
    }
}

pub fn list_shortcuts<W: Write>(expanded_path: &str, writer: &mut W) {
    // todo: extract function
    let content = fs::read_to_string(expanded_path).expect("Failed to read config file");
    let shortcuts: HashMap<String, String> = serde_json::from_str(&content).unwrap_or_default();

    if shortcuts.is_empty() {
        writeln!(writer, "No shortcuts found.").ok();
    } else {
        writeln!(writer, "Listing all shortcuts:").ok();
        let mut keys: Vec<&String> = shortcuts.keys().collect();
        keys.sort();
        // todo: remove for statement
        for key in keys {
            if let Some(value) = shortcuts.get(key) {
                writeln!(writer, "  {} -> {}", key, value).ok();
            }
        }
    }
}

pub fn show_shortcut<W: Write>(path: &str, key: &str, writer: &mut W) {
    // todo: extract function
    let expanded_path = expand_path(path);
    let content = fs::read_to_string(&expanded_path).expect("Failed to read config file");
    let shortcuts: HashMap<String, String> = serde_json::from_str(&content).unwrap_or_default();
    if let Some(value) = shortcuts.get(key) {
        writeln!(writer, "{}", value).ok();
    } else {
        writeln!(writer, "Shortcut not found: {}", key).ok();
    }
}
pub fn list_keys_for_completion<W: Write>(path: &str, writer: &mut W) {
    let expanded_path = expand_path(path);
    if let Ok(content) = fs::read_to_string(&expanded_path) {
        if let Ok(shortcuts) = serde_json::from_str::<HashMap<String, String>>(&content) {
            let keys: Vec<&String> = shortcuts.keys().collect();
            let keys_str = keys
                .into_iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .join(" ")
                + " ";
            write!(writer, "{}", keys_str).ok();
        }
    }
}
pub fn execute_shortcut<W: Write>(path: &str, args: Vec<String>, writer: &mut W) {
    if args.is_empty() {
        return;
    }
    let key = &args[0];
    // todo: extract function
    let expanded_path = expand_path(path);
    let content = fs::read_to_string(&expanded_path).expect("Failed to read config file");
    let shortcuts: HashMap<String, String> = serde_json::from_str(&content).unwrap_or_default();

    // todo invert if
    if let Some(command_str) = shortcuts.get(key) {
        writeln!(writer, "Executing: {}", command_str).ok();

        // todo current shell
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(command_str)
            .status(); // .status() esegue il comando e aspetta il risultato
        match status {
            Ok(s) => {
                if !s.success() {
                    writeln!(writer, "Command failed with status: {}", s).ok();
                }
            }
            Err(e) => {
                writeln!(writer, "Failed to execute command: {}", e).ok();
            }
        }
    } else {
        writeln!(writer, "Shortcut not found: {}", key).ok();
    }
}
