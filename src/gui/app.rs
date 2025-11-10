use iced::{Color, Element, Task, Subscription};
use iced::window;
use iced::platform_specific::shell::commands::layer_surface::{
    Anchor, KeyboardInteractivity, Layer, get_layer_surface, destroy_layer_surface, set_margin,
};
use iced::runtime::platform_specific::wayland::layer_surface::{
    IcedOutput, SctkLayerSurfaceSettings,
};

use crate::handler::notification::NotificationHandler;

pub fn start() -> Result<(), iced::Error> {
    let config = crate::data::config::primary::Config::load(None);

    iced::daemon(
        IcedWaylandWidgetCenter::title,
        IcedWaylandWidgetCenter::update,
        IcedWaylandWidgetCenter::view,
    )
    .subscription(IcedWaylandWidgetCenter::subscription)
    .theme(IcedWaylandWidgetCenter::theme)
    .style(IcedWaylandWidgetCenter::style)
    .run_with(move || IcedWaylandWidgetCenter::new(config.clone()))
}

use indexmap::IndexMap;

pub struct IcedWaylandWidgetCenter {
    pub config: crate::data::config::primary::Config,
    pub notification_ids:
        IndexMap<iced::window::Id, crate::gui::elements::notification::NotificationWindowInfo>,
    pub widget_ids: std::collections::HashMap<iced::window::Id, String>,
    pub precalc: crate::data::notification::PreCalc,
    pub main_layer_id: Option<window::Id>, // Optional main background layer
}

#[derive(Debug, Clone)]
pub enum Message {
    LayerSurfaceCreated(window::Id),
    //FontLoaded(Result<(), iced::font::Error>),
    Close(iced::window::Id),
    CloseByContentId(u32),
    IpcCommand(String, Option<String>),
    RightClick(iced::window::Id),
    TestMessage,
    RunCommand(String),
    EmptyAction,
    MoveNotifications,
    Notify(crate::data::notification::Notification),
    MarginChange {
        id: window::Id,
        margin: (i32, i32, i32, i32),
    },
    CreateWindow {
        id: window::Id,
        settings: SctkLayerSurfaceSettings,
    },
}

impl IcedWaylandWidgetCenter {
    fn new(cfg: crate::data::config::primary::Config) -> (Self, Task<Message>) {
        // Create a minimal background layer surface that acts as the daemon's main window
        // This is a tiny invisible surface that keeps the daemon running
        let id = window::Id::unique();
        
        let output = match &cfg.global.output {
            Some(_output_name) => {
                log::warn!("Specific output targeting not fully implemented, using active output");
                IcedOutput::Active
            }
            None => IcedOutput::Active,
        };

        let layer_task = get_layer_surface(SctkLayerSurfaceSettings {
            id,
            namespace: "iced-wayland-widget-center-main".to_string(),
            size: Some((Some(1), Some(1))),
            layer: Layer::Background,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: 0,
            output,
            anchor: Anchor::TOP | Anchor::RIGHT,
            //margin: (0, 0, 0, 0),
            ..Default::default()
        });

        (
            Self {
                precalc: crate::data::notification::PreCalc::generate(&cfg),
                config: cfg,
                notification_ids: IndexMap::new(),
                widget_ids: std::collections::HashMap::new(),
                main_layer_id: Some(id),
            },
            Task::batch(vec![
                layer_task,
                Task::done(Message::LayerSurfaceCreated(id)),
            ]),
        )
    }

    fn title(&self, _id: window::Id) -> String {
        String::from("IcedWaylandWidgetCenter")
    }

    fn theme(&self, _id: window::Id) -> iced::Theme {
        iced::Theme::default()
    }

    fn style(&self, _theme: &iced::Theme) -> iced::daemon::Appearance {
        iced::daemon::Appearance {
            background_color: Color::TRANSPARENT,
            text_color: Color::TRANSPARENT,
            icon_color: Color::TRANSPARENT,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let notification_subscription = if self.config.notifications.enable {
            
            NotificationHandler::subscribe_iced()
        } else {
            log::debug!("Notifications are disabled in config");
            Subscription::none()
        };

        let ipc_subscription = crate::handler::ipc::IpcServer::subscribe_iced();

        Subscription::batch([
            notification_subscription,
            ipc_subscription,
            iced::event::listen_with(|event, _status, id| match event {
                iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
                    iced::mouse::Button::Right,
                )) => Some(Message::RightClick(id)),
                _ => None,
            }),
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LayerSurfaceCreated(id) => {
                log::debug!("Layer surface created with ID: {:?}", id);
                Task::none()
            }
            Message::Close(id) => {
                self.notification_ids.shift_remove(&id);

                Task::batch([
                    destroy_layer_surface(id),
                    Task::done(Message::MoveNotifications),
                ])
            }
            Message::RightClick(id) => {
                let (notification_window_info, _widget_info) = self.id_info(id);
                if notification_window_info.is_some() {
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
                crate::handler::ipc::IpcServer::handle_command(self, command, subcommand)
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
            Message::MarginChange { id, margin } => {
                set_margin(id, margin.0, margin.1, margin.2, margin.3)
            }
            Message::TestMessage => {
                println!("TestMessage");
                Task::none()
            }
            Message::EmptyAction => {
                log::warn!("Empty action received");
                Task::none()
            }
            Message::RunCommand(command) => {
                log::debug!("Running command: {command}");
                tokio::spawn(async move {
                    crate::handler::actions::run_system_command(&command).await;
                });
                Task::none()
            }
            Message::Notify(notification) => {
                crate::handler::notification::handle_notification(self, notification)
            }
            // Message::FontLoaded(result) => {
            //     if let Err(e) = result {
            //         log::error!("Failed to load font: {:?}", e);
            //     }
            //     Task::none()
            // }
            Message::CreateWindow { id, settings } => {
                log::debug!("Creating window: {:?}", id);
                get_layer_surface(settings)
            }
        }
    }

    fn view(&self, id: iced::window::Id) -> Element<'_, Message> {
        // Check if this is the main background layer
        if self.main_layer_id == Some(id) {
            // Return minimal transparent element for the background daemon window
            return iced::widget::container(iced::widget::horizontal_space())
                .width(1)
                .height(1)
                .style(|_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::TRANSPARENT)),
                    text_color: Some(Color::TRANSPARENT),
                    ..Default::default()
                })
                .into();
        }

        let (notification_window_info, widget_info) = self.id_info(id);
        if let Some(notification_window_info) = notification_window_info {
            return crate::gui::elements::notification::body(self, notification_window_info).into();
        };

        if let Some(widget_info) = widget_info {
            return crate::gui::elements::element::body(self, widget_info).into();
        };

        iced::widget::container(iced::widget::horizontal_space())
            .style(|_| iced::widget::container::Style {
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
        (
            self.notification_ids.get(&id).cloned(),
            self.widget_ids.get(&id).cloned(),
        )
    }
}