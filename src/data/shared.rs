use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub static ICONS: Lazy<Mutex<HashMap<String, std::path::PathBuf>>> =
    Lazy::new(|| Mutex::new(crate::data::icons::get_system_icons_paths()));
