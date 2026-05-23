#[derive(Debug, Clone)]
pub struct TrayItem {
    pub bus_name: String,
    pub object_path: String,
    pub id: String,
    pub title: String,
    pub status: String,
    pub icon: TrayIcon,
    pub menu_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TrayIcon {
    Path(std::path::PathBuf),
    Pixmap { w: u32, h: u32, rgba: Vec<u8> },
    None,
}

pub fn parse_register_service(service: &str, sender: &str) -> (String, String) {
    if service.starts_with('/') {
        (sender.to_string(), service.to_string())
    } else {
        (service.to_string(), "/StatusNotifierItem".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_service_bus_name_form() {
        let (bus, path) = parse_register_service("org.kde.StatusNotifierItem-1-1", ":1.5");
        assert_eq!(bus, "org.kde.StatusNotifierItem-1-1");
        assert_eq!(path, "/StatusNotifierItem");
    }

    #[test]
    fn register_service_unique_name_form() {
        let (bus, path) = parse_register_service(":1.42", ":1.42");
        assert_eq!(bus, ":1.42");
        assert_eq!(path, "/StatusNotifierItem");
    }

    #[test]
    fn register_service_object_path_form() {
        let (bus, path) = parse_register_service("/org/ayatana/NotificationItem/app", ":1.7");
        assert_eq!(bus, ":1.7");
        assert_eq!(path, "/org/ayatana/NotificationItem/app");
    }
}
