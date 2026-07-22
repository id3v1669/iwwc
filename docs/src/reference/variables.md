# Built-in Variables

`iwwc.*` namespace is available in any `${…}` expression without declaration.
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

## Refresh

- `iwwc.ram.*` and `iwwc.cpu.*` are polled every second - only while the config
references the namespace.
- `iwwc.activesong` is event-driven, updated as the song changes. Requires `playerctld`
running.

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
