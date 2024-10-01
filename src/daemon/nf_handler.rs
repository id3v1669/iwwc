use zbus::interface;

pub struct NotificationHandler {
    count: u32,
    sender: tokio::sync::mpsc::Sender<crate::daemon::nf_struct::NotificationAction>,
}

impl NotificationHandler {
    pub fn new(
        sender: tokio::sync::mpsc::Sender<crate::daemon::nf_struct::NotificationAction>,
    ) -> Self {
        NotificationHandler { count: 0, sender }
    }
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationHandler {
    #[dbus_interface(name = "CloseNotification")]
    async fn close_notification(&mut self, notification_id: u32) -> zbus::fdo::Result<()> {
        self.sender
            .send(crate::daemon::nf_struct::NotificationAction::Close { notification_id })
            .await
            .map_err(crate::daemon::err_handler::ErrorHandler::from)?;
        Ok(())
    }

    #[dbus_interface(name = "Notify")]
    async fn notify(
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

        let notification = crate::daemon::nf_struct::Notification {
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

        self.sender
            .send(crate::daemon::nf_struct::NotificationAction::Notify { notification })
            .await
            .map_err(crate::daemon::err_handler::ErrorHandler::from)?;

        Ok(notification_id)
    }

    #[dbus_interface(
        out_args("name", "vendor", "version", "spec_version"),
        name = "GetServerInformation"
    )]
    fn get_server_information(&mut self) -> zbus::fdo::Result<(String, String, String, String)> {
        let name = std::env::var("CARGO_PKG_DESCRIPTION")
            .unwrap_or_else(|_| "No description found".to_string());
        let vendor =
            std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "No name found".to_string());
        let version =
            std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "No version found".to_string());
        let spec_version = String::from("1.2");

        Ok((name, vendor, version, spec_version))
    }

    #[dbus_interface(name = "GetCapabilities")]
    fn get_capabilities(&mut self) -> zbus::fdo::Result<Vec<&str>> {
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
