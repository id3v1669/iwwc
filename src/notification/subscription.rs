use std::sync::OnceLock;

use futures::stream::Stream;
use iced::Subscription;
use zbus::Connection;

use crate::daemon::Message;
use crate::notification::server::NotificationHandler;

static CONNECTION: OnceLock<Connection> = OnceLock::new();

pub fn connection() -> Option<Connection> {
    CONNECTION.get().cloned()
}

fn notification_stream() -> impl Stream<Item = Message> {
    iced::stream::channel(100, async |output| {
        let handler = NotificationHandler::new(output);
        let builder = match zbus::connection::Builder::session()
            .and_then(|b| b.name("org.freedesktop.Notifications"))
            .and_then(|b| b.serve_at("/org/freedesktop/Notifications", handler))
        {
            Ok(b) => b,
            Err(e) => {
                log::error!("notification dbus setup failed: {e}");
                return;
            }
        };
        let conn = match builder.build().await {
            Ok(c) => c,
            Err(e) => {
                log::error!("notification name unavailable (another daemon?): {e}");
                return;
            }
        };
        let _ = CONNECTION.set(conn);
        futures::future::pending::<()>().await;
    })
}

pub fn subscription() -> Subscription<Message> {
    Subscription::run(notification_stream)
}
