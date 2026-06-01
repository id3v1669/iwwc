use crate::tray::menu_types::MenuItem;
use iced::window::Id as WindowId;
const SEPARATOR_HEIGHT: f32 = 7.0;

pub struct MenuLevel {
    pub window: WindowId,
    pub bus_name: String,
    pub menu_path: String,
    pub items: Vec<MenuItem>,
    pub width: u32,
    pub height: u32,
}

fn item_height(item: &MenuItem, row_height: f32) -> f32 {
    if item.separator {
        SEPARATOR_HEIGHT
    } else {
        row_height
    }
}

pub fn menu_height(items: &[MenuItem], row_height: f32) -> u32 {
    let h: f32 = items
        .iter()
        .filter(|i| i.visible)
        .map(|i| item_height(i, row_height))
        .sum();
    h.max(row_height) as u32
}

pub fn row_top_offset(items: &[MenuItem], visible_index: usize, row_height: f32) -> i32 {
    items
        .iter()
        .filter(|i| i.visible)
        .take(visible_index)
        .map(|i| item_height(i, row_height))
        .sum::<f32>() as i32
}

pub fn root_anchor_rect(cursor_x: f32, cursor_y: f32) -> (i32, i32, i32, i32) {
    (cursor_x as i32, cursor_y as i32, 1, 1)
}

pub fn submenu_anchor_rect(parent_width: u32, row_top: i32, row_height: f32) -> (i32, i32, i32, i32) {
    (0, row_top, parent_width as i32, row_height as i32)
}

#[derive(Debug, Clone, Copy)]
pub struct MenuAnchor {
    pub parent: WindowId,
    pub cursor: (f32, f32),
}
