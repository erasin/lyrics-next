use std::path::Path;

pub fn normalize_text(s: &str) -> String {
    s.to_lowercase()
        .replace([' ', '_', '-', '(', ')', '（', '）'], "")
        .trim()
        .to_string()
}

pub fn ensure_parent_dir(path: &Path) {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).ok();
        }
    }
}
