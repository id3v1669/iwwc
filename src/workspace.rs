use std::sync::{Arc, Mutex};

use iced_wayland_subscriber::workspace::{
    State, WorkspaceCapabilities, WorkspaceEvent, WorkspaceId, WorkspaceSnapshot,
};
use wayland_client::Connection;

use crate::config::types::VarValue;
use crate::daemon::Message;

pub const MAX_SLOTS: i128 = 64;

static SNAPSHOT: Mutex<Option<Arc<WorkspaceSnapshot>>> = Mutex::new(None);

pub fn store(snapshot: Option<Arc<WorkspaceSnapshot>>) {
    *SNAPSHOT.lock().unwrap() = snapshot;
}

fn snapshot() -> Option<Arc<WorkspaceSnapshot>> {
    SNAPSHOT.lock().unwrap().clone()
}

pub fn subscription(connection: Connection) -> iced::Subscription<Message> {
    iced_wayland_subscriber::workspace::listen(connection).map(|event| match event {
        WorkspaceEvent::Updated(snapshot) => Message::Workspaces(Some(snapshot)),
        WorkspaceEvent::Unsupported => {
            log::info!("compositor does not support ext-workspace, workspace variables stay empty");
            Message::Workspaces(None)
        }
        WorkspaceEvent::Finished => {
            log::warn!("compositor ended the workspace protocol");
            Message::Workspaces(None)
        }
        WorkspaceEvent::Stop(e) => {
            log::warn!("workspace subscription died: {e}");
            Message::Workspaces(None)
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SlotView {
    coordinate: Option<i128>,
    name: String,
    protocol_id: Option<String>,
    id: Option<WorkspaceId>,
    active: bool,
    urgent: bool,
    hidden: bool,
    can_switch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GroupView {
    outputs: Vec<String>,
    slots: Vec<SlotView>,
}

fn views_of(snapshot: &WorkspaceSnapshot) -> Vec<GroupView> {
    snapshot
        .groups
        .iter()
        .map(|group| {
            let workspaces: Vec<_> = snapshot.workspaces_in(&group.id).collect();
            let gridded = workspaces.iter().any(|w| w.coordinates.len() > 1);
            GroupView {
                outputs: group
                    .outputs
                    .iter()
                    .filter_map(|output| output.name.clone())
                    .collect(),
                slots: workspaces
                    .into_iter()
                    .map(|workspace| SlotView {
                        coordinate: if gridded {
                            None
                        } else {
                            workspace.coordinates.first().map(|first| *first as i128)
                        },
                        name: workspace.name.clone(),
                        protocol_id: workspace.protocol_id.clone(),
                        id: Some(workspace.id.clone()),
                        active: workspace.state.contains(State::Active),
                        urgent: workspace.state.contains(State::Urgent),
                        hidden: workspace.state.contains(State::Hidden),
                        can_switch: workspace
                            .capabilities
                            .contains(WorkspaceCapabilities::Activate),
                    })
                    .collect(),
            }
        })
        .collect()
}

fn sorted(mut groups: Vec<GroupView>) -> Vec<GroupView> {
    groups.sort_by_key(|group| {
        let smallest = group.outputs.iter().min().cloned();
        (smallest.is_none(), smallest)
    });
    groups
}

fn token_for(protocol_id: &Option<String>, group: usize, slot: i128) -> String {
    match protocol_id {
        Some(id) => id.clone(),
        None => format!("g{group}:{slot}"),
    }
}

struct Placed<'a> {
    index: i128,
    origin: usize,
    slot: &'a SlotView,
}

fn placed_in<'a>(group: &'a GroupView, origin: usize) -> Vec<Placed<'a>> {
    group
        .slots
        .iter()
        .filter(|slot| !slot.hidden)
        .enumerate()
        .map(|(fallback, slot)| Placed {
            index: slot.coordinate.unwrap_or(fallback as i128),
            origin,
            slot,
        })
        .collect()
}

fn slot_for_token<'a>(groups: &'a [GroupView], token: &str) -> Option<&'a SlotView> {
    groups.iter().enumerate().find_map(|(position, group)| {
        placed_in(group, position)
            .into_iter()
            .find(|placed| token_for(&placed.slot.protocol_id, position, placed.index) == token)
            .map(|placed| placed.slot)
    })
}

