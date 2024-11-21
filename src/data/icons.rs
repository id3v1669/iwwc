use std::io::Write;

fn default_icon() {
    const DEFAULT_ICON: &[u8] = include_bytes!("../../assets/testing/default.svg");

    let path = std::env::var("HOME").unwrap() + "/.config/rs-nc";
    if !std::path::Path::new(&path).exists() {
        if let Err(e) = std::fs::create_dir_all(&path) {
            log::error!("Failed to create a default icon directory: {}", e);
            std::process::exit(1);
        }
    }
    let default_icon = path.clone() + "/default.svg";
    if !std::path::Path::new(&default_icon).exists() {
        if let Err(e) = std::fs::write(default_icon, DEFAULT_ICON) {
            log::error!("Failed to create a default icon: {}", e);
            std::process::exit(1);
        }
    } else {
        log::info!("Default icon already exists");
    }
}

pub fn get_system_icons_paths() -> std::collections::HashMap<String, std::path::PathBuf> {
    let mut icons = std::collections::HashMap::new();
    default_icon();
    let home_dir = std::env::var("HOME").unwrap();
    let icon_theme_name = {
        if std::path::Path::new(&(home_dir.clone() + "/.config/gtk-4.0/settings.ini")).exists() {
            let settings =
                std::fs::read_to_string(home_dir + "/.config/gtk-4.0/settings.ini").unwrap();
            let icon_theme = settings
                .lines()
                .find(|line| line.starts_with("gtk-icon-theme-name"))
                .unwrap();
            icon_theme.split('=').collect::<Vec<&str>>()[1]
                .trim()
                .to_string()
        } else if std::path::Path::new(&(home_dir.clone() + "/.config/gtk-3.0/settings.ini"))
            .exists()
        {
            let settings =
                std::fs::read_to_string(home_dir + "/.config/gtk-3.0/settings.ini").unwrap();
            let icon_theme = settings
                .lines()
                .find(|line| line.starts_with("gtk-icon-theme-name"))
                .unwrap();
            icon_theme.split('=').collect::<Vec<&str>>()[1]
                .trim()
                .to_string()
        } else {
            "Adwaita".to_string()
        }
    };
    let data_dirs = std::env::var("XDG_DATA_DIRS").unwrap_or("/usr/share".to_string());
    let data_dirs = data_dirs.split(':').collect::<Vec<&str>>();
    let mut icons_dir = None;
    for dir in data_dirs {
        if std::path::Path::new(&(dir.to_string() + "/icons/" + &icon_theme_name)).exists() {
            icons_dir = Some(dir.to_string() + "/icons/" + &icon_theme_name);
            break;
        }
    }
    if icons_dir.is_none() {
        log::warn!(
            "Icon theme {} not found, default icons will be used everywhere",
            icon_theme_name
        );
        return icons;
    }

    if let Some(folders) = find_folders_recursively(&icons_dir.unwrap().into(), "apps") {
        for folder in folders {
            find_icons_recursively(&folder.into()).map(|mut i| icons.extend(i));
        }
    }

    return icons;
}

fn find_folders_recursively(
    init_path: &std::path::PathBuf,
    folder_name: &str,
) -> Option<Vec<std::path::PathBuf>> {
    let mut folders: Vec<std::path::PathBuf> = vec![];
    for entry in std::fs::read_dir(init_path).unwrap() {
        let entry = entry.unwrap();
        let mut path = entry.path();
        if path.is_symlink() {
            path = std::fs::canonicalize(path).unwrap();
        }
        if path.is_dir() {
            if path.file_name().unwrap().to_str().unwrap() == folder_name {
                folders.push(path);
            } else {
                if let Some(mut sub_folders) = find_folders_recursively(&path, folder_name) {
                    folders.append(&mut sub_folders);
                }
            }
        }
    }
    if folders.is_empty() {
        None
    } else {
        Some(folders)
    }
}

fn find_icons_recursively(
    init_path: &std::path::PathBuf,
) -> Option<std::collections::HashMap<String, std::path::PathBuf>> {
    let mut icons = std::collections::HashMap::new();
    for entry in std::fs::read_dir(init_path).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "svg" {
                    icons.insert(
                        path.file_stem().unwrap().to_str().unwrap().to_string(),
                        path,
                    );
                }
            }
        } else if entry.file_type().unwrap().is_dir() {
            if let Some(mut sub_icons) = find_icons_recursively(&entry.path()) {
                icons.extend(sub_icons);
            }
        }
    }
    if icons.is_empty() {
        None
    } else {
        Some(icons)
    }
}
