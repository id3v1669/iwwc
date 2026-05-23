use futures::sink::SinkExt;
use futures::stream::Stream;
use iced::Subscription;
use std::sync::{Arc, Mutex};

use crate::daemon::Message;
use crate::ipc::{self, IpcServer};

fn ipc_stream() -> impl Stream<Item = Message> {
    iced::stream::channel(100, async |mut output| {
        let server = match IpcServer::bind(&ipc::socket_path()) {
            Ok(s) => s,
            Err(e) => {
                log::error!("ipc bind failed: {e}");
                return;
            }
        };
        let mut rx = server.serve();
        while let Some(inc) = rx.recv().await {
            let reply = Arc::new(Mutex::new(Some(inc.reply)));
            if output
                .send(Message::Ipc {
                    command: inc.command,
                    reply,
                })
                .await
                .is_err()
            {
                break;
            }
        }
    })
}

pub fn subscription() -> Subscription<Message> {
    Subscription::run(ipc_stream)
}
