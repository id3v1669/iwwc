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
        log::debug!("IPC server listening on {socket_path:?}");

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
                let command_line = line.trim();
                log::debug!("Received IPC command: {command_line}");

                // Parse command and subcommand
                let parts: Vec<&str> = command_line.split_whitespace().collect();
                let command = parts.first().unwrap();
                let subcommand = parts.get(1);

                log::debug!(
                    "Sending command \"{command}\" with subcommand \"{subcommand:?}\" to iced"
                );
                // not calling handle_command directly as getting iwwc here would be pain in the ass
                let message =
                    Message::IpcCommand(command.to_string(), subcommand.map(|s| s.to_string()));

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
                if socket_path.exists()
                    && let Err(e) = std::fs::remove_file(&socket_path)
                {
                    log::error!("Failed to remove old socker file: {e}");
                }
                false
            }
        }
    }

    pub fn get_socket_path() -> PathBuf {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(runtime_dir).join("iwwc.sock")
    }

    pub async fn send_ipc_command(
        command: &str,
        subcommand: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let socket_path = Self::get_socket_path();
        use tokio::io::AsyncWriteExt;
        match tokio::net::UnixStream::connect(&socket_path).await {
            Ok(mut stream) => {
                let message = match subcommand {
                    Some(sub) => format!("{command} {sub}\n"),
                    None => format!("{command}\n"),
                };
                stream.write_all(message.as_bytes()).await?;
                log::debug!(
                    "Command '{command}' with subcommand '{subcommand:?}' sent successfully"
                );
            }
            Err(_) => {
                log::error!("Failed to connect to daemon. Is the daemon running?");
                std::process::exit(1);
            }
        }
        Ok(())
    }
    pub fn subscribe_iced() -> iced::Subscription<Message> {
        iced::Subscription::run_with_id(
            "ipc-server",
            iced::stream::channel(
                100,
                |sender: futures::channel::mpsc::Sender<_>| async move {
                    log::debug!("IPC server started");
                    let ipc_server = match crate::handler::ipc::IpcServer::new() {
                        Ok(server) => server,
                        Err(e) => {
                            log::error!("Failed to create IPC server: {e}");
                            return;
                        }
                    };

                    loop {
                        match ipc_server.accept().await {
                            Ok(stream) => {
                                let sender_clone = sender.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = crate::handler::ipc::IpcServer::handle_client(
                                        stream,
                                        sender_clone,
                                    )
                                    .await
                                    {
                                        log::error!("Error handling IPC client: {e}");
                                    }
                                });
                            }
                            Err(e) => {
                                log::error!("Failed to accept IPC connection: {e}");
                                break;
                            }
                        }
                    }
                },
            )
        )
    }
    pub fn handle_command(
        iwwc: &mut crate::gui::app::IcedWaylandWidgetCenter,
        command: String,
        subcommand: Option<String>,
    ) -> iced::Task<crate::gui::app::Message> {
        use crate::gui::app::Message;
        
        match command.as_str() {
            "test2" => {
                log::debug!("test2");
                iced::Task::done(Message::TestMessage)
            }
            "open" => match iwwc.config.widgets.get(subcommand.as_ref().unwrap()) {
                Some(window) => {
                    log::debug!("Found window with name: {:?}", subcommand.as_ref().unwrap());

                    // Verify if window with the same namespace is already open
                    if iwwc.widget_ids.values().any(|element| element == &window.namespace) {
                        log::warn!("Window with name {:?} is already open", subcommand.as_ref().unwrap());
                        return iced::Task::none();
                    }
                    let window_id = iced::window::Id::unique();
                    iwwc.widget_ids.insert(window_id, window.namespace.clone());
                    
                    let widget_settings = iced::platform_specific::runtime::wayland::layer_surface::SctkLayerSurfaceSettings {
                        id: window_id,
                        layer: window.layer,
                        keyboard_interactivity: window.keyboard_interactivity,
                        input_zone: None, //FIXME: figure out what is it Option<Vec<Rectangle>>,
                        anchor: window.anchor,
                        output: iced::platform_specific::runtime::wayland::layer_surface::IcedOutput::Active, // FIXME: rewrite config to support output selection
                        namespace: window.namespace.clone(),
                        //margin: //IcedMargin, //FIXME: add margin support
                        size: Some((Some(window.size.0), Some(window.size.1))),
                        exclusive_zone: window.exclusive_zone,
                        //size_limits: iced_core::layout::Limits //FIXME: add size limits support
                        ..Default::default()
                    };
                    
                    let widget_window = iced::Task::done(Message::CreateWindow {
                        id: window_id,
                        settings: widget_settings,
                    });
                    
                    let timeout = window.timeout.unwrap_or(0);
                    let timeout_task = if timeout > 0 {
                        iced::Task::perform(
                            tokio::time::sleep(std::time::Duration::from_secs(timeout as u64)),
                            move |_| Message::Close(window_id),
                        )
                    } else {
                        iced::Task::none()
                    };
                    
                    log::debug!("Widget window created with ID: {window_id:?}");
                    iced::Task::batch([widget_window, timeout_task])
                }
                None => {
                    log::warn!("No window found with name: {:?}", subcommand);
                    iced::Task::none()
                }
            },
            "close" => match subcommand {
                Some(sub) => {
                    let window_id_opt = iwwc
                        .widget_ids
                        .iter()
                        .find_map(|(id, element)| {
                            log::debug!("Checking window ID: {:?} with element name: {:?}", id, element);
                            if element == &sub { Some(*id) } else { None }
                        });
                    
                    if let Some(window_id) = window_id_opt {
                        log::debug!("Closing window with ID: {:?}", window_id);
                        iwwc.widget_ids.remove(&window_id);
                        iced::Task::done(Message::Close(window_id))
                    } else {
                        log::warn!("No window found with name: {:?}", sub);
                        iced::Task::none()
                    }
                }
                None => {
                    log::warn!("No subcommand provided for close command");
                    iced::Task::none()
                }
            },
            _ => {
                log::warn!("Yet unsupported command: {command}");
                iced::Task::none()
            }
        }
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        let socket_path = Self::get_socket_path();
        if socket_path.exists()
            && let Err(e) = fs::remove_file(&socket_path)
        {
            log::error!("Failed to remove socket file: {e}");
        }
    }
}


