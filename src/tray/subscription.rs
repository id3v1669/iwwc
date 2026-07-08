use std::sync::OnceLock;

use futures::sink::SinkExt;
use futures::stream::{StreamExt, select_all};
use iced::Subscription;
use zbus::Connection;

use crate::daemon::Message;
use crate::tray::dbusmenu::DBusMenuProxy;
use crate::tray::host::build_item;
use crate::tray::proxy::StatusNotifierItemProxy;
use crate::tray::watcher::{Items, Watcher};

static CONNECTION: OnceLock<Connection> = OnceLock::new();

pub fn connection() -> Option<Connection> {
    CONNECTION.get().cloned()
}

const WATCHER_NAME: &str = "org.kde.StatusNotifierWatcher";
const WATCHER_PATH: &str = "/StatusNotifierWatcher";

#[zbus::proxy(
    interface = "org.kde.StatusNotifierWatcher",
    default_service = "org.kde.StatusNotifierWatcher",
    default_path = "/StatusNotifierWatcher"
)]
pub trait StatusNotifierWatcher {
    fn register_status_notifier_host(&self, service: &str) -> zbus::Result<()>;
    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> zbus::Result<Vec<String>>;
}

fn split_entry(entry: &str) -> (String, String) {
    match entry.find('/') {
        Some(i) => (entry[..i].to_string(), entry[i..].to_string()),
        None => (entry.to_string(), "/StatusNotifierItem".to_string()),
    }
}

fn tray_stream(args: &(Option<String>, u16)) -> futures::stream::BoxStream<'static, Message> {
    let (icon_theme, icon_size) = args.clone();
    iced::stream::channel(16, async move |mut output| {
        let conn = match Connection::session().await {
            Ok(c) => c,
            Err(e) => {
                log::error!("tray: no session bus: {e}");
                return;
            }
        };
        let items = Items::default();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();

        let dbus = match zbus::fdo::DBusProxy::new(&conn).await {
            Ok(d) => d,
            Err(e) => {
                log::error!("tray: dbus proxy: {e}");
                return;
            }
        };
        let watcher_taken = match WATCHER_NAME.try_into() {
            Ok(n) => dbus.name_has_owner(n).await.unwrap_or(false),
            Err(_) => false,
        };

        if !watcher_taken {
            let _ = conn
                .object_server()
                .at(WATCHER_PATH, Watcher::new(items.clone(), tx.clone()))
                .await;
            let _ = conn.request_name(WATCHER_NAME).await;
        }
        let _ = CONNECTION.set(conn.clone());
        log::debug!("tray: watcher_taken={watcher_taken}");

        if let Ok(w) = StatusNotifierWatcherProxy::new(&conn).await {
            let host = format!("org.kde.StatusNotifierHost-{}", std::process::id());
            let _ = w.register_status_notifier_host(&host).await;
            if watcher_taken && let Ok(list) = w.registered_status_notifier_items().await {
                let mut g = items.0.lock().unwrap();
                for e in list {
                    if !g.contains(&e) {
                        g.push(e);
                    }
                }
            }
        }

        let mut noc = match dbus.receive_name_owner_changed().await {
            Ok(s) => s,
            Err(e) => {
                log::error!("tray: name_owner_changed: {e}");
                return;
            }
        };

        'outer: loop {
            let entries = { items.0.lock().unwrap().clone() };
            let mut tray_items = Vec::new();
            for e in &entries {
                if let Some(item) = build_item(&conn, e, icon_size, icon_theme.as_deref()).await {
                    tray_items.push(item);
                }
            }
            let menu_keys: Vec<(String, String)> = tray_items
                .iter()
                .filter_map(|i| i.menu_path.clone().map(|p| (i.bus_name.clone(), p)))
                .collect();
            if output.send(Message::TrayItems(tray_items)).await.is_err() {
                break;
            }

            let mut proxies = Vec::new();
            for entry in &entries {
                let (bus, path) = split_entry(entry);
                if let Ok(b) = StatusNotifierItemProxy::builder(&conn).destination(bus)
                    && let Ok(b) = b.path(path)
                    && let Ok(p) = b.build().await
                {
                    proxies.push(p);
                }
            }
            let mut sigs = Vec::new();
            for p in &proxies {
                if let Ok(s) = p.receive_new_icon().await {
                    sigs.push(s.map(|_| ()).boxed());
                }
                if let Ok(s) = p.receive_new_status().await {
                    sigs.push(s.map(|_| ()).boxed());
                }
                if let Ok(s) = p.receive_new_title().await {
                    sigs.push(s.map(|_| ()).boxed());
                }
            }
            let mut merged = select_all(sigs);

            let mut menu_sigs = Vec::new();
            for (bus, path) in &menu_keys {
                if let Ok(b) = DBusMenuProxy::builder(&conn).destination(bus.clone())
                    && let Ok(b) = b.path(path.clone())
                    && let Ok(p) = b.build().await
                    && let Ok(s) = p.receive_layout_updated().await
                {
                    let bus = bus.clone();
                    let path = path.clone();
                    menu_sigs.push(s.map(move |_| (bus.clone(), path.clone())).boxed());
                }
            }
            let mut menu_merged = select_all(menu_sigs);

            loop {
                tokio::select! {
                    _ = rx.recv() => { continue 'outer; }
                    ev = noc.next() => {
                        let Some(ev) = ev else { break 'outer; };
                        if let Ok(args) = ev.args()
                            && args.new_owner().is_none()
                        {
                            let prefix = format!("{}/", args.name().as_str());
                            let mut g = items.0.lock().unwrap();
                            g.retain(|e| !e.starts_with(&prefix));
                        }
                        continue 'outer;
                    }
                    _ = merged.next(), if !merged.is_empty() => { continue 'outer; }
                    key = menu_merged.next(), if !menu_merged.is_empty() => {
                        if let Some((bus_name, menu_path)) = key
                            && output
                                .send(Message::MenuRefetch { bus_name, menu_path })
                                .await
                                .is_err()
                        {
                            break 'outer;
                        }
                    }
                }
            }
        }
    })
    .boxed()
}

pub fn subscription(icon_theme: Option<String>, icon_size: u16) -> Subscription<Message> {
    Subscription::run_with((icon_theme, icon_size), tray_stream)
}
