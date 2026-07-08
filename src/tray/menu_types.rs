#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: i32,
    pub label: String,
    pub enabled: bool,
    pub visible: bool,
    pub separator: bool,
    pub toggle: Toggle,
    pub icon: MenuIcon,
    pub has_submenu: bool,
    pub children: Vec<MenuItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Toggle {
    None,
    Check(bool),
    Radio(bool),
}

#[derive(Debug, Clone)]
pub enum MenuIcon {
    None,
    Name(String),
    File(std::path::PathBuf),
    Png(iced::widget::image::Handle),
}

pub fn resolve_icons(item: &mut MenuItem, size: u16) {
    if let MenuIcon::Name(name) = &item.icon {
        let p = std::path::Path::new(name);
        item.icon = if p.is_absolute() && p.is_file() {
            MenuIcon::File(p.to_path_buf())
        } else {
            match freedesktop_icons::lookup(name)
                .with_size(size)
                .with_cache()
                .find()
            {
                Some(found) => MenuIcon::File(found),
                None => MenuIcon::None,
            }
        };
    }
    for child in &mut item.children {
        resolve_icons(child, size);
    }
}

pub fn strip_mnemonic(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    let mut chars = label.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '_' {
            if chars.peek() == Some(&'_') {
                out.push('_');
                chars.next();
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_mnemonic_cases() {
        assert_eq!(strip_mnemonic("_File"), "File");
        assert_eq!(strip_mnemonic("Save __as"), "Save _as");
        assert_eq!(strip_mnemonic("No marker"), "No marker");
        assert_eq!(strip_mnemonic("a_b_c"), "abc");
    }
}
