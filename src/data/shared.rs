use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub static ACTIVE_NOTIFICATIONS: Lazy<Arc<Mutex<HashMap<i32, u32>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new()))); //first position, second id form Notification

pub static CONFIG: Lazy<Mutex<crate::data::config::Config>> =
    Lazy::new(|| Mutex::new(crate::data::config::Config::read()));

pub static NVIDIA_SUCKS: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

pub static ICONS: Lazy<Mutex<HashMap<String, std::path::PathBuf>>> =
    Lazy::new(|| Mutex::new(crate::data::icons::get_system_icons_paths()));
