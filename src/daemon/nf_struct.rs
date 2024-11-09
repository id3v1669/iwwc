#[derive(Debug, Clone)]
pub struct Notification {
    pub app_name: String,
    pub app_icon: String,
    pub replaces_id: u32,
    pub summary: String,
    pub body: String,
    pub actions: Vec<String>,
    pub expire_timeout: i32,
    pub notification_id: u32,
    pub desktop_entry: String,
}

#[derive(Debug)]
pub enum NotificationAction {
    ActionClose { notification_id: u32, reason: u32 },
    ActionInvoked { notification_id: u32 },
    Notify { notification: Notification },
    Close { notification_id: u32 },
}
