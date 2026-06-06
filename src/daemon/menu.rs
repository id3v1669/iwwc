use crate::tray::menu_types::MenuItem;
use iced::window::Id as WindowId;

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
