use std::collections::HashMap;

use futures::channel::mpsc::Sender;
use futures::sink::SinkExt;
use futures::stream::{Stream, StreamExt};
use iced::Subscription;
use zbus::zvariant::OwnedValue;

use crate::daemon::Message;

#[zbus::proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_service = "org.mpris.MediaPlayer2.playerctld",
    default_path = "/org/mpris/MediaPlayer2"
)]
trait Player {
    #[zbus(property)]
    fn metadata(&self) -> zbus::Result<HashMap<String, OwnedValue>>;
}

fn title_of(meta: &HashMap<String, OwnedValue>) -> Option<String> {
    meta.get("xesam:title")
        .and_then(|v| String::try_from(&**v).ok())
        .filter(|s| !s.is_empty())
}

async fn publish(output: &mut Sender<Message>, title: Option<String>) {
    if crate::config::smart::set_activesong(title) {
        let _ = output.send(Message::SmartRefresh).await;
    }
}

async fn watch(output: &mut Sender<Message>) -> zbus::Result<()> {
    let conn = zbus::Connection::session().await?;
    let proxy = PlayerProxy::new(&conn).await?;
    let title = title_of(&proxy.metadata().await?);
    let mut owner_changes = proxy.inner().receive_owner_changed().await?;
    let mut meta_changes = proxy.receive_metadata_changed().await;
    publish(output, title).await;
    loop {
        tokio::select! {
            change = meta_changes.next() => {
                let Some(change) = change else { return Ok(()) };
                let title = change.get().await.ok().and_then(|m| title_of(&m));
                publish(output, title).await;
            }
            _ = owner_changes.next() => { return Ok(()) }
        }
    }
}

fn mpris_stream() -> impl Stream<Item = Message> {
    iced::stream::channel(16, async |mut output| {
        loop {
            match watch(&mut output).await {
                Ok(()) => {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                Err(e) => {
                    log::debug!("mpris: {e}");
                    publish(&mut output, None).await;
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    })
}

pub fn subscription() -> Subscription<Message> {
    Subscription::run(mpris_stream)
}
