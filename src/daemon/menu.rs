use crate::tray::menu_types::MenuItem;
use iced::window::Id as WindowId;
use iced_layershell::actions::IcedNewPopupSettings;
use iced_layershell::reexport::{PixelSize, PopupAnchor, PopupConstraintAdjustment, PopupGravity};

pub fn root_popup_settings(anchor: MenuAnchor, width: u32, height: u32) -> IcedNewPopupSettings {
    let (cx, cy) = anchor.cursor;
    IcedNewPopupSettings::at_position(
        anchor.parent,
        PixelSize::px(width.max(1), height.max(1)),
        (cx as i32, cy as i32),
    )
    .anchor(PopupAnchor::BottomLeft)
    .gravity(PopupGravity::BottomRight)
    .constraint_adjustment(
        PopupConstraintAdjustment::FlipX
            | PopupConstraintAdjustment::FlipY
            | PopupConstraintAdjustment::SlideX
            | PopupConstraintAdjustment::SlideY,
    )
}

pub fn submenu_popup_settings(
    parent: WindowId,
    width: u32,
    height: u32,
    top_offset: i32,
    parent_width: u32,
    row_height: f32,
) -> IcedNewPopupSettings {
    IcedNewPopupSettings::new(
        parent,
        PixelSize::px(width.max(1), height.max(1)),
        (0, top_offset),
        PixelSize::px(parent_width.max(1), (row_height as u32).max(1)),
    )
    .anchor(PopupAnchor::TopRight)
    .gravity(PopupGravity::BottomRight)
    .constraint_adjustment(PopupConstraintAdjustment::FlipX | PopupConstraintAdjustment::SlideY)
}

pub struct MenuLevel {
    pub window: WindowId,
    pub bus_name: String,
    pub menu_path: String,
    pub items: Vec<MenuItem>,
    pub width: u32,
    pub height: u32,
    pub active_child: Option<i32>,
}

pub fn row_top_offset(items: &[MenuItem], visible_index: usize, row_height: f32) -> i32 {
    items
        .iter()
        .filter(|i| i.visible)
        .take(visible_index)
        .map(|i| {
            if i.separator {
                row_height / 3.0
            } else {
                row_height
            }
        })
        .sum::<f32>() as i32
}

#[derive(Debug, Clone, Copy)]
pub struct MenuAnchor {
    pub parent: WindowId,
    pub cursor: (f32, f32),
}

pub fn visible_items(root: &MenuItem) -> Vec<MenuItem> {
    root.children
        .iter()
        .filter(|i| i.visible)
        .cloned()
        .collect()
}

pub fn submenu_items(items: &[MenuItem], id: i32) -> Option<Vec<MenuItem>> {
    let item = items.iter().find(|i| i.id == id)?;
    if !item.has_submenu || !item.enabled {
        return None;
    }
    let children: Vec<MenuItem> = item
        .children
        .iter()
        .filter(|c| c.visible)
        .cloned()
        .collect();
    if children.is_empty() {
        None
    } else {
        Some(children)
    }
}
