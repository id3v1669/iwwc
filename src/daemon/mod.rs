pub mod action;
pub mod ipc_bridge;
pub mod menu;
pub mod notification;
pub mod pull;
pub mod window;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use iced::window::Id as WindowId;
use iced::{Element, Subscription, Task};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;
use indexmap::IndexMap;
use tokio::sync::oneshot;

use crate::config::store::Store;
use crate::ipc::{Command, Response};
use crate::notification::types::{Notification, PreCalc};
use crate::render;
use crate::render::UiMessage;

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    Ui(render::UiMessage),
    Ipc {
        command: Command,
        reply: Arc<Mutex<Option<oneshot::Sender<Response>>>>,
    },
    WindowClosed(WindowId),
    Notify(crate::notification::types::Notification),
    NotifClose(u32),
    NotifTimeout(u32),
    TrayItems(Vec<crate::tray::types::TrayItem>),
    MenuOpen {
        bus_name: String,
        menu_path: String,
        root: crate::tray::menu_types::MenuItem,
        ctx: crate::daemon::menu::AnchorCtx,
    },
    MenuCloseAll,
    CursorMoved {
        window: WindowId,
        x: f32,
        y: f32,
    },
    NotifRightClick(WindowId),
    PullTick(String),
    PullResult {
        name: String,
        value: String,
    },
    SmartRefresh,
    Noop,
}

pub struct App {
    store: Store,
    config_path: std::path::PathBuf,
    windows: HashMap<WindowId, String>,
    notifications: IndexMap<u32, NotifState>,
    notif_windows: HashMap<WindowId, u32>,
    tray_items: Vec<crate::tray::types::TrayItem>,
    menus: Vec<crate::daemon::menu::MenuLevel>,
    menu_windows: HashMap<WindowId, usize>,
    menu_overlay: Option<WindowId>,
    cursor: HashMap<WindowId, (f32, f32)>,
}

struct NotifState {
    notification: Notification,
    icon: Option<std::path::PathBuf>,
    precalc: PreCalc,
    window: Option<WindowId>,
}

pub fn run(store: Store, config_path: std::path::PathBuf) -> iced_layershell::Result {
    iced_layershell::daemon(
        move || {
            let app = App::new(store.clone(), config_path.clone());
            let init: Vec<Task<Message>> = app
                .store
                .pulls()
                .iter()
                .map(|(name, decl)| {
                    run_pull_task(name.clone(), decl.command.clone(), decl.default.clone())
                })
                .collect();
            (app, Task::batch(init))
        },
        App::namespace,
        App::update,
        App::view,
    )
    .subscription(App::subscription)
    .style(App::style)
    .settings(Settings {
        layer_settings: hidden_initial(),
        ..Default::default()
    })
    .run()
}

fn hidden_initial() -> LayerShellSettings {
    LayerShellSettings {
        size: Some((1, 1)),
        layer: Layer::Background,
        anchor: Anchor::Top | Anchor::Left,
        exclusive_zone: 0,
        keyboard_interactivity: KeyboardInteractivity::None,
        start_mode: StartMode::Active,
        ..Default::default()
    }
}

impl App {
    fn new(store: Store, config_path: std::path::PathBuf) -> Self {
        App {
            store,
            config_path,
            windows: HashMap::new(),
            notifications: IndexMap::new(),
            notif_windows: HashMap::new(),
            tray_items: Vec::new(),
            menus: Vec::new(),
            menu_windows: HashMap::new(),
            menu_overlay: None,
            cursor: HashMap::new(),
        }
    }

    fn namespace() -> String {
        "iwwc".to_string()
    }

