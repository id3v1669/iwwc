use futures::SinkExt;
use futures::channel::mpsc;
use std::fs;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use crate::gui::app::Message;

pub struct IpcServer {
    listener: UnixListener,
}

impl IpcServer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let socket_path = Self::get_socket_path();

        // Remove existing socket if it exists
        if socket_path.exists() {
            fs::remove_file(&socket_path)?;
        }

        let listener = UnixListener::bind(&socket_path)?;
        log::info!("IPC server listening on {socket_path:?}");

        Ok(Self { listener })
    }

    pub async fn accept(&self) -> Result<UnixStream, Box<dyn std::error::Error>> {
        let (stream, _) = self.listener.accept().await?;
        Ok(stream)
    }

    pub async fn handle_client(
        stream: UnixStream,
        mut sender: mpsc::Sender<Message>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        match reader.read_line(&mut line).await {
            Ok(0) => return Ok(()), // Connection closed
            Ok(_) => {
                let command = line.trim();
                log::debug!("Received IPC command: {command}");

                let message = match command {
                    "test" => Message::TestMessage,
                    window => {
                        log::debug!("Sending command \"{command}\" to iced");
                        Message::IpcCommand(window.to_string())
                    }
                };

                if let Err(e) = sender.send(message).await {
                    log::error!("Failed to send message: {e}");
                }
            }
            Err(e) => {
                log::error!("Error reading from IPC client: {e}");
            }
        }

        Ok(())
    }

    pub async fn is_active() -> bool {
        let socket_path = Self::get_socket_path();

        match UnixStream::connect(&socket_path).await {
            Ok(_) => true,
            Err(_) => {
                if socket_path.exists() {
                    if let Err(e) = std::fs::remove_file(&socket_path) {
                        log::error!("Failed to remove old socker file: {e}");
                    }
                }
                false
            }
        }
    }

    pub fn get_socket_path() -> PathBuf {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(runtime_dir).join("iwwc.sock")
    }

    pub async fn send_ipc_command(command: &str) -> Result<(), Box<dyn std::error::Error>> {
        let socket_path = Self::get_socket_path();
        use tokio::io::AsyncWriteExt;
        match tokio::net::UnixStream::connect(&socket_path).await {
            Ok(mut stream) => {
                let message = format!("{command}\n");
                stream.write_all(message.as_bytes()).await?;
                log::debug!("Command '{command}' sent successfully");
            }
            Err(_) => {
                log::error!("Failed to connect to daemon. Is the daemon running?");
                std::process::exit(1);
            }
        }
        Ok(())
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        let socket_path = Self::get_socket_path();
        if socket_path.exists() {
            if let Err(e) = fs::remove_file(&socket_path) {
                log::error!("Failed to remove socket file: {e}");
            }
        }
    }
}

pub fn handle_command(
    iwwc: &mut crate::gui::app::IcedWaylandWidgetCenter,
    command: String,
) -> iced::Task<Message> {
    match command.as_str() {
        "test2" => {
            log::info!("test2");
            iced::Task::done(Message::TestMessage)
        }
        _ => {
            match iwwc.config.widgets.iter().find(|w| w.id == command) {
                Some(window) => {
                    log::debug!("Found window with name: {command}");
                    let window_id = iced::window::Id::unique();

                    iwwc.widget_ids.insert(window_id, window.element.clone());

                    let widget_window = iced::Task::done(Message::NewLayerShell {
                        settings: window.settings.clone(),
                        id: window_id,
                    });

                    let timeout = window.timeout.unwrap_or(0);
                    let timeout_task = if timeout > 0 {
                        iced::Task::none()
                        // iced::Task::perform(
                        //     tokio::time::sleep(std::time::Duration::from_secs(timeout as u64)),
                        //     move |_| Message::Close(window_id),
                        // )
                    } else {
                        iced::Task::none()
                    };
                    log::debug!("Widget window created with ID: {window_id:?}");
                    return iced::Task::batch([widget_window, timeout_task]);
                }
                None => {
                    log::warn!("No window found with name: {command}");
                    return iced::Task::none();
                }
            }
        }
    }
}
