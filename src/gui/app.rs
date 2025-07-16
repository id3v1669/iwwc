use std::collections::HashMap;

use iced::{Color, Element, Task};
use iced_layershell::build_pattern::daemon;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::to_layer_message;

use crate::handler::notification::NotificationHandler;

pub fn start() -> Result<(), iced_layershell::Error> {
    let settings = Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::Top | Anchor::Right,
            layer: Layer::Overlay,
            exclusive_zone: 0,
            size: Some((50, 50)),
            margin: (10, 10, 10, 10),
            keyboard_interactivity: KeyboardInteractivity::None,
            //start_mode: iced_layershell::settings::StartMode::TargetScreen("DP-3".to_string()),
            start_mode: iced_layershell::settings::StartMode::Background,
            ..Default::default()
        },
        //antialiasing: false,
        ..Default::default()
    };
    daemon(
        IcedWaylandWidgetCenter::new,
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

struct IcedWaylandWidgetCenter {
    ids: HashMap<iced::window::Id, WindowInfo>,
    precalc: crate::data::notification::PreCalc,
}

#[derive(Debug, Clone, PartialEq)]
struct WindowInfo {
    notification: crate::data::notification::Notification,
    icon: std::path::PathBuf,
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
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                ids: HashMap::new(),
                precalc: crate::data::notification::PreCalc::generate(),
            },
            Task::none(),
        )
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch([
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
            }),
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
                let mut active_notifications =
                    crate::data::shared::ACTIVE_NOTIFICATIONS.lock().unwrap();
                let info = self.id_info(id).unwrap();

                //TODO rework this method to reverce cycle, to eliminate use of find
                if let Some((&key, _)) = active_notifications
                    .iter()
                    .find(|&(_, &value)| value == info.notification.notification_id)
                {
                    let pre_last = (active_notifications.len() - 1) as i32;
                    for i in key..=pre_last {
                        if let Some(&next_value) = active_notifications.get(&(i + 1)) {
                            active_notifications.insert(i, next_value);
                        }
                    }
                    active_notifications.remove(&(pre_last + 1));
                }

                Task::batch([
                    Task::done(Message::RemoveWindow(id)),
                    Task::done(Message::MoveNotifications),
                ])
            }
            Message::CloseByContentId(notification_id) => {
                if let Some(id) = self.window_id(notification_id) {
                    return Task::done(Message::Close(*id));
                }
                Task::none()
            }
            Message::MoveNotifications => {
                let config = crate::data::shared::CONFIG.lock().unwrap();

                let active_notifications =
                    crate::data::shared::ACTIVE_NOTIFICATIONS.lock().unwrap();
                let mut move_notifications: Vec<Task<Message>> = Vec::new();

                for (position_in_q, id_in_q) in active_notifications.clone() {
                    if let Some(id) = self.window_id(id_in_q) {
                        let offset: i32 = {
                            (config.height as i32 * (position_in_q - 1))
                                + (config.vertical_margin * (position_in_q - 1))
                                + config.vertical_margin
                        };
                        move_notifications.push(Task::done(Message::MarginChange {
                            id: *id,
                            margin: (
                                offset,
                                config.horizontal_margin,
                                config.vertical_margin,
                                config.horizontal_margin,
                            ),
                        }));
                    }
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
                let mut overflow = Task::none();
                let id = notification.notification_id;
                let config = crate::data::shared::CONFIG.lock().unwrap();
                let mut active_notifications =
                    crate::data::shared::ACTIVE_NOTIFICATIONS.lock().unwrap();
                let active_notifications_count = active_notifications.len() as i32;
                if let Some(id_last) = active_notifications.get(&config.max_notifications) {
                    overflow = Task::done(Message::CloseByContentId(*id_last));
                }
                for i in
                    (1..=std::cmp::min(active_notifications_count, config.max_notifications - 1))
                        .rev()
                {
                    if let Some(&prev_value) = active_notifications.get(&i) {
                        active_notifications.insert(i + 1, prev_value);
                    }
                }
                active_notifications
                    .entry(1)
                    .and_modify(|value| *value = id)
                    .or_insert(id);
                let timeout =
                    if config.respect_notification_timeout && notification.expire_timeout > 0 {
                        notification.expire_timeout
                    } else {
                        config.local_expire_timeout
                    };
                let icons = crate::data::shared::ICONS.lock().unwrap();

                let icon_name = if !notification.app_icon.is_empty() {
                    notification.app_icon.clone()
                } else if !notification.app_name.is_empty() {
                    notification.app_name.clone().to_lowercase()
                } else {
                    "default".to_string()
                };
                let icon = if let Some(icon) = icons.get(&icon_name) {
                    icon.clone()
                } else {
                    std::path::PathBuf::from(
                        std::env::var("HOME").unwrap() + "/.config/iwwc/default.svg",
                    )
                };

                Task::batch([
                    overflow,
                    Task::done(Message::MoveNotifications),
                    Task::done(Message::NewLayerShell {
                        settings: NewLayerShellSettings {
                            size: Some((config.width, config.height)),
                            exclusive_zone: None,
                            anchor: Anchor::Top | Anchor::Right,
                            layer: Layer::Overlay,
                            margin: Some((
                                config.vertical_margin,
                                config.horizontal_margin,
                                config.vertical_margin,
                                config.horizontal_margin,
                            )),
                            keyboard_interactivity: KeyboardInteractivity::None,
                            // would've used flag if it wasnt broken
                            // Message::ForgetLastOutput was used for this, but issue seems to be with implementation itself as
                            // notifications allways appear on second screen instead of last used
                            output_option: iced_layershell::reexport::OutputOption::LastOutput,
                            ..Default::default()
                        },
                        id: {
                            let id = iced::window::Id::unique();
                            self.set_id_info(id, WindowInfo { notification, icon });
                            id
                        },
                    }),
                    Task::perform(Self::sleep_timer(timeout.try_into().unwrap()), move |_| {
                        Message::CloseByContentId(id)
                    }),
                ])
            }
            _ => unreachable!(),
        }
    }

    fn view(&self, id: iced::window::Id) -> Element<Message> {
        if let Some(window_info) = self.id_info(id) {
            return iced::widget::container(
                iced::widget::row![
                    iced::widget::svg(window_info.icon.clone())
                        .width(iced::Length::Fixed(self.precalc.image_size))
                        .height(iced::Length::Fixed(self.precalc.image_size)),
                    iced::widget::column![
                        iced::widget::column![
                            iced::widget::text(window_info.notification.summary.clone())
                                .size(self.precalc.font_size_summary)
                                .align_x(iced::alignment::Horizontal::Left),
                        ]
                        .padding(self.precalc.text_summary_paddings),
                        iced::widget::column![
                            iced::widget::text(window_info.notification.body.clone())
                                .size(self.precalc.font_size_body),
                        ]
                        .padding(self.precalc.text_body_paddings),
                    ]
                    .padding(self.precalc.text_paddings_block)
                ]
                .align_y(iced::alignment::Vertical::Center)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill),
            )
            .padding(self.precalc.general_padding)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(move |_| crate::gui::elements::style::notification_style())
            .into();
        }
        iced::widget::container("ss")
            .padding(10)
            .center(800)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(move |_| crate::gui::elements::style::notification_style())
            .into()
    }

    // Style method is now handled in the daemon builder chain above

    async fn sleep_timer(sleep_time: u64) {
        tokio::time::sleep(std::time::Duration::from_secs(sleep_time)).await;
    }
    fn window_id(&self, notification_id: u32) -> Option<&iced::window::Id> {
        for (k, v) in self.ids.iter() {
            if notification_id == v.notification.notification_id {
                return Some(k);
            }
        }
        None
    }

    fn id_info(&self, id: iced::window::Id) -> Option<WindowInfo> {
        self.ids.get(&id).cloned()
    }

    fn set_id_info(&mut self, id: iced::window::Id, info: WindowInfo) {
        self.ids.insert(id, info);
    }
}