    fn style(_state: &App, theme: &iced::Theme) -> iced::theme::Style {
        iced::theme::Style {
            background_color: iced::Color::TRANSPARENT,
            text_color: theme.palette().text,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            ipc_bridge::subscription(),
            crate::notification::subscription::subscription(),
            crate::tray::subscription::subscription(),
            iced::window::close_events().map(Message::WindowClosed),
            iced::keyboard::listen().map(|ev| match ev {
                iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                    ..
                } => Message::Ui(UiMessage::MenuDismiss),
                _ => Message::Noop,
            }),
            iced::event::listen_with(pointer_event),
        ];
        for (name, decl) in self.store.pulls() {
            subs.push(
                iced::time::every(decl.interval)
                    .with(name.clone())
                    .map(|(name, _instant)| Message::PullTick(name)),
            );
        }
        let mut intervals: Vec<std::time::Duration> = self
            .store
            .resolved()
            .smart_polls
            .iter()
            .map(|(_, d)| *d)
            .collect();
        intervals.sort();
        intervals.dedup();
        for d in intervals {
            subs.push(iced::time::every(d).map(|_| Message::SmartRefresh));
        }
        Subscription::batch(subs)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Ui(UiMessage::Action(cmd)) => {
                action::run_action(&cmd);
                Task::none()
            }
            Message::Ui(UiMessage::NotifAction { id, key }) => {
                let emit = emit_action_task(id, key);
                let close = self.close_notification(id, 2);
                Task::batch([emit, close])
            }
            Message::WindowClosed(id) => {
                self.windows.remove(&id);
                if let Some(nid) = self.notif_windows.remove(&id) {
                    self.notifications.shift_remove(&nid);
                    return self.restack();
                }
                if self.menu_overlay == Some(id) || self.menu_windows.contains_key(&id) {
                    return self.close_menus();
                }
                Task::none()
            }
            Message::Notify(n) => self.on_notify(n),
            Message::NotifClose(id) => self.close_notification(id, 3),
            Message::NotifTimeout(id) => self.close_notification(id, 1),
            Message::TrayItems(items) => {
                self.tray_items = items;
                Task::none()
            }
            Message::Ui(UiMessage::TrayActivate(idx)) => self.tray_call(idx, TrayMethod::Activate),
            Message::Ui(UiMessage::TraySecondary(idx)) => {
                self.tray_call(idx, TrayMethod::Secondary)
            }
            Message::Ui(UiMessage::TrayContextMenu { window, idx }) => {
                match self.tray_items.get(idx) {
                    Some(item) => match item.menu_path.clone() {
                        Some(path) => {
                            let ctx = self.menu_anchor_ctx(window);
                            menu_fetch_task(item.bus_name.clone(), path, ctx)
                        }
                        None => self.tray_call(idx, TrayMethod::ContextMenu),
                    },
                    None => Task::none(),
                }
            }
            Message::Ui(UiMessage::TrayScroll { idx, delta }) => {
                self.tray_call(idx, TrayMethod::Scroll(delta))
            }
            Message::Ui(UiMessage::MenuDismiss) => self.close_menus(),
            Message::Ui(UiMessage::MenuClick { level, id }) => {
                let addr = self
                    .menus
                    .get(level)
                    .map(|l| (l.bus_name.clone(), l.menu_path.clone()));
                let close = self.close_menus();
                match addr {
                    Some((bus, path)) => Task::batch([menu_event_task(bus, path, id), close]),
                    None => close,
                }
            }
            Message::Ui(UiMessage::MenuHover { level, id }) => self.menu_hover(level, id),
            Message::Ui(UiMessage::MenuLeave { level }) => self.close_from(level),
            Message::MenuOpen {
                bus_name,
                menu_path,
                root,
                ctx,
            } => self.open_root_menu(bus_name, menu_path, root, ctx),
            Message::MenuCloseAll => self.close_menus(),
            Message::CursorMoved { window, x, y } => {
                self.cursor.insert(window, (x, y));
                Task::none()
            }
            Message::PullTick(name) => match self.store.pulls().get(&name) {
                Some(decl) => {
                    run_pull_task(name.clone(), decl.command.clone(), decl.default.clone())
                }
                None => Task::none(),
            },
            Message::PullResult { name, value } => {
                if let Err(e) = self.store.update(&name, &value) {
                    log::debug!("pull {name} update rejected: {e}");
                }
                Task::none()
            }
            Message::SmartRefresh => {
                self.store.refresh();
                Task::none()
            }
            Message::Ipc { command, reply } => {
                let (response, task) = self.dispatch_command(command);
                if let Some(tx) = reply.lock().unwrap().take() {
                    let _ = tx.send(response);
                }
                task
            }
            Message::NotifRightClick(id) => match self.notif_windows.get(&id).copied() {
                Some(nid) => self.close_notification(nid, 2),
                None => Task::none(),
            },
            Message::Noop => Task::none(),
            _ => Task::none(),
        }
    }

    fn dispatch_command(&mut self, command: Command) -> (Response, Task<Message>) {
        match command {
            Command::Update { name, value } => match self.store.update(&name, &value) {
                Ok(()) => (Response::Ok, Task::none()),
                Err(e) => (Response::Error(e.to_string()), Task::none()),
            },
            Command::Open { window } => {
                if self.windows.values().any(|n| n == &window) {
                    return (Response::Ok, Task::none());
                }
                match self.store.resolved().widgets.get(&window) {
                    Some(w) => {
                        let settings = window::layer_settings_for(w);
                        let (id, task) = Message::layershell_open(settings);
                        self.windows.insert(id, window);
                        (Response::Ok, task)
                    }
                    None => (
                        Response::Error(format!("no such widget \"{window}\"")),
                        Task::none(),
                    ),
                }
            }
            Command::Close { window } => {
                match self
                    .windows
                    .iter()
                    .find(|(_, n)| *n == &window)
                    .map(|(id, _)| *id)
                {
                    Some(id) => {
                        self.windows.remove(&id);
                        (Response::Ok, Task::done(Message::RemoveWindow(id)))
                    }
                    None => (
                        Response::Error(format!("window \"{window}\" is not open")),
                        Task::none(),
                    ),
                }
            }
            Command::Toggle { window } => {
                let open_id = self
                    .windows
                    .iter()
                    .find(|(_, n)| *n == &window)
                    .map(|(id, _)| *id);
                match open_id {
                    Some(id) => {
                        self.windows.remove(&id);
                        (Response::Ok, Task::done(Message::RemoveWindow(id)))
                    }
                    None => self.dispatch_command(Command::Open { window }),
                }
            }
            Command::Reload => match self.store.reload(&self.config_path) {
                Ok(warns) => {
                    let task = self.reapply();
                    let resp = if warns.is_empty() {
                        Response::Ok
                    } else {
                        Response::Note(warns.join("\n"))
                    };
                    (resp, task)
                }
                Err(errs) => (Response::Error(errs.join("\n")), Task::none()),
            },
        }
    }

    fn on_notify(&mut self, n: Notification) -> Task<Message> {
        let id = n.notification_id;
        let settings = self.store.resolved().notification.clone();
        let icon = crate::notification::icons::resolve_icon(
            &n.app_icon,
            n.image_path.as_deref(),
            settings.height as u16,
        );
        let precalc = PreCalc::generate(&settings);
        let timer = timeout_task(id, n.expire_timeout, &settings);

        if self.notifications.contains_key(&id) {
            let st = self.notifications.get_mut(&id).unwrap();
            st.notification = n;
            st.icon = icon;
            st.precalc = precalc;
            return Task::batch([self.restack(), timer]);
        }

        self.notifications.insert(
            id,
            NotifState {
                notification: n,
                icon,
                precalc,
                window: None,
            },
        );
        if self.notif_windows.len() >= settings.max as usize {
            return Task::none();
        }
        let (wid, open_task) = Message::layershell_open(
            crate::daemon::notification::notif_layer_settings(&settings, 0),
        );
        if let Some(st) = self.notifications.get_mut(&id) {
            st.window = Some(wid);
        }
        self.notif_windows.insert(wid, id);
        Task::batch([open_task, self.restack(), timer])
    }

    fn close_notification(&mut self, id: u32, reason: u32) -> Task<Message> {
        let Some(st) = self.notifications.shift_remove(&id) else {
            return Task::none();
        };
        let mut tasks = vec![emit_closed_task(id, reason)];
        if let Some(wid) = st.window {
            self.notif_windows.remove(&wid);
            tasks.push(Task::done(Message::RemoveWindow(wid)));
        }
        tasks.push(self.promote_queued());
        tasks.push(self.restack());
        Task::batch(tasks)
    }

    fn promote_queued(&mut self) -> Task<Message> {
        let settings = self.store.resolved().notification.clone();
        if self.notif_windows.len() >= settings.max as usize {
            return Task::none();
        }
        let next = self
            .notifications
            .iter()
            .find(|(_, st)| st.window.is_none())
            .map(|(id, _)| *id);
        let Some(id) = next else {
            return Task::none();
        };
        let timeout = self
            .notifications
            .get(&id)
            .map(|s| s.notification.expire_timeout)
            .unwrap_or(-1);
        let (wid, open_task) = Message::layershell_open(
            crate::daemon::notification::notif_layer_settings(&settings, 0),
        );
        if let Some(st) = self.notifications.get_mut(&id) {
            st.window = Some(wid);
        }
        self.notif_windows.insert(wid, id);
        Task::batch([open_task, timeout_task(id, timeout, &settings)])
    }

    fn restack(&self) -> Task<Message> {
        let settings = self.store.resolved().notification.clone();
        let open: Vec<WindowId> = self
            .notifications
            .iter()
            .rev()
            .filter_map(|(_, st)| st.window)
            .collect();
        let tasks: Vec<Task<Message>> = open
            .iter()
            .enumerate()
            .map(|(slot, wid)| {
                let margin = crate::daemon::notification::margin_for_slot(&settings, slot);
                Task::done(Message::MarginChange { id: *wid, margin })
            })
            .collect();
        Task::batch(tasks)
    }

    fn menu_anchor_ctx(&self, window: WindowId) -> crate::daemon::menu::AnchorCtx {
        let cursor = self.cursor.get(&window).copied().unwrap_or((0.0, 0.0));
        let bar = self
            .windows
            .get(&window)
            .and_then(|name| self.store.resolved().widgets.get(name));
        crate::daemon::menu::AnchorCtx {
            bar_anchor: bar.and_then(|w| w.anchor),
            bar_margin: bar.and_then(|w| w.margin),
            bar_w: bar.and_then(|w| w.w).unwrap_or(0.0),
            bar_h: bar.and_then(|w| w.h).unwrap_or(0.0),
            cursor,
        }
    }

    fn open_root_menu(
        &mut self,
        bus_name: String,
        menu_path: String,
        root: crate::tray::menu_types::MenuItem,
        ctx: crate::daemon::menu::AnchorCtx,
    ) -> Task<Message> {
        let close = self.close_menus();
        let settings = self.store.resolved().apptray.clone();
        let items: Vec<_> = root.children.into_iter().filter(|i| i.visible).collect();
        let width = crate::render::menu::menu_pixel_width(&items, &settings) as u32;
        let height = crate::daemon::menu::menu_height(&items, settings.row_height);
        let placement = crate::daemon::menu::place_root(&ctx, width, height);
        let mut tasks = vec![close];
        let (oid, otask) = Message::layershell_open(overlay_settings());
        self.menu_overlay = Some(oid);
        tasks.push(otask);
        let (wid, mtask) = Message::layershell_open(menu_surface_settings(placement, true));
        self.menu_windows.insert(wid, 0);
        self.menus.push(crate::daemon::menu::MenuLevel {
            window: wid,
            bus_name,
            menu_path,
            items,
            placement,
        });
        tasks.push(mtask);
        Task::batch(tasks)
    }

    fn close_menus(&mut self) -> Task<Message> {
        let mut tasks = Vec::new();
        for lvl in self.menus.drain(..) {
            tasks.push(Task::done(Message::RemoveWindow(lvl.window)));
        }
        self.menu_windows.clear();
        if let Some(o) = self.menu_overlay.take() {
            tasks.push(Task::done(Message::RemoveWindow(o)));
        }
        Task::batch(tasks)
    }

    fn reapply(&mut self) -> Task<Message> {
        let mut tasks = vec![self.close_menus()];
        for (old_id, name) in std::mem::take(&mut self.windows) {
            tasks.push(Task::done(Message::RemoveWindow(old_id)));
            self.cursor.remove(&old_id);
            let settings = match self.store.resolved().widgets.get(&name) {
                Some(w) => window::layer_settings_for(w),
                None => continue,
            };
            let (new_id, open_task) = Message::layershell_open(settings);
            self.windows.insert(new_id, name);
            tasks.push(open_task);
        }
        for (name, decl) in self.store.pulls() {
            tasks.push(run_pull_task(
                name.clone(),
                decl.command.clone(),
                decl.default.clone(),
            ));
        }
        Task::batch(tasks)
    }

    fn menu_hover(&mut self, level: usize, id: i32) -> Task<Message> {
        let opens = self
            .menus
            .get(level)
            .and_then(|l| l.items.iter().find(|i| i.id == id))
            .map(|it| it.has_submenu && it.enabled)
            .unwrap_or(false);
        if opens {
            self.open_submenu(level, id)
        } else {
            self.close_from(level + 1)
        }
    }

    fn close_from(&mut self, level: usize) -> Task<Message> {
        let mut tasks = Vec::new();
        if level < self.menus.len() {
            for lvl in self.menus.split_off(level) {
                self.menu_windows.remove(&lvl.window);
                tasks.push(Task::done(Message::RemoveWindow(lvl.window)));
            }
        }
        Task::batch(tasks)
    }

    fn open_submenu(&mut self, level: usize, id: i32) -> Task<Message> {
        let settings = self.store.resolved().apptray.clone();
        let extracted = {
            let Some(parent) = self.menus.get(level) else {
                return Task::none();
            };
            let visible: Vec<&crate::tray::menu_types::MenuItem> =
                parent.items.iter().filter(|i| i.visible).collect();
            let Some(item_index) = visible.iter().position(|i| i.id == id) else {
                return Task::none();
            };
            let children: Vec<_> = visible[item_index]
                .children
                .iter()
                .filter(|c| c.visible)
                .cloned()
                .collect();
            if children.is_empty() {
                return Task::none();
            }
            let top_offset =
                crate::daemon::menu::row_top_offset(&parent.items, item_index, settings.row_height);
            (
                top_offset,
                children,
                parent.bus_name.clone(),
                parent.menu_path.clone(),
                parent.placement,
            )
        };
        let (top_offset, children, bus, path, parent_placement) = extracted;

        let mut tasks = Vec::new();
        for lvl in self.menus.split_off(level + 1) {
            self.menu_windows.remove(&lvl.window);
            tasks.push(Task::done(Message::RemoveWindow(lvl.window)));
        }
        let width = crate::render::menu::menu_pixel_width(&children, &settings) as u32;
        let height = crate::daemon::menu::menu_height(&children, settings.row_height);
        let placement =
            crate::daemon::menu::place_child(&parent_placement, top_offset, width, height);
        let (wid, mtask) = Message::layershell_open(menu_surface_settings(placement, false));
        let new_level = self.menus.len();
        self.menu_windows.insert(wid, new_level);
        self.menus.push(crate::daemon::menu::MenuLevel {
            window: wid,
            bus_name: bus,
            menu_path: path,
            items: children,
            placement,
        });
        tasks.push(mtask);
        Task::batch(tasks)
    }

    fn tray_call(&self, idx: usize, method: TrayMethod) -> Task<Message> {
        let Some(item) = self.tray_items.get(idx) else {
            return Task::none();
        };
        tray_method_task(item.bus_name.clone(), item.object_path.clone(), method)
    }

    fn view(&self, id: WindowId) -> Element<'_, Message> {
        if self.menu_overlay == Some(id) {
            let overlay: Element<'_, UiMessage> = iced::widget::mouse_area(
                iced::widget::container(iced::widget::Space::new())
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill),
            )
            .on_press(UiMessage::MenuDismiss)
            .on_right_press(UiMessage::MenuDismiss)
            .into();
            return overlay.map(Message::Ui);
        }
        if let Some(level) = self.menu_windows.get(&id) {
            if let Some(lvl) = self.menus.get(*level) {
                let settings = self.store.resolved().apptray.clone();
                return crate::render::menu::view_menu(&lvl.items, *level, &settings)
                    .map(Message::Ui);
            }
            return iced::widget::text("").into();
        }
        if let Some(nid) = self.notif_windows.get(&id) {
            if let Some(st) = self.notifications.get(nid) {
                let settings = self.store.resolved().notification.clone();
                return crate::render::notification::view_notification(
                    &settings,
                    &st.precalc,
                    &st.notification,
                    st.icon.as_deref(),
                )
                .map(Message::Ui);
            }
            return iced::widget::text("").into();
        }
        match self.windows.get(&id) {
            Some(name) => match self.store.resolved().widgets.get(name) {
                Some(w) => render::view_widget(
                    w,
                    &render::RenderCtx {
                        tray: &self.tray_items,
                        window: id,
                    },
                )
                .map(Message::Ui),
                None => iced::widget::text("").into(),
            },
            None => iced::widget::text("").into(),
        }
    }
}

