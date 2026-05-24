use crate::config::types::{Anchor as CfgAnchor, Edges};
use crate::tray::menu_types::MenuItem;
use iced::window::Id as WindowId;
use iced_layershell::reexport::Anchor as LayerAnchor;
const SEPARATOR_HEIGHT: f32 = 7.0;

#[derive(Debug, Clone, Copy)]
pub struct Placement {
    pub top: bool,
    pub left: bool,
    pub v_margin: i32,
    pub h_margin: i32,
    pub width: u32,
    pub height: u32,
}

impl Placement {
    pub fn anchor(&self) -> LayerAnchor {
        let v = if self.top {
            LayerAnchor::Top
        } else {
            LayerAnchor::Bottom
        };
        let h = if self.left {
            LayerAnchor::Left
        } else {
            LayerAnchor::Right
        };
        v | h
    }

    pub fn margin(&self) -> (i32, i32, i32, i32) {
        let top = if self.top { self.v_margin } else { 0 };
        let bottom = if self.top { 0 } else { self.v_margin };
        let left = if self.left { self.h_margin } else { 0 };
        let right = if self.left { 0 } else { self.h_margin };
        (top, right, bottom, left)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AnchorCtx {
    pub bar_anchor: Option<CfgAnchor>,
    pub bar_margin: Option<Edges>,
    pub bar_w: f32,
    pub bar_h: f32,
    pub screen_w: f32,
    pub screen_h: f32,
    pub cursor: (f32, f32),
}

pub struct MenuLevel {
    pub window: WindowId,
    pub bus_name: String,
    pub menu_path: String,
    pub items: Vec<MenuItem>,
    pub placement: Placement,
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

pub fn place_root(ctx: &AnchorCtx, width: u32, height: u32) -> Placement {
    let a = ctx.bar_anchor.unwrap_or(CfgAnchor {
        top: true,
        bottom: false,
        left: true,
        right: false,
    });
    let m = ctx.bar_margin.unwrap_or(Edges::all(0.0));
    let (cx, cy) = ctx.cursor;
    let bar_left = if a.left {
        m.left
    } else if a.right {
        ctx.screen_w - m.right - ctx.bar_w
    } else {
        (ctx.screen_w - ctx.bar_w) / 2.0
    };
    let bar_top = if a.top {
        m.top
    } else if a.bottom {
        ctx.screen_h - m.bottom - ctx.bar_h
    } else {
        (ctx.screen_h - ctx.bar_h) / 2.0
    };

    let (left, h_margin) = fit_axis(
        !a.right || a.left,
        bar_left + cx,
        width as f32,
        ctx.screen_w,
    );
    let (top, v_margin) = fit_axis(
        !a.bottom || a.top,
        bar_top + cy,
        height as f32,
        ctx.screen_h,
    );

    Placement {
        top,
        left,
        v_margin: v_margin.max(0.0) as i32,
        h_margin: h_margin.max(0.0) as i32,
        width,
        height,
    }
}

fn fit_axis(prefer_high: bool, cursor: f32, size: f32, screen: f32) -> (bool, f32) {
    let fits_high = screen <= 0.0 || cursor + size <= screen;
    let fits_low = screen <= 0.0 || cursor - size >= 0.0;
    let grow_high = if prefer_high {
        if fits_high {
            true
        } else if fits_low {
            false
        } else {
            (screen - cursor) >= cursor
        }
    } else if fits_low {
        false
    } else if fits_high {
        true
    } else {
        (screen - cursor) >= cursor
    };
    if grow_high {
        (true, cursor)
    } else {
        (false, screen - cursor)
    }
}

pub fn place_child(parent: &Placement, row_top: i32, width: u32, height: u32) -> Placement {
    let h_margin = parent.h_margin + parent.width as i32;
    let v_margin = if parent.top {
        parent.v_margin + row_top
    } else {
        (parent.v_margin + parent.height as i32 - row_top - height as i32).max(0)
    };
    Placement {
        top: parent.top,
        left: parent.left,
        v_margin,
        h_margin,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tray::menu_types::{MenuIcon, MenuItem, Toggle};

    fn item(separator: bool) -> MenuItem {
        MenuItem {
            id: 0,
            label: String::new(),
            enabled: true,
            visible: true,
            separator,
            toggle: Toggle::None,
            icon: MenuIcon::None,
            has_submenu: false,
            children: vec![],
        }
    }

    fn ctx(anchor: CfgAnchor, cursor: (f32, f32)) -> AnchorCtx {
        AnchorCtx {
            bar_anchor: Some(anchor),
            bar_margin: Some(Edges::all(8.0)),
            bar_w: 400.0,
            bar_h: 36.0,
            screen_w: 2000.0,
            screen_h: 2000.0,
            cursor,
        }
    }

    #[test]
    fn row_offset_accounts_for_short_separators() {
        let items = vec![item(false), item(true), item(false), item(false)];
        assert_eq!(row_top_offset(&items, 3, 26.0), 59);
    }

    #[test]
    fn top_right_bar_anchors_top_right_at_cursor() {
        let p = place_root(
            &ctx(
                CfgAnchor {
                    top: true,
                    bottom: false,
                    left: false,
                    right: true,
                },
                (350.0, 20.0),
            ),
            200,
            100,
        );
        assert!(p.top && !p.left);
        assert_eq!(p.v_margin, 8 + 20); // top margin + cursor.y
        assert_eq!(p.h_margin, 8 + (400 - 350)); // right margin + (bar_w - cursor.x)
    }

    #[test]
    fn bottom_left_bar_anchors_bottom_left_at_cursor() {
        let p = place_root(
            &ctx(
                CfgAnchor {
                    top: false,
                    bottom: true,
                    left: true,
                    right: false,
                },
                (40.0, 10.0),
            ),
            200,
            100,
        );
        assert!(!p.top && p.left);
        assert_eq!(p.v_margin, 8 + (36 - 10)); // bottom margin + (bar_h - cursor.y)
        assert_eq!(p.h_margin, 8 + 40); // left margin + cursor.x
    }

    #[test]
    fn child_extends_in_parent_direction() {
        let parent = Placement {
            top: true,
            left: false,
            v_margin: 28,
            h_margin: 58,
            width: 200,
            height: 100,
        };
        let c = place_child(&parent, 26, 180, 80);
        assert!(c.top && !c.left);
        assert_eq!(c.h_margin, 58 + 200); // past the parent's width
        assert_eq!(c.v_margin, 28 + 26); // parent margin + row offset (top-anchored)
    }

    #[test]
    fn child_vertical_for_bottom_anchored_parent() {
        let parent = Placement {
            top: false,
            left: true,
            v_margin: 30,
            h_margin: 50,
            width: 200,
            height: 100,
        };
        let c = place_child(&parent, 26, 180, 80);
        assert_eq!(c.v_margin, 30 + 100 - 26 - 80);
    }

    #[test]
    fn full_width_bar_flips_menu_left_near_right_edge() {
        let c = AnchorCtx {
            bar_anchor: Some(CfgAnchor {
                top: true,
                bottom: false,
                left: true,
                right: true,
            }),
            bar_margin: Some(Edges::all(8.0)),
            bar_w: 0.0,
            bar_h: 30.0,
            screen_w: 1000.0,
            screen_h: 1000.0,
            cursor: (950.0, 10.0),
        };
        let p = place_root(&c, 200, 300);
        assert!(!p.left);
        assert_eq!(p.h_margin, 1000 - (8 + 950));
    }

    #[test]
    fn full_height_sidebar_flips_menu_up_near_bottom_edge() {
        let c = AnchorCtx {
            bar_anchor: Some(CfgAnchor {
                top: true,
                bottom: true,
                left: true,
                right: false,
            }),
            bar_margin: Some(Edges::all(8.0)),
            bar_w: 30.0,
            bar_h: 0.0,
            screen_w: 1000.0,
            screen_h: 1000.0,
            cursor: (10.0, 950.0),
        };
        let p = place_root(&c, 240, 300);
        assert!(!p.top);
        assert_eq!(p.v_margin, 1000 - (8 + 950));
        assert!(p.left);
    }

    #[test]
    fn full_height_sidebar_top_tray_expands_down() {
        let c = AnchorCtx {
            bar_anchor: Some(CfgAnchor {
                top: true,
                bottom: true,
                left: true,
                right: false,
            }),
            bar_margin: Some(Edges::all(8.0)),
            bar_w: 30.0,
            bar_h: 0.0,
            screen_w: 1000.0,
            screen_h: 1000.0,
            cursor: (10.0, 20.0),
        };
        let p = place_root(&c, 240, 300);
        assert!(p.top);
        assert_eq!(p.v_margin, 8 + 20);
    }

    #[test]
    fn unknown_screen_uses_natural_direction() {
        let c = AnchorCtx {
            bar_anchor: Some(CfgAnchor {
                top: true,
                bottom: false,
                left: true,
                right: true,
            }),
            bar_margin: Some(Edges::all(8.0)),
            bar_w: 0.0,
            bar_h: 30.0,
            screen_w: 0.0,
            screen_h: 0.0,
            cursor: (950.0, 10.0),
        };
        let p = place_root(&c, 200, 300);
        assert!(p.left);
        assert_eq!(p.h_margin, 8 + 950);
    }
}
