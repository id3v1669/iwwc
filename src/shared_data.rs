use once_cell::sync::Lazy;
use std::collections::{ VecDeque, HashMap };
use std::sync::{Mutex, Arc};


pub static GLOBAL_DATA_MAP: Lazy<Arc<Mutex<HashMap<Option<String>, crate::daemon::nf_struct::Notification>>>> =
  Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));
pub static ID_QUEUE: Lazy<Arc<Mutex<VecDeque<Option<String>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(VecDeque::new())));

//int ammount of active notifications
pub static ACTIVE_NOTIFICATIONS: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(0));

pub static GTK_ACTIVE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));