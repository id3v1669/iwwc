use std::path::PathBuf;

pub fn effective_theme(icon_theme: Option<&str>) -> String {
    icon_theme
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .or_else(freedesktop_icons::default_theme_gtk)
        .unwrap_or_else(|| "hicolor".to_string())
}

pub fn lookup_named(name: &str, size: u16, theme: &str) -> Option<PathBuf> {
    freedesktop_icons::lookup(name)
        .with_size(size)
        .with_theme(theme)
        .with_cache()
        .find()
}
