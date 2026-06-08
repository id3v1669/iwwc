use crate::tray::menu_types::{MenuIcon, MenuItem, Toggle, strip_mnemonic};
use std::collections::HashMap;
use zbus::zvariant::{OwnedValue, Value};

type LayoutNode = (i32, HashMap<String, OwnedValue>, Vec<OwnedValue>);

#[zbus::proxy(interface = "com.canonical.dbusmenu")]
pub trait DBusMenu {
    fn get_layout(
        &self,
        parent_id: i32,
        recursion_depth: i32,
        property_names: &[&str],
    ) -> zbus::Result<(u32, LayoutNode)>;
    fn about_to_show(&self, id: i32) -> zbus::Result<bool>;
    fn event(&self, id: i32, event_id: &str, data: &Value<'_>, timestamp: u32) -> zbus::Result<()>;
    #[zbus(signal)]
    fn layout_updated(&self, revision: u32, parent: i32) -> zbus::Result<()>;
}

fn str_prop(props: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
    props.get(key).and_then(|v| String::try_from(&**v).ok())
}
fn bool_prop(props: &HashMap<String, OwnedValue>, key: &str, default: bool) -> bool {
    props
        .get(key)
        .and_then(|v| bool::try_from(&**v).ok())
        .unwrap_or(default)
}
fn i32_prop(props: &HashMap<String, OwnedValue>, key: &str) -> Option<i32> {
    props.get(key).and_then(|v| i32::try_from(&**v).ok())
}
fn bytes_prop(props: &HashMap<String, OwnedValue>, key: &str) -> Option<Vec<u8>> {
    props
        .get(key)
        .and_then(|v| Vec::<u8>::try_from(v.try_clone().ok()?).ok())
}

pub fn parse_node(node: &LayoutNode) -> MenuItem {
    let (id, props, raw_children) = node;
    let toggle = match str_prop(props, "toggle-type").as_deref() {
        Some("checkmark") => Toggle::Check(i32_prop(props, "toggle-state").unwrap_or(0) == 1),
        Some("radio") => Toggle::Radio(i32_prop(props, "toggle-state").unwrap_or(0) == 1),
        _ => Toggle::None,
    };
    let icon = if let Some(name) = str_prop(props, "icon-name").filter(|s| !s.is_empty()) {
        MenuIcon::Name(name)
    } else if let Some(data) = bytes_prop(props, "icon-data") {
        MenuIcon::Png(iced::widget::image::Handle::from_bytes(data))
    } else {
        MenuIcon::None
    };
    let mut children = Vec::new();
    for child in raw_children {
        if let Ok(cn) = <LayoutNode>::try_from(&**child) {
            children.push(parse_node(&cn));
        }
    }
    let has_submenu =
        str_prop(props, "children-display").as_deref() == Some("submenu") || !children.is_empty();
    MenuItem {
        id: *id,
        label: strip_mnemonic(&str_prop(props, "label").unwrap_or_default()),
        enabled: bool_prop(props, "enabled", true),
        visible: bool_prop(props, "visible", true),
        separator: str_prop(props, "type").as_deref() == Some("separator"),
        toggle,
        icon,
        has_submenu,
        children,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zbus::zvariant::Value;

    fn owned(v: Value<'static>) -> OwnedValue {
        OwnedValue::try_from(v).unwrap()
    }

    fn node(id: i32, props: &[(&str, Value<'static>)], children: Vec<OwnedValue>) -> LayoutNode {
        let map: HashMap<String, OwnedValue> = props
            .iter()
            .cloned()
            .map(|(k, v)| (k.to_string(), owned(v)))
            .collect();
        (id, map, children)
    }

    #[test]
    fn defaults_when_props_absent() {
        let m = parse_node(&node(7, &[("label", Value::from("Bare"))], vec![]));
        assert!(m.enabled);
        assert!(m.visible);
        assert_eq!(m.toggle, Toggle::None);
        assert!(matches!(m.icon, MenuIcon::None));
        assert!(!m.has_submenu);
    }

    #[test]
    fn radio_off_and_invisible_and_disabled() {
        let m = parse_node(&node(
            8,
            &[
                ("toggle-type", Value::from("radio")),
                ("toggle-state", Value::from(0i32)),
                ("enabled", Value::from(false)),
                ("visible", Value::from(false)),
            ],
            vec![],
        ));
        assert_eq!(m.toggle, Toggle::Radio(false));
        assert!(!m.enabled);
        assert!(!m.visible);
    }

    #[test]
    fn icon_name_preferred_then_png_data() {
        let by_name = parse_node(&node(9, &[("icon-name", Value::from("firefox"))], vec![]));
        assert!(matches!(by_name.icon, MenuIcon::Name(ref n) if n == "firefox"));

        let by_data = parse_node(&node(
            10,
            &[("icon-data", Value::from(vec![1u8, 2, 3, 4]))],
            vec![],
        ));
        assert!(
            matches!(by_data.icon, MenuIcon::Png(_)),
            "expected png icon, got {:?}",
            by_data.icon
        );
    }

    #[test]
    fn has_submenu_from_children_or_marker() {
        let with_children = parse_node(&node(
            11,
            &[("label", Value::from("Sub"))],
            vec![owned(Value::from(node(
                12,
                &[("label", Value::from("Inner"))],
                vec![],
            )))],
        ));
        assert!(with_children.has_submenu);
        assert_eq!(with_children.children.len(), 1);
        assert_eq!(with_children.children[0].label, "Inner");

        let by_marker = parse_node(&node(
            13,
            &[("children-display", Value::from("submenu"))],
            vec![],
        ));
        assert!(by_marker.has_submenu);
    }

    #[test]
    fn nesting_recurses_grandchildren() {
        let root = node(
            0,
            &[],
            vec![owned(Value::from(node(
                2,
                &[("label", Value::from("A"))],
                vec![owned(Value::from(node(
                    3,
                    &[("label", Value::from("B"))],
                    vec![owned(Value::from(node(
                        4,
                        &[("label", Value::from("C"))],
                        vec![],
                    )))],
                )))],
            )))],
        );
        let m = parse_node(&root);
        assert_eq!(m.children[0].children[0].children[0].label, "C");
    }
}
