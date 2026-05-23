use std::path::{Path, PathBuf};

pub fn resolve_icon(app_icon: &str, image_path_hint: Option<&str>, size: u16) -> Option<PathBuf> {
    if let Some(hint) = image_path_hint {
        let raw = hint.strip_prefix("file://").unwrap_or(hint);
        let p = Path::new(raw);
        if p.is_file() {
            return Some(p.to_path_buf());
        }
    }
    if !app_icon.is_empty() {
        let raw = app_icon.strip_prefix("file://").unwrap_or(app_icon);
        let p = Path::new(raw);
        if p.is_absolute() && p.is_file() {
            return Some(p.to_path_buf());
        }
        return cosmic_freedesktop_icons::lookup(app_icon)
            .with_size(size)
            .with_cache()
            .find();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn image_path_hint_passthrough_when_exists() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("icon.png");
        fs::write(&p, b"x").unwrap();
        let got = resolve_icon("firefox", Some(p.to_str().unwrap()), 48);
        assert_eq!(got, Some(p));
    }

    #[test]
    fn absolute_app_icon_path_passthrough() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.svg");
        fs::write(&p, b"x").unwrap();
        let got = resolve_icon(p.to_str().unwrap(), None, 48);
        assert_eq!(got, Some(p));
    }

    #[test]
    fn file_uri_hint_is_stripped() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("b.png");
        fs::write(&p, b"x").unwrap();
        let uri = format!("file://{}", p.to_str().unwrap());
        assert_eq!(resolve_icon("x", Some(&uri), 48), Some(p));
    }

    #[test]
    fn unknown_name_returns_none() {
        assert_eq!(
            resolve_icon("definitely-not-an-icon-zzz-12345", None, 48),
            None
        );
    }
}