fn menu_fetch_task(
    bus: String,
    path: String,
    ctx: crate::daemon::menu::AnchorCtx,
) -> Task<Message> {
    Task::perform(
        async move {
            let conn = crate::tray::subscription::connection()?;
            let proxy = crate::tray::dbusmenu::DBusMenuProxy::builder(&conn)
                .destination(bus.clone())
                .ok()?
                .path(path.clone())
                .ok()?
                .build()
                .await
                .ok()?;
            let _ = proxy.about_to_show(0).await;
            let (_rev, layout) = proxy.get_layout(0, -1, &[]).await.ok()?;
            let root = crate::tray::dbusmenu::parse_node(&layout);
            Some((bus, path, root))
        },
        move |res| match res {
            Some((bus_name, menu_path, root)) => Message::MenuOpen {
                bus_name,
                menu_path,
                root,
                ctx,
            },
            None => Message::Noop,
        },
    )
}

fn run_pull_task(name: String, command: String, default: String) -> Task<Message> {
    Task::perform(crate::daemon::pull::run(command, default), move |value| {
        Message::PullResult {
            name: name.clone(),
            value,
        }
    })
}

fn pointer_event(
    event: iced::Event,
    _status: iced::event::Status,
    window: iced::window::Id,
) -> Option<Message> {
    use iced::mouse::{Button, Event::ButtonReleased};
    match event {
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
            Some(Message::CursorMoved {
                window,
                x: position.x,
                y: position.y,
            })
        }
        iced::Event::Mouse(ButtonReleased(Button::Right)) => Some(Message::NotifRightClick(window)),
        _ => None,
    }
}

