use std::path::{Path, PathBuf};

// TODO: relpace with good icon on release
const DEFAULT_SVG: &[u8] = include_bytes!("../../assets/testing/default.svg");

pub fn default_icon_path(config_dir: &Path) -> PathBuf {
    config_dir.join("default.svg")
}

pub fn ensure_default_svg(config_dir: &Path) {
    let path = default_icon_path(config_dir);
    if !path.exists()
        && let Err(e) = std::fs::write(&path, DEFAULT_SVG)
    {
        log::warn!("could not write {}: {}", path.display(), e);
    }
}

pub fn resolve_icon(
    app_icon: &str,
    app_name: &str,
    image_path_hint: Option<&str>,
    size: u16,
    respect_icon: bool,
    icon_theme: Option<&str>,
    config_dir: &Path,
) -> PathBuf {
    let default = default_icon_path(config_dir);
    if !respect_icon {
        return default;
    }
    let theme = crate::iconlookup::effective_theme(icon_theme);
    if let Some(hint) = image_path_hint {
        let raw = hint.strip_prefix("file://").unwrap_or(hint);
        let p = Path::new(raw);
        if p.is_file() {
            return p.to_path_buf();
        }
        if let Some(found) = crate::iconlookup::lookup_named(raw, size, &theme) {
            return found;
        }
    }
    if !app_icon.is_empty() {
        let raw = app_icon.strip_prefix("file://").unwrap_or(app_icon);
        let p = Path::new(raw);
        if p.is_absolute() && p.is_file() {
            return p.to_path_buf();
        }
        if let Some(found) = crate::iconlookup::lookup_named(app_icon, size, &theme) {
            return found;
        }
    }
    let lowered = app_name.to_lowercase();
    if !lowered.is_empty()
        && let Some(p) = crate::iconlookup::lookup_named(&lowered, size, &theme)
    {
        return p;
    }
    default
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn respect_false_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let got = resolve_icon("firefox", "Firefox", None, 48, false, None, dir.path());
        assert_eq!(got, dir.path().join("default.svg"));
    }

    #[test]
    fn image_path_hint_passthrough_when_exists() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("icon.png");
        fs::write(&p, b"x").unwrap();
        let got = resolve_icon("firefox", "Firefox", Some(p.to_str().unwrap()), 48, true, None, dir.path());
        assert_eq!(got, p);
    }

    #[test]
    fn absolute_app_icon_path_passthrough() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.svg");
        fs::write(&p, b"x").unwrap();
        let got = resolve_icon(p.to_str().unwrap(), "", None, 48, true, None, dir.path());
        assert_eq!(got, p);
    }

    #[test]
    fn file_uri_hint_is_stripped() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("b.png");
        fs::write(&p, b"x").unwrap();
        let uri = format!("file://{}", p.to_str().unwrap());
        let got = resolve_icon("x", "", Some(&uri), 48, true, None, dir.path());
        assert_eq!(got, p);
    }

    #[test]
    fn unknown_name_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let got = resolve_icon(
            "definitely-not-an-icon-zzz-12345", "also-not-real-app-zzz",
            None, 48, true, None, dir.path());
        assert_eq!(got, dir.path().join("default.svg"));
    }

    #[test]
    fn ensure_default_svg_writes_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        ensure_default_svg(dir.path());
        let p = dir.path().join("default.svg");
        assert!(p.is_file());
        assert_eq!(std::fs::read(&p).unwrap(), DEFAULT_SVG);
    }

    #[test]
    fn ensure_default_svg_is_noop_when_present() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("default.svg");
        std::fs::write(&p, b"custom").unwrap();
        ensure_default_svg(dir.path());
        assert_eq!(std::fs::read(&p).unwrap(), b"custom");
    }

    #[test]
    fn default_icon_path_joins_config_dir() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(default_icon_path(dir.path()), dir.path().join("default.svg"));
    }
}