# Element Reference

Every element is declared at the top level as `<node> <id> key=value { … }` and composed
via `child=` (single child) or a `children` block (see [Elements](../guide/elements.md)).
Length, color, and style-id types are described in [Style Properties](./style-properties.md).

## widget

| Property | Type | Notes |
|---|---|---|
| `child` | id of element | root element |
| `w`/`h` | number | surface size in px, unset spans the full axis |
| `anchor` | edge list string | `t`/`top`, `b`/`bottom`, `l`/`left`, `r`/`right`, joined with `\|`, default top-left |
| `layer` | `top` `bottom` `background` `overlay` | default `top` |
| `exclusive` | bool | reserves screen space |
| `margin` | number, or block with 1, 2 or 4 values | offset from anchored edges |
| `output` | `active`, `last`, output name, `"@widget"` | monitor to open on, default `last` |
| `keyboard` | bool | exclusive keyboard focus while open |
| `transparent` | bool | pointer events pass through |

See [Windows & Surfaces](../guide/widgets.md) for how anchors, sizes, and exclusive zones interact.

## container

| Property | Type | Notes |
|---|---|---|
| `child` | id of element | required |
| `w` / `h` | length | |
| `padding` | number, or block with 1, 2 or 4 values | |
| `align_x` | `l` `c` `r` / `left` `center` `right` | child alignment |
| `align_y` | `t` `c` `b` / `top` `center` `bottom` | child alignment |
| `clip` | bool | |
| `style` | id of `style` | |

## row and column

| Property | Type | Notes |
|---|---|---|
| `children` block | ids | required; `children a b c` |
| `w` / `h` | length | |
| `padding` | number, or block with 1, 2 or 4 values | |
| `spacing` | number | gap between children |
| `align` | row: `t` `c` `b`; column: `l` `c` `r` | cross-axis alignment |
| `clip` | bool | |

## text

| Property | Type | Notes |
|---|---|---|
| `text` | string | content |
| `color` | color | |
| `font` | id of `font` | |
| `align_x` | `l` `c` `r` `j` / `left` `center` `right` `justified` | |
| `align_y` | `t` `c` `b` / `top` `center` `bottom` | |
| `w` / `h` | length | |

## button

| Property | Type | Notes |
|---|---|---|
| `child` | id of element | required |
| `action` | string | shell command, run on left click |
| `w` / `h` | length | |
| `padding` | number, or block with 1, 2 or 4 values | |
| `clip` | bool | |
| `style` `style:hover` `style:active` `style:disabled` | id of `style` | per-state styling |

## revealer

| Property | Type | Notes |
|---|---|---|
| `child` | id of element | required |
| `transition` | `none` `slideup` `slidedown` `slideleft` `slideright` | default `none` |
| `duration` | duration | default `300ms` |
| `active` | bool | shown when `#true`; default `#true` |

## event

| Property | Type | Notes |
|---|---|---|
| `type` | `onhover` `onhoverexit` `rightclick` `watchon` `watchoff` `timeout` | required, literal only |
| `action` | string | required; shell command |
| `child` | id of element | pointer types only, required for them |
| `var` | bool variable name | watch types only, required for them |
| `duration` | duration | `timeout` only, required for it |

Pointer events wrap their `child`.

Watch-type events are global and cannot be used as a child (see [Events & Actions](../guide/events-actions.md)).

## apptray

The system tray is used as a child under the fixed id `apptray` - it takes no declaration.
An optional top-level `apptray` node tunes it (see [System Tray](../guide/tray.md)):

| Property | Type | Notes |
|---|---|---|
| `icon_size` | number | default 22 |
| `spacing` | number | default 4 |
| `padding` | number, or block with 1, 2 or 4 values | |
| `bg` | color | strip background |
| `border` | id of `border` | |
| `vertical` | bool | stack icons vertically |
| `swap_buttons` | bool | swap left/right click |
| `menu_bg` / `menu_text` / `menu_disabled` | color | popup menu colors |

## Durations

A number with a unit suffix: `ms`, `s`, `m`, or `h` - `"500ms"`, `"30s"`, `"2m"`, `"1h"`.
