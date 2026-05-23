#[zbus::proxy(
    interface = "org.kde.StatusNotifierItem",
    default_path = "/StatusNotifierItem"
)]
pub trait StatusNotifierItem {
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn title(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn status(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn icon_name(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn icon_theme_path(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn icon_pixmap(&self) -> zbus::Result<Vec<(i32, i32, Vec<u8>)>>;
    #[zbus(property)]
    fn item_is_menu(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn menu(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    fn activate(&self, x: i32, y: i32) -> zbus::Result<()>;
    fn secondary_activate(&self, x: i32, y: i32) -> zbus::Result<()>;
    fn context_menu(&self, x: i32, y: i32) -> zbus::Result<()>;
    fn scroll(&self, delta: i32, orientation: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    fn new_icon(&self) -> zbus::Result<()>;
    #[zbus(signal)]
    fn new_status(&self, status: String) -> zbus::Result<()>;
    #[zbus(signal)]
    fn new_title(&self) -> zbus::Result<()>;
    #[zbus(signal)]
    fn new_tool_tip(&self) -> zbus::Result<()>;
}
