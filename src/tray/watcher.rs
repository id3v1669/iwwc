use std::sync::{Arc, Mutex};

use zbus::object_server::SignalEmitter;

#[derive(Clone, Default)]
pub struct Items(pub Arc<Mutex<Vec<String>>>);

pub struct Watcher {
    items: Items,
    notify: tokio::sync::mpsc::UnboundedSender<()>,
}

impl Watcher {
    pub fn new(items: Items, notify: tokio::sync::mpsc::UnboundedSender<()>) -> Self {
        Watcher { items, notify }
    }
}

#[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
impl Watcher {
    async fn register_status_notifier_item(
        &self,
        service: String,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
    ) -> zbus::fdo::Result<()> {
        let sender = hdr.sender().map(|s| s.to_string()).unwrap_or_default();
        let (bus, path) = crate::tray::types::parse_register_service(&service, &sender);
        let entry = format!("{bus}{path}");
        {
            let mut g = self.items.0.lock().unwrap();
            if !g.contains(&entry) {
                g.push(entry);
            }
        }
        let _ = self.notify.send(());
        Ok(())
    }

    async fn register_status_notifier_host(&self, _service: String) -> zbus::fdo::Result<()> {
        Ok(())
    }

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        self.items.0.lock().unwrap().clone()
    }

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn protocol_version(&self) -> i32 {
        0
    }

    #[zbus(signal)]
    async fn status_notifier_item_registered(
        emitter: &SignalEmitter<'_>,
        service: &str,
    ) -> zbus::Result<()>;
    #[zbus(signal)]
    async fn status_notifier_item_unregistered(
        emitter: &SignalEmitter<'_>,
        service: &str,
    ) -> zbus::Result<()>;
    #[zbus(signal)]
    async fn status_notifier_host_registered(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;
}