fn menu_event_task(bus: String, path: String, id: i32) -> Task<Message> {
    Task::perform(
        async move {
            let Some(conn) = crate::tray::subscription::connection() else {
                return;
            };
            let built = crate::tray::dbusmenu::DBusMenuProxy::builder(&conn)
                .destination(bus)
                .and_then(|b| b.path(path));
            if let Ok(b) = built
                && let Ok(proxy) = b.build().await
            {
                let data = zbus::zvariant::Value::I32(0);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as u32)
                    .unwrap_or(0);
                if let Err(e) = proxy.event(id, "clicked", &data, now).await {
                    log::warn!("dbusmenu event failed: {e}");
                }
            }
        },
        |_| Message::Noop,
    )
}

fn menu_surface_settings(
    p: crate::daemon::menu::Placement,
    keyboard: bool,
) -> iced_layershell::reexport::NewLayerShellSettings {
    use iced_layershell::reexport::{
        KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
    };
    NewLayerShellSettings {
        size: Some((p.width, p.height)),
        layer: Layer::Overlay,
        anchor: p.anchor(),
        exclusive_zone: Some(0),
        margin: Some(p.margin()),
        keyboard_interactivity: if keyboard {
            KeyboardInteractivity::OnDemand
        } else {
            KeyboardInteractivity::None
        },
        output_option: OutputOption::LastOutput,
        events_transparent: false,
        namespace: Some("iwwc".to_string()),
    }
}

