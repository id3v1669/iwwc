use crate::gui::app::Message;
use zbus::interface;

pub struct NotificationHandler {
    count: u32,
    sender: futures::channel::mpsc::Sender<Message>,
}

impl NotificationHandler {
    pub fn new(sender: futures::channel::mpsc::Sender<Message>) -> Self {
        NotificationHandler { count: 0, sender }
    }
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationHandler {
    #[allow(non_snake_case)]
    async fn CloseNotification(&mut self, notification_id: u32) -> zbus::fdo::Result<()> {
        self.sender
            .try_send(Message::CloseByContentId(notification_id))
            .ok();
        Ok(())
    }

    #[allow(non_snake_case, clippy::too_many_arguments)]
    async fn Notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: std::collections::HashMap<String, zbus::zvariant::Value<'_>>,
        expire_timeout: i32,
    ) -> zbus::fdo::Result<u32> {
        let notification_id = if replaces_id == 0 {
            self.count += 1;
            self.count
        } else {
            replaces_id
        };

        let desktop_entry = if hints.contains_key("desktop-entry") {
            hints["desktop-entry"].to_string()
        } else {
            String::new()
        };

        let notification = crate::data::notification::Notification {
            app_name,
            replaces_id,
            app_icon,
            summary,
            body,
            actions,
            expire_timeout,
            notification_id,
            desktop_entry,
        };

        self.sender.try_send(Message::Notify(notification)).ok();

        Ok(notification_id)
    }

    #[allow(non_snake_case)]
    fn GetServerInformation(&mut self) -> zbus::fdo::Result<(String, String, String, String)> {
        let name = std::env::var("CARGO_PKG_DESCRIPTION")
            .unwrap_or_else(|_| "No description found".to_string());
        let vendor =
            std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "No name found".to_string());
        let version =
            std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "No version found".to_string());
        let spec_version = String::from("1.2");

        Ok((name, vendor, version, spec_version))
    }

    #[allow(non_snake_case)]
    fn GetCapabilities(&mut self) -> zbus::fdo::Result<Vec<&str>> {
        let capabilities = vec![
            "action-icons",
            "actions",
            "body",
            "body-hyperlinks",
            "body-images",
            "body-markup",
            "icon-multi",
            "icon-static",
            "persistence",
            "sound",
        ];

        Ok(capabilities)
    }
}
