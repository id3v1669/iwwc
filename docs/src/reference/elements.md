# Element Reference

Every element is declared at the top level as `<node> <id> { … }`, with one child node per
field, and composed via a `child` field (single child) or a `children` field (see
[Elements](../guide/elements.md)).
Length, color, and style-id types are described in [Style Properties](./style-properties.md).

## widget

| Field | Type | Notes |
|---|---|---|
| `child` | id of element | root element |
| `w`/`h` | number | surface size in px, unset spans the full axis |
| `anchor` | edge list string | `t`/`top`, `b`/`bottom`, `l`/`left`, `r`/`right`, joined with `\|`, default top-left |
| `layer` | `top` `bottom` `background` `overlay` | default `top` |
| `exclusive` | bool | reserves screen space |
| `margin` | 1, 2 or 4 numbers | offset from anchored edges |
| `output` | `active`, `last`, output name, `"@widget"` | monitor to open on, default `last` |
| `keyboard` | bool | exclusive keyboard focus while open |
| `transparent` | bool | pointer events pass through |

See [Windows & Surfaces](../guide/widgets.md) for how anchors, sizes, and exclusive zones interact.

## container

| Field | Type | Notes |
|---|---|---|
| `child` | id of element | required |
| `w` / `h` | length | |
| `padding` | 1, 2 or 4 numbers | |
| `align_x` | `l` `c` `r` / `left` `center` `right` | child alignment |
| `align_y` | `t` `c` `b` / `top` `center` `bottom` | child alignment |
| `clip` | bool | |
| `style` | id of `style` | |

## row and column

| Field | Type | Notes |
|---|---|---|
| `children` | ids | required; `children a b c` |
| `w` / `h` | length | |
| `padding` | 1, 2 or 4 numbers | |
| `spacing` | number | gap between children |
| `align` | row: `t` `c` `b`; column: `l` `c` `r` | cross-axis alignment |
| `clip` | bool | |

## text

| Field | Type | Notes |
|---|---|---|
| `text` | string | content; usually given as the node's second argument |
| `color` | color | |
| `font` | id of `font` | |
| `align_x` | `l` `c` `r` `j` / `left` `center` `right` `justified` | |
| `align_y` | `t` `c` `b` / `top` `center` `bottom` | |
| `w` / `h` | length | |

## button

| Field | Type | Notes |
|---|---|---|
| `child` | id of element | required |
| `action` | string | shell command, run on left click |
| `w` / `h` | length | |
| `padding` | 1, 2 or 4 numbers | |
| `clip` | bool | |
| `style` `style:hover` `style:active` `style:disabled` | id of `style` | per-state styling |

## revealer

| Field | Type | Notes |
|---|---|---|
| `child` | id of element | required |
| `transition` | `none` `slideup` `slidedown` `slideleft` `slideright` | default `none` |
| `duration` | duration | default `300ms` |
| `active` | bool | shown when `#true`; default `#true` |

## event

| Field | Type | Notes |
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

| Field | Type | Notes |
|---|---|---|
| `icon_size` | number | default 22 |
| `spacing` | number | default 4 |
| `padding` | 1, 2 or 4 numbers | |
| `bg` | color | strip background |
| `border` | id of `border` | |
| `vertical` | bool | stack icons vertically |
| `swap_buttons` | bool | swap left/right click |

## apptraymenu

Styles the tray popup menu from a font size and three colors; every remaining value is
derived from the font size. Use `apptraymenu_advanced` to pick the font.

| Field | Type | Notes |
|---|---|---|
| `font_size` | number | default 14, drives every derived value |
| `menu_bg` | color | menu surface background |
| `button_fg` | color | menu row text |
| `button_bg` | color | menu row background |

`button_fg` / `button_bg` color the normal row; hover and active colors are derived from
them (lightened or darkened depending on how bright `button_bg` is), and disabled rows
dim the text.

## apptraymenu_advanced

Spells out every menu value by hand. Unset fields keep their defaults; they are *not*
derived from `font_size`. If both this and `apptraymenu` are present, `apptraymenu` is
ignored with a warning.

| Field | Type | Notes |
|---|---|---|
| `font` | id of `font` | |
| `font_size` | number | default 14 |
| `icon_size` | number | default 14 |
| `row_spacing` | number | default 3, gap between icon, label and submenu arrow |
| `menu_container_padding` | 1, 2 or 4 numbers | menu inner padding, default 6 |
| `menu_container_style` | id of `style` | menu surface (bg, border, shadow) |
| `button_padding` | 1, 2 or 4 numbers | menu row padding, default 5 8 |
| `button_style` | id of `style` | menu row, normal |
| `button_style_hover` | id of `style` | |
| `button_style_active` | id of `style` | pressed, and the parent of an open submenu |
| `button_style_disabled` | id of `style` | |

## Durations

A number with a unit suffix: `ms`, `s`, `m`, or `h` - `"500ms"`, `"30s"`, `"2m"`, `"1h"`.