fn overlay_settings() -> iced_layershell::reexport::NewLayerShellSettings {
    use iced_layershell::reexport::{
        Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
    };
    NewLayerShellSettings {
        size: Some((0, 0)),
        layer: Layer::Overlay,
        anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
        exclusive_zone: Some(0),
        margin: Some((0, 0, 0, 0)),
        keyboard_interactivity: KeyboardInteractivity::None,
        output_option: OutputOption::LastOutput,
        events_transparent: false,
        namespace: Some("iwwc".to_string()),
    }
}

enum TrayMethod {
    Activate,
    Secondary,
    ContextMenu,
    Scroll(f32),
}

fn tray_method_task(bus: String, path: String, method: TrayMethod) -> Task<Message> {
    Task::perform(
        async move {
            let Some(conn) = crate::tray::subscription::connection() else {
                return;
            };
            let built = crate::tray::proxy::StatusNotifierItemProxy::builder(&conn)
                .destination(bus)
                .and_then(|b| b.path(path));
            let proxy = match built {
                Ok(b) => match b.build().await {
                    Ok(p) => p,
                    Err(e) => {
                        log::warn!("tray proxy build: {e}");
                        return;
                    }
                },
                Err(e) => {
                    log::warn!("tray proxy addr: {e}");
                    return;
                }
            };
            let r = match method {
                TrayMethod::Activate => proxy.activate(0, 0).await,
                TrayMethod::Secondary => proxy.secondary_activate(0, 0).await,
                TrayMethod::ContextMenu => proxy.context_menu(0, 0).await,
                TrayMethod::Scroll(d) => {
                    proxy.scroll(if d > 0.0 { 1 } else { -1 }, "vertical").await
                }
            };
            if let Err(e) = r {
                log::warn!("tray method failed: {e}");
            }
        },
        |_| Message::Noop,
    )
}

fn timeout_task(
    id: u32,
    timeout: i32,
    settings: &crate::config::resolved::ResolvedNotificationSettings,
) -> Task<Message> {
    if timeout == 0 {
        return Task::none();
    }
    let dur = std::time::Duration::from_millis(if timeout < 0 {
        settings.timeout_ms as u64
    } else {
        timeout as u64
    });
    Task::perform(tokio::time::sleep(dur), move |_| Message::NotifTimeout(id))
}

fn emit_closed_task(id: u32, reason: u32) -> Task<Message> {
    Task::perform(
        async move {
            if let Some(conn) = crate::notification::subscription::connection()
                && let Err(e) = crate::notification::server::emit_closed(&conn, id, reason).await
            {
                log::warn!("emit NotificationClosed failed: {e}");
            }
        },
        |_| Message::Noop,
    )
}

fn emit_action_task(id: u32, key: String) -> Task<Message> {
    Task::perform(
        async move {
            if let Some(conn) = crate::notification::subscription::connection()
                && let Err(e) =
                    crate::notification::server::emit_action_invoked(&conn, id, &key).await
            {
                log::warn!("emit ActionInvoked failed: {e}");
            }
        },
        |_| Message::Noop,
    )
}
