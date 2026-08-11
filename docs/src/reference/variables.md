# Built-in Variables

`iwwc.*` namespace is available in any `${...}` expression without declaration.
`iwwc get iwwc` lists the namespaces, `iwwc get iwwc.cpu` the entries under one.
All vars are read-only with exception for `iwwc.activesong`.

| Variable | Type | Notes |
|---|---|---|
| `iwwc.ram.total` | int | total ram, bytes |
| `iwwc.ram.used` | int | used ram, bytes (total minus available) |
| `iwwc.cpu.<n>.usage` | float | usage percent of core `n` (0-based) |
| `iwwc.cpu.<n>.frequency` | int | frequency of core `n`, MHz |
| `iwwc.cpu.avg.usage` | float | usage percent across all cores |
| `iwwc.activesong` | string | title of the currently playing song(MPRIS) |

Usage percentages are rounded to 2 decimals.

## `iwwc.ws.*` - workspaces

Populated from the compositor's `ext-workspace-v1` protocol. Groups are sorted by their
name. Hidden workspaces are filtered out. Slots are indexed by workspace coordinate,
gaps between coordinates are emitted with `exists #false`.

| Variable | Type | Notes |
|---|---|---|
| `iwwc.ws.groups` | int | number of workspace groups |
| `iwwc.ws.g<N>.outputs` | string | the group's output names, comma-joined |
| `iwwc.ws.g<N>.count` | int | visible workspaces in the group |
| `iwwc.ws.g<N>.first` / `.last` | int | slot index range; `last < first` when empty |
| `iwwc.ws.g<N>.slot.<M>.exists` | bool | `#false` for a gap slot |
| `iwwc.ws.g<N>.slot.<M>.active` | bool | |
| `iwwc.ws.g<N>.slot.<M>.urgent` | bool | |
| `iwwc.ws.g<N>.slot.<M>.can_switch` | bool | the compositor advertises switching to it |
| `iwwc.ws.g<N>.slot.<M>.name` | string | not unique across groups; empty for a gap |
| `iwwc.ws.g<N>.slot.<M>.id` | string | opaque token for `iwwc ws switch`; empty for a gap |

`iwwc.ws.all.*` carries the same field set with every group merged into one slot range,
plus `iwwc.ws.all.collides` (bool): when two groups place a workspace at the same slot
index, the first sorted group wins that slot in `all`, the loser stays visible in its own
`g<N>` scope, and `collides` is `#true`. A config written against `all` works unchanged
whether the compositor uses one group per output or a single global group.

`g0` and `all` are always emitted - `count 0`, `last < first` - even before the first
snapshot or on a compositor without the protocol, so configs resolve unconditionally.

## Refresh

- `iwwc.ram.*` and `iwwc.cpu.*` are polled every second - only while the config
references the namespace.
- `iwwc.activesong` is event-driven, updated as the song changes. Requires `playerctld`
running.
- `iwwc.ws.*` is event-driven, updated as the compositor announces workspace changes.
Requires a compositor implementing `ext-workspace-v1`.

## Fallback

`iwwc.activesong` is the only built-in that accepts a `var` declaration: while it is empty,
the declared value substitutes.

```kdl
var iwwc.activesong="nothing playing"
```

Declarations for other `iwwc.*` names are ignored - the built-in value always wins.

## `dnd`

Not a variable: `dnd` is reserved daemon state for the do-not-disturb level, readable and
settable at runtime via `iwwc get dnd` / `iwwc update dnd <0|1|2>` - see
[Notifications](../guide/notifications.md#do-not-disturb).
