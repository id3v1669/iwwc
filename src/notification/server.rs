use std::collections::HashMap;

use futures::channel::mpsc::Sender;
use zbus::Connection;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::Value;

use crate::daemon::Message;
use crate::notification::types::Notification;

const PATH: &str = "/org/freedesktop/Notifications";
const IFACE: &str = "org.freedesktop.Notifications";

pub struct NotificationHandler {
    count: u32,
    sender: Sender<Message>,
}

impl NotificationHandler {
    pub fn new(sender: Sender<Message>) -> Self {
        NotificationHandler { count: 0, sender }
    }
}

fn hint_string(hints: &HashMap<String, Value<'_>>, key: &str) -> Option<String> {
    hints.get(key).and_then(|v| String::try_from(v).ok())
}

fn hint_urgency(hints: &HashMap<String, Value<'_>>) -> u8 {
    let v = match hints.get("urgency") {
        Some(v) => v,
        None => return 1,
    };
    let n = match v {
        Value::U8(n) => *n as i64,
        Value::U16(n) => *n as i64,
        Value::U32(n) => *n as i64,
        Value::U64(n) => *n as i64,
        Value::I16(n) => *n as i64,
        Value::I32(n) => *n as i64,
        Value::I64(n) => *n,
        _ => return 1,
    };
    n.clamp(0, 2) as u8
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl NotificationHandler {
    #[allow(non_snake_case, clippy::too_many_arguments)]
    async fn Notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, Value<'_>>,
        expire_timeout: i32,
    ) -> zbus::fdo::Result<u32> {
        let notification_id = if replaces_id != 0 {
            replaces_id
        } else {
            self.count += 1;
            self.count
        };
        let desktop_entry = hint_string(&hints, "desktop-entry").unwrap_or_default();
        let image_path = hint_string(&hints, "image-path");
        let urgency = hint_urgency(&hints);
        let n = Notification {
            app_name,
            app_icon,
            replaces_id,
            summary,
            body,
            actions,
            expire_timeout,
            notification_id,
            desktop_entry,
            image_path,
            urgency,
        };
        self.sender.try_send(Message::Notify(n)).ok();
        Ok(notification_id)
    }

    #[allow(non_snake_case)]
    async fn CloseNotification(&mut self, id: u32) -> zbus::fdo::Result<()> {
        self.sender.try_send(Message::NotifClose(id)).ok();
        Ok(())
    }

    #[allow(non_snake_case)]
    fn GetServerInformation(&self) -> zbus::fdo::Result<(String, String, String, String)> {
        Ok((
            "iwwc".into(),
            "iwwc".into(),
            env!("CARGO_PKG_VERSION").into(),
            "1.2".into(),
        ))
    }

    #[allow(non_snake_case)]
    fn GetCapabilities(&self) -> zbus::fdo::Result<Vec<String>> {
        Ok(vec![
            "actions".into(),
            "body".into(),
            "persistence".into(),
            "icon-static".into(),
        ])
    }

    #[zbus(signal)]
    async fn notification_closed(
        emitter: &SignalEmitter<'_>,
        id: u32,
        reason: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn action_invoked(
        emitter: &SignalEmitter<'_>,
        id: u32,
        action_key: &str,
    ) -> zbus::Result<()>;
}

pub async fn emit_closed(conn: &Connection, id: u32, reason: u32) -> zbus::Result<()> {
    SignalEmitter::new(conn, PATH)?
        .emit(IFACE, "NotificationClosed", &(id, reason))
        .await
}

pub async fn emit_action_invoked(conn: &Connection, id: u32, key: &str) -> zbus::Result<()> {
    SignalEmitter::new(conn, PATH)?
        .emit(IFACE, "ActionInvoked", &(id, key))
        .await
}
