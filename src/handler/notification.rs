use crate::gui::app::{Message};
use iced::platform_specific::shell::commands::layer_surface::{ KeyboardInteractivity, Layer };
use iced::Task;
use zbus::interface;

pub struct NotificationHandler {
    count: u32,
    sender: futures::channel::mpsc::Sender<Message>,
}

impl NotificationHandler {
    pub fn new(sender: futures::channel::mpsc::Sender<Message>) -> Self {
        NotificationHandler { count: 0, sender }
    }
    pub fn subscribe_iced() -> iced::Subscription<Message> {
        iced::Subscription::run_with_id(
                "notification-listener",
                iced::stream::channel(100, |sender| async move {
                    log::debug!("Notification listener started");
                    let builder = zbus::connection::Builder::session()
                        .unwrap()
                        .name("org.freedesktop.Notifications")
                        .unwrap()
                        .serve_at(
                            "/org/freedesktop/Notifications",
                            NotificationHandler::new(sender),
                        )
                        .unwrap();
                    let _connection = match builder.build().await {
                        Ok(connection) => connection,
                        Err(e) => {
                            log::error!("Failed to build the connection: {e}");
                            std::process::exit(1);
                        }
                    };
                    futures::future::pending::<()>().await;
                    unreachable!()
                })
            )
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

pub fn handle_notification(
    iwwc: &mut crate::gui::app::IcedWaylandWidgetCenter,
    notification: crate::data::notification::Notification,
) -> iced::Task<Message> {
    let mut overflow = iced::Task::none();
    let id = notification.notification_id;

    let window_id = iced::window::Id::unique();

    if iwwc.notification_ids.len() >= iwwc.config.notifications.max_notifications as usize 
        && let Some((_, info)) = iwwc.notification_ids.shift_remove_index(0) {
            overflow = iced::Task::done(Message::CloseByContentId(info.notification.notification_id));
        
    }

    let timeout = if iwwc.config.notifications.respect_notification_timeout
        && notification.expire_timeout > 0
    {
        notification.expire_timeout
    } else {
        iwwc.config.notifications.local_expire_timeout
    };

    let icons = crate::data::icons::ICONS.lock().unwrap();

    let icon_name = if !notification.app_icon.is_empty()
        && iwwc.config.notifications.respect_notification_icon
    {
        notification.app_icon.clone()
    } else if !notification.app_name.is_empty()
        && iwwc.config.notifications.respect_notification_icon
    {
        notification.app_name.clone().to_lowercase()
    } else {
        "default".to_string()
    };

    let icon = if let Some(icon) = icons.get(&icon_name) {
        icon.clone()
    } else {
        std::path::PathBuf::from(std::env::var("HOME").unwrap() + "/.config/iwwc/default.svg")
    };

    iwwc.notification_ids.insert(
        window_id,
        crate::gui::elements::notification::NotificationWindowInfo { notification, icon },
    );

     let notification_settings = iced::platform_specific::runtime::wayland::layer_surface::SctkLayerSurfaceSettings {
                        id: window_id,
                        layer: Layer::Overlay,
                        keyboard_interactivity: KeyboardInteractivity::None,
                        input_zone: None, //FIXME: figure out what is it Option<Vec<Rectangle>>,
                        anchor: iwwc.config.notifications.location,
                        output: iced::platform_specific::runtime::wayland::layer_surface::IcedOutput::Active, // FIXME: rewrite config to support output selection
                        namespace: format!("notification-{}", window_id),
                        margin: iced::platform_specific::runtime::wayland::layer_surface::IcedMargin {
                            top: iwwc.config.notifications.vertical_margin,
                            right: iwwc.config.notifications.horizontal_margin,
                            bottom: iwwc.config.notifications.vertical_margin,
                            left: iwwc.config.notifications.horizontal_margin,
                        },
                        //margin: //IcedMargin, //FIXME: add margin support
                        size: Some((Some(iwwc.config.notifications.width), Some(iwwc.config.notifications.height))),
                        exclusive_zone: 0,
                        //size_limits: iced_core::layout::Limits //FIXME: add size limits support
                        ..Default::default()
                    };

    iced::Task::batch([
        overflow,
        Task::done(Message::MoveNotifications),
        Task::done(Message::CreateWindow {
            id: window_id,
            settings: notification_settings,
        }),
        iced::Task::perform(
            tokio::time::sleep(std::time::Duration::from_secs(timeout.try_into().unwrap())),
            move |_| Message::CloseByContentId(id),
        ),
    ])
}