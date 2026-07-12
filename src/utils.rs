use std::env;

pub fn get_current_working_dir() -> String {
    let cwd = env::current_dir().unwrap();
    String::from(cwd.to_string_lossy())
}

pub fn expand_path(path: &str) -> String {
    if path.starts_with("~") {
        let home = env::var("HOME").expect("HOME environment variable not set");
        path.replacen("~", &home, 1)
    } else {
        path.to_string()
    }
}
pub fn resolve_config_path(global: bool) -> String {
    if global {
        String::from("~/.mm_global")
    } else {
        get_current_working_dir() + "/.mm"
    }
}
