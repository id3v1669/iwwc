use zbus::Connection;

use crate::tray::proxy::StatusNotifierItemProxy;
use crate::tray::types::TrayItem;

fn split_entry(entry: &str) -> (String, String) {
    match entry.find('/') {
        Some(i) => (entry[..i].to_string(), entry[i..].to_string()),
        None => (entry.to_string(), "/StatusNotifierItem".to_string()),
    }
}

pub async fn build_item(conn: &Connection, entry: &str, icon_size: u16) -> Option<TrayItem> {
    let (bus_name, object_path) = split_entry(entry);
    let proxy = StatusNotifierItemProxy::builder(conn)
        .destination(bus_name.clone())
        .ok()?
        .path(object_path.clone())
        .ok()?
        .build()
        .await
        .ok()?;
    let id = proxy.id().await.unwrap_or_default();
    let title = proxy.title().await.unwrap_or_default();
    let status = proxy
        .status()
        .await
        .unwrap_or_else(|_| "Active".to_string());
    let icon_name = proxy.icon_name().await.unwrap_or_default();
    let theme_path = proxy.icon_theme_path().await.unwrap_or_default();
    let pixmaps = proxy.icon_pixmap().await.unwrap_or_default();
    let menu_path = proxy.menu().await.ok().map(|p| p.as_str().to_string());
    let icon = crate::tray::icons::resolve_icon(&icon_name, &theme_path, &pixmaps, icon_size);
    Some(TrayItem {
        bus_name,
        object_path,
        id,
        title,
        status,
        icon,
        menu_path,
    })
}

pub async fn snapshot(conn: &Connection, entries: &[String], icon_size: u16) -> Vec<TrayItem> {
    let mut out = Vec::new();
    for e in entries {
        if let Some(item) = build_item(conn, e, icon_size).await {
            out.push(item);
        }
    }
    out
}
