use mm_rust::commands::{
    add_shortcut, create_file, execute_shortcut, is_file_exists, list_shortcuts, remove_shortcut,
    show_shortcut,
};
use mm_rust::utils::{expand_path, resolve_config_path};
use std::collections::HashMap;
use std::env;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_path() -> String {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut p = env::temp_dir();
    p.push(format!("mm_test_{}_{}.json", std::process::id(), id));
    p.to_string_lossy().into_owned()
}

fn cleanup(path: &str) {
    let _ = std::fs::remove_file(expand_path(path));
}

fn read_map(path: &str) -> HashMap<String, String> {
    let content = std::fs::read_to_string(expand_path(path)).unwrap();
    serde_json::from_str(&content).unwrap()
}

fn write_raw(path: &str, content: &str) {
    std::fs::write(expand_path(path), content).unwrap();
}

#[test]
fn create_file_writes_empty_map() {
    let path = temp_path();
    assert!(!is_file_exists(&path));
    assert!(create_file(&path).unwrap());
    assert!(is_file_exists(&path));
    assert_eq!(read_map(&path), HashMap::new());
    cleanup(&path);
}

#[test]
fn create_file_fails_if_exists() {
    let path = temp_path();
    create_file(&path).unwrap();
    assert!(create_file(&path).is_err());
    cleanup(&path);
}

#[test]
fn add_shortcut_persists_and_reports() {
    let path = temp_path();
    create_file(&path).unwrap();
    let mut out = Vec::new();
    add_shortcut(&path, "build", "cargo build", &mut out).unwrap();
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "Added shortcut: build -> cargo build\n"
    );
    assert_eq!(read_map(&path).get("build").unwrap(), "cargo build");

    // overwrite esistente
    let mut out2 = Vec::new();
    add_shortcut(&path, "build", "make", &mut out2).unwrap();
    assert_eq!(read_map(&path).get("build").unwrap(), "make");
    cleanup(&path);
}

#[test]
fn remove_shortcut_existing_and_missing() {
    let path = temp_path();
    create_file(&path).unwrap();
    let mut out = Vec::new();
    add_shortcut(&path, "a", "echo a", &mut out).unwrap();

    let mut rm = Vec::new();
    remove_shortcut(&path, "a", &mut rm).unwrap();
    assert_eq!(String::from_utf8(rm).unwrap(), "Removed shortcut: a\n");
    assert!(!read_map(&path).contains_key("a"));

    let mut missing = Vec::new();
    remove_shortcut(&path, "nope", &mut missing).unwrap();
    assert_eq!(
        String::from_utf8(missing).unwrap(),
        "Shortcut not found: nope\n"
    );
    cleanup(&path);
}

#[test]
fn list_shortcuts_empty_and_sorted() {
    let path = temp_path();
    create_file(&path).unwrap();

    let mut out = Vec::new();
    list_shortcuts(&path, &mut out).unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), "No shortcuts found.\n");

    let mut sink = Vec::new();
    add_shortcut(&path, "zeta", "z", &mut sink).unwrap();
    add_shortcut(&path, "alpha", "a", &mut sink).unwrap();
    add_shortcut(&path, "middle", "m", &mut sink).unwrap();

    let mut listed = Vec::new();
    list_shortcuts(&path, &mut listed).unwrap();
    assert_eq!(
        String::from_utf8(listed).unwrap(),
        "Listing all shortcuts:\n  alpha -> a\n  middle -> m\n  zeta -> z\n"
    );
    cleanup(&path);
}

#[test]
fn show_shortcut_found_and_missing() {
    let path = temp_path();
    create_file(&path).unwrap();
    let mut sink = Vec::new();
    add_shortcut(&path, "k", "v", &mut sink).unwrap();

    let mut out = Vec::new();
    show_shortcut(&path, "k", &mut out).unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), "v\n");

    let mut missing = Vec::new();
    show_shortcut(&path, "other", &mut missing).unwrap();
    assert_eq!(
        String::from_utf8(missing).unwrap(),
        "Shortcut not found: other\n"
    );
    cleanup(&path);
}

