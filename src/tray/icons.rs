use crate::tray::types::TrayIcon;
use std::path::PathBuf;

pub fn argb32_to_rgba8(argb: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(argb.len());
    for px in argb.chunks_exact(4) {
        let (a, r, g, b) = (px[0], px[1], px[2], px[3]);
        out.extend_from_slice(&[r, g, b, a]);
    }
    out
}

pub fn pick_pixmap(pixmaps: &[(i32, i32, Vec<u8>)], target: u32) -> Option<&(i32, i32, Vec<u8>)> {
    let target = target as i32;
    pixmaps
        .iter()
        .filter(|p| p.0 >= target)
        .min_by_key(|p| p.0)
        .or_else(|| pixmaps.iter().max_by_key(|p| p.0))
}

pub fn resolve_icon(
    icon_name: &str,
    theme_path: &str,
    pixmaps: &[(i32, i32, Vec<u8>)],
    size: u16,
) -> TrayIcon {
    if !icon_name.is_empty() {
        if !theme_path.is_empty() {
            let direct = PathBuf::from(theme_path).join(format!("{icon_name}.png"));
            if direct.is_file() {
                return TrayIcon::Path(direct);
            }
            let svg = PathBuf::from(theme_path).join(format!("{icon_name}.svg"));
            if svg.is_file() {
                return TrayIcon::Path(svg);
            }
        }
        if let Some(p) = cosmic_freedesktop_icons::lookup(icon_name)
            .with_size(size)
            .with_cache()
            .find()
        {
            return TrayIcon::Path(p);
        }
    }
    if let Some((w, h, argb)) = pick_pixmap(pixmaps, size as u32) {
        return TrayIcon::Pixmap {
            w: *w as u32,
            h: *h as u32,
            rgba: argb32_to_rgba8(argb),
        };
    }
    TrayIcon::None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argb_to_rgba_reorders_one_pixel() {
        let argb = vec![0xff, 0xff, 0x00, 0x00];
        assert_eq!(argb32_to_rgba8(&argb), vec![0xff, 0x00, 0x00, 0xff]);
    }

    #[test]
    fn pick_pixmap_prefers_smallest_ge_target() {
        let pms = vec![
            (16, 16, vec![0u8; 16 * 16 * 4]),
            (32, 32, vec![0u8; 32 * 32 * 4]),
            (64, 64, vec![0u8; 64 * 64 * 4]),
        ];
        let chosen = pick_pixmap(&pms, 24).unwrap();
        assert_eq!((chosen.0, chosen.1), (32, 32));
    }

    #[test]
    fn pick_pixmap_falls_back_to_largest_when_all_smaller() {
        let pms = vec![
            (8, 8, vec![0u8; 8 * 8 * 4]),
            (16, 16, vec![0u8; 16 * 16 * 4]),
        ];
        let chosen = pick_pixmap(&pms, 64).unwrap();
        assert_eq!((chosen.0, chosen.1), (16, 16));
    }

    #[test]
    fn pick_pixmap_empty_is_none() {
        let pms: Vec<(i32, i32, Vec<u8>)> = vec![];
        assert!(pick_pixmap(&pms, 24).is_none());
    }
}