pub fn switch(token: &str) -> Result<(), String> {
    let Some(snapshot) = snapshot() else {
        return Err("no workspace information available".to_string());
    };
    let groups = sorted(views_of(&snapshot));
    let Some(id) = slot_for_token(&groups, token).and_then(|slot| slot.id.clone()) else {
        return Err(format!("no such workspace \"{token}\""));
    };
    snapshot.activate(&id).map_err(|e| e.to_string())
}

pub fn values() -> Vec<(String, VarValue)> {
    vars_of(
        snapshot()
            .map(|snapshot| views_of(&snapshot))
            .unwrap_or_default(),
    )
}

fn scope_vars(prefix: &str, outputs: &str, placed: &[Placed]) -> Vec<(String, VarValue)> {
    let first = placed.iter().map(|p| p.index).min().unwrap_or(0);
    let mut last = placed.iter().map(|p| p.index).max().unwrap_or(-1);
    if last - first + 1 > MAX_SLOTS {
        static CLAMP_WARNING: std::sync::Once = std::sync::Once::new();
        CLAMP_WARNING.call_once(|| {
            log::warn!(
                "{prefix}: workspace coordinates span too many slots, clamping to {MAX_SLOTS}"
            );
        });
        last = first + MAX_SLOTS - 1;
    }

    let mut out = vec![
        (
            format!("{prefix}.outputs"),
            VarValue::Str(outputs.to_string()),
        ),
        (
            format!("{prefix}.count"),
            VarValue::Int(placed.len() as i128),
        ),
        (format!("{prefix}.first"), VarValue::Int(first)),
        (format!("{prefix}.last"), VarValue::Int(last)),
    ];
    for index in first..=last {
        let key = format!("{prefix}.slot.{index}");
        let found = placed.iter().find(|p| p.index == index);
        out.push((format!("{key}.exists"), VarValue::Bool(found.is_some())));
        out.push((
            format!("{key}.active"),
            VarValue::Bool(found.is_some_and(|p| p.slot.active)),
        ));
        out.push((
            format!("{key}.urgent"),
            VarValue::Bool(found.is_some_and(|p| p.slot.urgent)),
        ));
        out.push((
            format!("{key}.can_switch"),
            VarValue::Bool(found.is_some_and(|p| p.slot.can_switch)),
        ));
        out.push((
            format!("{key}.name"),
            VarValue::Str(found.map(|p| p.slot.name.clone()).unwrap_or_default()),
        ));
        out.push((
            format!("{key}.id"),
            VarValue::Str(
                found
                    .map(|p| token_for(&p.slot.protocol_id, p.origin, index))
                    .unwrap_or_default(),
            ),
        ));
    }
    out
}

fn vars_of(groups: Vec<GroupView>) -> Vec<(String, VarValue)> {
    let groups = sorted(groups);

    let mut out = vec![(
        "iwwc.ws.groups".to_string(),
        VarValue::Int(groups.len() as i128),
    )];

    if groups.is_empty() {
        out.extend(scope_vars("iwwc.ws.g0", "", &[]));
        out.extend(scope_vars("iwwc.ws.all", "", &[]));
        out.push(("iwwc.ws.all.collides".to_string(), VarValue::Bool(false)));
        return out;
    }

    let mut merged: Vec<Placed> = Vec::new();
    let mut collides = false;

    for (position, group) in groups.iter().enumerate() {
        let placed = placed_in(group, position);
        out.extend(scope_vars(
            &format!("iwwc.ws.g{position}"),
            &group.outputs.join(","),
            &placed,
        ));
        for entry in placed {
            if merged.iter().any(|seen| seen.index == entry.index) {
                collides = true;
                continue;
            }
            merged.push(entry);
        }
    }

    let all_outputs: Vec<String> = groups
        .iter()
        .flat_map(|group| group.outputs.iter().cloned())
        .collect();
    merged.sort_by_key(|entry| entry.index);
    out.extend(scope_vars("iwwc.ws.all", &all_outputs.join(","), &merged));
    out.push(("iwwc.ws.all.collides".to_string(), VarValue::Bool(collides)));
    out
}
