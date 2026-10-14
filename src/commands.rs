use crate::utils::expand_path;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io;
use std::io::Write;

pub fn is_file_exists(path: &str) -> bool {
    let expanded_path = expand_path(path);
    match fs::metadata(&expanded_path) {
        Ok(metadata) => metadata.is_file(),
        Err(_) => false,
    }
}
pub fn create_file(path: &str) -> Result<bool, io::Error> {
    let expanded_path = expand_path(path);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&expanded_path)?;

    file.write_all(b"{}")?;
    Ok(true)
}

fn load_shortcuts(path: &str) -> io::Result<HashMap<String, String>> {
    let expanded_path = expand_path(path);
    let content = fs::read_to_string(&expanded_path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("Failed to read config file '{expanded_path}': {e}"),
        )
    })?;
    serde_json::from_str(&content).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid JSON in config file '{expanded_path}': {e}"),
        )
    })
}

fn save_shortcuts(path: &str, shortcuts: &HashMap<String, String>) -> io::Result<()> {
    let expanded_path = expand_path(path);
    let json = serde_json::to_string_pretty(shortcuts).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to serialize shortcuts: {e}"),
        )
    })?;
    fs::write(&expanded_path, json).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("Failed to write config file '{expanded_path}': {e}"),
        )
    })
}
pub fn add_shortcut<W: Write>(
    path: &str,
    alias: &str,
    cmd: &str,
    writer: &mut W,
) -> io::Result<()> {
    let mut shortcuts = load_shortcuts(path)?;

    shortcuts.insert(alias.to_string(), cmd.to_string());
    save_shortcuts(path, &shortcuts)?;
    writeln!(writer, "Added shortcut: {alias} -> {cmd}")?;
    Ok(())
}
pub fn remove_shortcut<W: Write>(path: &str, alias: &str, writer: &mut W) -> io::Result<()> {
    let mut shortcuts = load_shortcuts(path)?;
    if shortcuts.remove(alias).is_some() {
        save_shortcuts(path, &shortcuts)?;
        writeln!(writer, "Removed shortcut: {alias}")?;
    } else {
        writeln!(writer, "Shortcut not found: {alias}")?;
    }
    Ok(())
}
pub fn list_shortcuts<W: Write>(path: &str, writer: &mut W) -> io::Result<()> {
    let shortcuts = load_shortcuts(path)?;

    if shortcuts.is_empty() {
        writeln!(writer, "No shortcuts found.")?;
    } else {
        writeln!(writer, "Listing all shortcuts:")?;
        let mut keys: Vec<&String> = shortcuts.keys().collect();
        keys.sort();
        // todo: remove for statement
        for key in keys {
            if let Some(value) = shortcuts.get(key) {
                writeln!(writer, "  {key} -> {value}")?;
            }
        }
    }
    Ok(())
}
pub fn show_shortcut<W: Write>(path: &str, key: &str, writer: &mut W) -> io::Result<()> {
    let shortcuts = load_shortcuts(path)?;
    if let Some(value) = shortcuts.get(key) {
        writeln!(writer, "{value}")?;
    } else {
        writeln!(writer, "Shortcut not found: {key}")?;
    }
    Ok(())
}

/// Quota un singolo argomento per l'uso dentro `sh -c`.
pub fn shell_escape(arg: &str) -> String {
    const SAFE: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_@%+=:,./-";
    if !arg.is_empty() && arg.bytes().all(|b| SAFE.contains(&b)) {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', "'\"'\"'"))
    }
}

/// Sostituisce ogni `{{}}` nel template con gli argomenti extra in ordine.
/// Gli argomenti in eccesso vengono accodati in coda.
/// Ritorna il comando renderizzato e il numero di placeholder senza argomento.
pub fn render_command(template: &str, extra: &[String]) -> (String, usize) {
    let mut rendered = String::with_capacity(template.len());
    let mut args = extra.iter();
    let mut missing = 0;
    let mut rest = template;
    while let Some(pos) = rest.find("{{}}") {
        rendered.push_str(&rest[..pos]);
        match args.next() {
            Some(arg) => rendered.push_str(&shell_escape(arg)),
            None => {
                rendered.push_str("{{}}");
                missing += 1;
            }
        }
        rest = &rest[pos + "{{}}".len()..];
    }
    rendered.push_str(rest);
    for arg in args {
        rendered.push(' ');
        rendered.push_str(&shell_escape(arg));
    }
    (rendered, missing)
}
pub fn execute_shortcut<W: Write>(path: &str, args: Vec<String>, writer: &mut W) -> io::Result<()> {
    if args.is_empty() {
        return Ok(());
    }
    let key = &args[0];
    let shortcuts = load_shortcuts(path)?;

    // todo invert if
    if let Some(template) = shortcuts.get(key) {
        let (command_str, missing) = render_command(template, &args[1..]);
        writeln!(writer, "Executing: {command_str}")?;
        if missing > 0 {
            writeln!(writer, "Warning: {missing} placeholder(s) without argument")?;
        }

        // todo current shell
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(&command_str)
            .status()
            .map_err(|e| {
                io::Error::new(
                    e.kind(),
                    format!("Failed to execute command '{command_str}': {e}"),
                )
            })?;
        if !status.success() {
            writeln!(writer, "Command failed with status: {status}")?;
        }
    } else {
        writeln!(writer, "Shortcut not found: {key}")?;
    }
    Ok(())
}
