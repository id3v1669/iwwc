use iced::{Color, Element, Task};
use iced_layershell::build_pattern::daemon;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::to_layer_message;

use crate::handler::notification::NotificationHandler;

pub fn start() -> Result<(), iced_layershell::Error> {
    let config = crate::data::config::Config::default(); //TODO: change to read from file
    let settings = Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::Top | Anchor::Right,
            layer: Layer::Overlay,
            exclusive_zone: 0,
            size: None,
            margin: (10, 10, 10, 10),
            keyboard_interactivity: KeyboardInteractivity::None,
            start_mode: iced_layershell::settings::StartMode::Background,
            ..Default::default()
        },
        antialiasing: config.global.antialiasing,
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
    pub config: crate::data::config::Config,
    pub notification_ids:
        IndexMap<iced::window::Id, crate::gui::elements::notification::NotificationWindowInfo>,
    pub precalc: crate::data::notification::PreCalc,
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    Close(iced::window::Id),
    CloseByContentId(u32),
    TestMessage,
    MoveNotifications,
    Notify(crate::data::notification::Notification),
}

impl IcedWaylandWidgetCenter {
    fn new(cfg: crate::data::config::Config) -> (Self, Task<Message>) {
        (
            Self {
                precalc: crate::data::notification::PreCalc::generate(&cfg),
                config: cfg,
                notification_ids: IndexMap::new(),
            },
            Task::none(),
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
            iced::event::listen_with(|event, _status, id| match event {
                iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
                    iced::mouse::Button::Right,
                )) => Some(Message::Close(id)),
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
        let (notification_window_info, _) = self.id_info(id);
        let notification: iced::widget::Container<Message> =
            if let Some(notification_window_info) = notification_window_info {
                crate::gui::elements::notification::body(self, notification_window_info)
            } else {
                iced::widget::container(iced::widget::horizontal_space())
                    .style(move |_| crate::gui::elements::style::notification_style(&self.config))
            };
        iced::widget::stack![notification]
            //.padding(10)
            //.center(800)
            //.width(iced::Length::Fill)
            //.height(iced::Length::Fill)
            //.style(move |_| crate::gui::elements::style::notification_style(&self.config))
            .into()
    }

    fn id_info(
        &self,
        id: iced::window::Id,
    ) -> (
        Option<crate::gui::elements::notification::NotificationWindowInfo>,
        Option<bool>,
    ) {
        //None to be info of widget elements
        (self.notification_ids.get(&id).cloned(), None)
    }
}
