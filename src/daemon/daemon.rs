//func to launch the daemon

use crate::shared_data::GLOBAL_DATA_MAP;

pub async fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, mut receiver) = tokio::sync::mpsc::channel(5);
    
    // needed to avoid GDBus.Error:org.freedesktop.DBus.Error.ServiceUnknown
    let _connection = connect_dbus(sender).await?;

    tokio::spawn(async move {

        while let Some(action) = receiver.recv().await {
            match action {
                crate::daemon::nf_struct::NotificationAction::ActionInvoked { notification_id } => {
                    log::debug!("NotificationAction::ActionInvoked: {}", notification_id);
                    // for debug purposes run command
                    let _ = tokio::process::Command::new("eww")
                        .arg("update")
                        .arg("testvar=true")
                        .output()
                        .await;
                }
                crate::daemon::nf_struct::NotificationAction::ActionClose {
                    notification_id,
                    reason,
                } => {
                    log::debug!(
                        "NotificationAction::ActionClose: {} {}",
                        notification_id,
                        reason
                    );
                    // for debug purposes run command
                    let _ = tokio::process::Command::new("eww")
                        .arg("update")
                        .arg("testvar=true")
                        .output()
                        .await;
                }
                crate::daemon::nf_struct::NotificationAction::Notify { notification } => {
                    log::debug!(
                        "NotificationAction::Notify all options of notification: {:?}", notification
                    );
                    log::debug!(
                        "NotificationAction::Notify app_name: {:?}",
                        notification.app_name
                    );
                    log::debug!(
                        "NotificationAction::Notify icon: {:?}",
                        notification.app_icon
                    );
                    log::debug!(
                        "NotificationAction::Notify replaces_id: {:?}",
                        notification.replaces_id
                    );
                    log::debug!(
                        "NotificationAction::Notify summary: {:?}",
                        notification.summary
                    );
                    log::debug!(
                        "NotificationAction::Notify body: {:?}",
                        notification.body
                    );
                    log::debug!(
                        "NotificationAction::Notify actions: {:?}",
                        notification.actions
                    );
                    log::debug!(
                        "NotificationAction::Notify expire_timeout: {:?}",
                        notification.expire_timeout
                    );
                    log::debug!(
                        "NotificationAction::Notify notification_id: {:?}",
                        notification.notification_id
                    );
                    log::debug!(
                        "NotificationAction::Notify desktop_entry: {:?}",
                        notification.desktop_entry
                    );
                    let id: Option<String> = Some(notification.notification_id.to_string().clone());
                    let mut data_map = GLOBAL_DATA_MAP.lock().unwrap();
                    data_map.insert(id.clone(), notification);
                    log::debug!("NotificationAction::Notify before gen_ui");
        
                    tokio::spawn(async move {
                        crate::notification::simple::gen_ui(400, 100, id.clone()).await.unwrap();
                    });
                    log::debug!("NotificationAction::Notify after gen_ui");

                }
                crate::daemon::nf_struct::NotificationAction::Close { notification_id } => {
                    log::debug!("NotificationAction::Close: {}", notification_id);
                    // for debug purposes run command
                    let _ = tokio::process::Command::new("eww")
                        .arg("update")
                        .arg("testvar=true")
                        .output()
                        .await;
                }
            }
        }
    });
    std::future::pending::<()>().await;
    Ok(())
}

pub async fn connect_dbus(
    sender: tokio::sync::mpsc::Sender<crate::daemon::nf_struct::NotificationAction>,
) -> Result<zbus::Connection, Box<dyn std::error::Error>> {
    let handler = crate::daemon::nf_handler::NotificationHandler::new(sender);
    let conn = zbus::connection::Builder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", handler)?
        .build()
        .await?;
    Ok(conn)
}
