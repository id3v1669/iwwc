use iced::{Color, Element, Task};
use iced_layershell::build_pattern::daemon;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::to_layer_message;

use crate::handler::notification::NotificationHandler;

pub fn start() -> Result<(), iced_layershell::Error> {
    let config = crate::data::config::primary::Config::load(None); //TODO: change to read from file in args
    // start mode should be background, but since lib is kinda broken, use target screen as workaround
    // active is also working not as expected, but need some kind of fallback
    let start_mode = match config.global.output {
        Some(ref output) => iced_layershell::settings::StartMode::TargetScreen(output.to_string()),
        None => iced_layershell::settings::StartMode::Active,
    };
    let settings = Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::Top | Anchor::Right,
            layer: Layer::Background,
            exclusive_zone: 0,
            size: Some((4, 4)),
            margin: (10, 10, 10, 10),
            keyboard_interactivity: KeyboardInteractivity::None,
            start_mode: start_mode,
            ..Default::default()
        },
        antialiasing: config.global.antialiasing.unwrap_or(true),
        ..Default::default()
    };
    daemon(
        move || IcedWaylandWidgetCenter::new(config.clone()),
        "IcedWaylandWidgetCenter",
        IcedWaylandWidgetCenter::update,
        IcedWaylandWidgetCenter::view,
    )
    .subscription(IcedWaylandWidgetCenter::subscription)
    .style(|_state, _theme| iced::theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: Color::TRANSPARENT,
    })
    .settings(settings)
    .run()
}

use indexmap::IndexMap;

pub struct IcedWaylandWidgetCenter {
    pub config: crate::data::config::primary::Config,
    pub notification_ids:
        IndexMap<iced::window::Id, crate::gui::elements::notification::NotificationWindowInfo>,
    pub widget_ids: std::collections::HashMap<iced::window::Id, String>,
    pub precalc: crate::data::notification::PreCalc,
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    FontLoaded(Result<(), iced::font::Error>),
    Close(iced::window::Id),
    CloseByContentId(u32),
    IpcCommand(String, Option<String>),
    RightClick(iced::window::Id),
    TestMessage,
    MoveNotifications,
    Notify(crate::data::notification::Notification),
}

impl IcedWaylandWidgetCenter {
    fn new(cfg: crate::data::config::primary::Config) -> (Self, Task<Message>) {
        (
            Self {
                precalc: crate::data::notification::PreCalc::generate(&cfg),
                config: cfg,
                notification_ids: IndexMap::new(),
                widget_ids: std::collections::HashMap::new(),
            },
            Task::none(),
            // Task::batch(vec![
            //     iced::font::load(ICOFONT_BYTES).map(Message::FontLoaded),
            //     // iced::font::load(
            //     //     iced_fonts::REQUIRED_FONT_BYTES
            //     // ).map(Message::FontLoaded),
            //     // iced::font::load(
            //     //     iced_fonts::NERD_FONT_BYTES
            //     // ).map(Message::FontLoaded),
            //     // iced::font::load(
            //     //     iced_fonts::BOOTSTRAP_FONT_BYTES
            //     // ).map(Message::FontLoaded),
            // ]),
        )
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        let notification_subscription = if self.config.notifications.enable {
            iced::Subscription::run(|| {
                iced::stream::channel(100, |sender| async move {
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
            })
        } else {
            iced::Subscription::none()
        };

        let ipc_subscription = iced::Subscription::run(|| {
            iced::stream::channel(
                100,
                |sender: futures::channel::mpsc::Sender<_>| async move {
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
        });

        iced::Subscription::batch([
            notification_subscription,
            ipc_subscription,
            iced::event::listen_with(|event, status, id| match event {
                iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
                    iced::mouse::Button::Right,
                )) => Some(Message::RightClick(id)),
                _ => None,
            }),
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Close(id) => {
                self.notification_ids.shift_remove(&id);

                Task::batch([
                    Task::done(Message::RemoveWindow(id)),
                    Task::done(Message::MoveNotifications),
                ])
            }
            Message::RightClick(id) => {
                let (notification_window_info, widget_info) = self.id_info(id);
                if let Some(notification_window_info) = notification_window_info {
                    return Task::done(Message::Close(id));
                }
                Task::none()
            }
            Message::CloseByContentId(notification_id) => {
                if let Some((window_id, _)) = self
                    .notification_ids
                    .iter()
                    .find(|(_, info)| info.notification.notification_id == notification_id)
                    .map(|(k, v)| (*k, v))
                {
                    return Task::done(Message::Close(window_id));
                }
                Task::none()
            }
            Message::IpcCommand(command, subcommand) => {
                log::debug!("Received IPC command: {command}");
                crate::handler::ipc::handle_command(self, command, subcommand)
            }
            Message::MoveNotifications => {
                let mut move_notifications: Vec<Task<Message>> = Vec::new();

                for (position, (window_id, _)) in self.notification_ids.iter().enumerate() {
                    let offset: i32 = {
                        (self.config.notifications.height as i32 * position as i32)
                            + (self.config.notifications.vertical_margin * position as i32)
                            + self.config.notifications.vertical_margin
                    };
                    move_notifications.push(Task::done(Message::MarginChange {
                        id: *window_id,
                        margin: (
                            offset,
                            self.config.notifications.horizontal_margin,
                            self.config.notifications.vertical_margin,
                            self.config.notifications.horizontal_margin,
                        ),
                    }));
                }

                if !move_notifications.is_empty() {
                    return Task::batch(move_notifications);
                }
                Task::none()
            }
            Message::TestMessage => {
                println!("TestMessage");
                Task::none()
            }
            Message::Notify(notification) => {
                crate::handler::notification::handle_notification(self, notification)
            }
            _ => unreachable!(),
        }
    }

    fn view(&self, id: iced::window::Id) -> Element<Message> {
        let (notification_window_info, widget_info) = self.id_info(id);
        if let Some(notification_window_info) = notification_window_info {
            return crate::gui::elements::notification::body(self, notification_window_info).into();
        };

        if let Some(widget_info) = widget_info {
            return crate::gui::elements::element::body(self, widget_info).into();
        };

        iced::widget::container(iced::widget::horizontal_space())
            .style(move |_| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::TRANSPARENT)),
                text_color: Some(Color::TRANSPARENT),
                ..Default::default()
            })
            .into()
    }

    fn id_info(
        &self,
        id: iced::window::Id,
    ) -> (
        Option<crate::gui::elements::notification::NotificationWindowInfo>,
        Option<String>,
    ) {
        //None to be info of widget elements
        (
            self.notification_ids.get(&id).cloned(),
            self.widget_ids.get(&id).cloned(),
        )
    }
    fn style(_counter: &Self, theme: &iced::Theme) -> iced::theme::Style {
        use iced::theme::Style;
        Style {
            background_color: Color::TRANSPARENT,
            text_color: theme.palette().text,
        }
    }
}
