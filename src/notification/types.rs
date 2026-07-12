use crate::config::resolved::ResolvedNotificationSettings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub app_name: String,
    pub app_icon: String,
    pub replaces_id: u32,
    pub summary: String,
    pub body: String,
    pub actions: Vec<String>,
    pub expire_timeout: i32,
    pub notification_id: u32,
    pub desktop_entry: String,
    pub image_path: Option<String>,
}

pub fn action_pairs(actions: &[String]) -> Vec<(String, String)> {
    actions
        .chunks_exact(2)
        .map(|c| (c[0].clone(), c[1].clone()))
        .collect()
}

pub fn default_action_key(actions: &[String]) -> Option<String> {
    action_pairs(actions)
        .into_iter()
        .find(|(id, _)| id == "default")
        .map(|(id, _)| id)
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreCalc {
    pub general_padding: f32,
    pub font_size_summary: f32,
    pub font_size_body: f32,
    pub image_size: f32,
    pub text_summary_paddings: iced::Padding,
    pub text_body_paddings: iced::Padding,
    pub text_paddings_block: iced::Padding,
}

impl PreCalc {
    pub fn generate(s: &ResolvedNotificationSettings) -> Self {
        let width = s.width;
        // #ratio
        let height = width * 0.275;
        PreCalc {
            general_padding: ((height * 0.15) as u16).min((width * 0.03) as u16) as f32,
            font_size_summary: ((height * 0.24) as u16)
                .min(((width - (height * 0.65)) * 0.06) as u16)
                as f32,
            font_size_body: ((height * 0.17) as u16).min(((width - (height * 0.65)) * 0.042) as u16)
                as f32,
            image_size: height * 0.65,
            text_summary_paddings: iced::Padding {
                top: 0.0,
                bottom: 0.0,
                left: (height * 0.05) + (height * 0.01),
                right: 0.0,
            },
            text_body_paddings: iced::Padding {
                top: 0.0,
                bottom: 0.0,
                left: height * 0.05,
                right: 0.0,
            },
            text_paddings_block: iced::Padding {
                top: height * 0.1,
                bottom: height * 0.1,
                left: height * 0.15,
                right: height * 0.15,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::resolved::ResolvedNotificationSettings;

    #[test]
    fn action_pairs_splits_flat_list() {
        let a = vec![
            "default".into(),
            "Open".into(),
            "reply".into(),
            "Reply".into(),
        ];
        assert_eq!(
            action_pairs(&a),
            vec![
                ("default".to_string(), "Open".to_string()),
                ("reply".to_string(), "Reply".to_string()),
            ]
        );
    }

    #[test]
    fn action_pairs_ignores_trailing_unpaired() {
        let a = vec!["a".into(), "A".into(), "dangling".into()];
        assert_eq!(action_pairs(&a), vec![("a".to_string(), "A".to_string())]);
    }

    #[test]
    fn default_action_detected() {
        let a = vec!["default".into(), "Open".into(), "x".into(), "X".into()];
        assert_eq!(default_action_key(&a), Some("default".to_string()));
        let b = vec!["x".into(), "X".into()];
        assert_eq!(default_action_key(&b), None);
    }

    #[test]
    fn precalc_matches_reference_formulas() {
        let s = ResolvedNotificationSettings {
            width: 400.0,
            ..Default::default()
        };
        let p = PreCalc::generate(&s);
        assert_eq!(p.image_size, 110.0 * 0.65);
        assert_eq!(
            p.general_padding,
            ((110.0_f32 * 0.15) as u16).min((400.0_f32 * 0.03) as u16) as f32
        );
        assert_eq!(
            p.font_size_summary,
            ((110.0_f32 * 0.24) as u16).min((((400.0_f32) - (110.0_f32 * 0.65)) * 0.06) as u16)
                as f32
        );
        assert_eq!(p.text_paddings_block.top, 110.0 * 0.1);
    }
}