#[test]
fn execute_shortcut_empty_args_writes_nothing() {
    let path = temp_path();
    create_file(&path).unwrap();
    let mut out = Vec::new();
    execute_shortcut(&path, vec![], &mut out).unwrap();
    assert!(out.is_empty());
    cleanup(&path);
}

#[test]
fn execute_shortcut_unknown_alias() {
    let path = temp_path();
    create_file(&path).unwrap();
    let mut out = Vec::new();
    execute_shortcut(&path, vec!["nope".to_string()], &mut out).unwrap();
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "Shortcut not found: nope\n"
    );
    cleanup(&path);
}

#[test]
fn execute_shortcut_success_and_failure() {
    let path = temp_path();
    create_file(&path).unwrap();
    let mut sink = Vec::new();
    add_shortcut(&path, "ok", "true", &mut sink).unwrap();
    add_shortcut(&path, "ko", "false", &mut sink).unwrap();

    let mut out_ok = Vec::new();
    execute_shortcut(&path, vec!["ok".to_string()], &mut out_ok).unwrap();
    assert_eq!(String::from_utf8(out_ok).unwrap(), "Executing: true\n");

    let mut out_ko = Vec::new();
    execute_shortcut(&path, vec!["ko".to_string()], &mut out_ko).unwrap();
    let text = String::from_utf8(out_ko).unwrap();
    assert!(text.starts_with("Executing: false\nCommand failed with status: "));
    cleanup(&path);
}

#[test]
fn missing_config_returns_not_found_error() {
    let path = temp_path();
    assert!(!is_file_exists(&path));
    let mut out = Vec::new();
    let err = show_shortcut(&path, "k", &mut out).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
    assert!(out.is_empty());
}

#[test]
fn corrupt_config_returns_invalid_data_and_preserves_file() {
    let path = temp_path();
    create_file(&path).unwrap();
    write_raw(&path, "{ not valid json");
    let before = std::fs::read_to_string(expand_path(&path)).unwrap();

    let mut out = Vec::new();
    let err = show_shortcut(&path, "k", &mut out).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);

    let mut sink = Vec::new();
    let err = add_shortcut(&path, "k", "v", &mut sink).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);

    // nessun overwrite silenzioso del file corrotto
    let after = std::fs::read_to_string(expand_path(&path)).unwrap();
    assert_eq!(before, after);
    cleanup(&path);
}

#[test]
fn write_failure_returns_error() {
    // path che punta a una directory: la lettura fallisce o la scrittura fallisce
    let dir = env::temp_dir()
        .join(format!("mm_test_dir_{}", COUNTER.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.to_string_lossy().into_owned();

    let mut out = Vec::new();
    assert!(add_shortcut(&path, "k", "v", &mut out).is_err());
    assert!(list_shortcuts(&path, &mut out).is_err());

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn is_file_exists_false_for_directory() {
    let dir = env::temp_dir()
        .join(format!(
            "mm_test_isfile_{}",
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.to_string_lossy().into_owned();
    assert!(!is_file_exists(&path));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn expand_path_leaves_plain_paths_unchanged() {
    assert_eq!(expand_path("/tmp/foo"), "/tmp/foo");
    assert_eq!(expand_path("relative/path"), "relative/path");
}

#[test]
fn expand_path_expands_tilde_with_home() {
    let home = env::var("HOME").expect("HOME must be set for tests");
    assert_eq!(expand_path("~/.mm_global"), format!("{home}/.mm_global"));
}

#[test]
fn resolve_config_path_global_and_local() {
    assert_eq!(resolve_config_path(true), "~/.mm_global");
    let local = resolve_config_path(false);
    assert!(local.ends_with("/.mm"), "unexpected local path: {local}");
}
