use once_cell::sync::Lazy;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

pub static GLOBAL_DATA_MAP: Lazy<
    Arc<Mutex<HashMap<Option<String>, crate::data::nf_struct::Notification>>>,
> = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));
pub static ID_QUEUE: Lazy<Arc<Mutex<VecDeque<Option<String>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(VecDeque::new())));

//int ammount of active notifications
// pub static ACTIVE_NOTIFICATIONS: Lazy<Mutex<HashMap<i32, u32>>> = Lazy::new(|| Mutex::new(0));
pub static ACTIVE_NOTIFICATIONS: Lazy<Arc<Mutex<HashMap<i32, u32>>>> = Lazy::new(|| Arc::new(Mutex::new(HashMap::new()))); //first position, second id form Notification
// pub static NOTIFICATION_QUEUE: Lazy<Arc<Mutex<VecDeque<u32>>> = Lazy::new(|| Arc::new(Mutex::new(VecDeque::new())));


pub static GTK_ACTIVE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));

pub static CONFIG: Lazy<Mutex<crate::data::cfg_struct::Config>> =
    Lazy::new(|| Mutex::new(crate::data::cfg_struct::Config::default()));
