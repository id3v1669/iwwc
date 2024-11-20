use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub static ACTIVE_NOTIFICATIONS: Lazy<Arc<Mutex<HashMap<i32, u32>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new()))); //first position, second id form Notification

pub static CONFIG: Lazy<Mutex<crate::data::cfg_struct::Config>> =
    Lazy::new(|| Mutex::new(crate::data::cfg_struct::Config::default()));

pub static NVIDIA_SUCKS: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
