# Style Properties

## style

| Field | Type | Notes |
|---|---|---|
| `bg` | color | background |
| `text` | color | text/foreground |
| `border` | id of `border` | |
| `shadow` | id of `shadow` | |
| `snap` | bool | pixel snapping |

Used by `container` (`style`), `button` (`style`, `style:hover`,
`style:active`, `style:disabled`), and the `notification` node's
`ok:style*` / `no:style*` button styles.

## border

| Field | Type | Notes |
|---|---|---|
| `color` | color | |
| `w` | number | line width |
| `radius` | 1 or 4 numbers | 4 values: top-left, top-right, bottom-right, bottom-left |

## shadow

| Field | Type | Notes |
|---|---|---|
| `color` | color | |
| `blur_radius` | number | |
| `offset` | exactly 2 numbers | x, y |

## font

| Field | Values |
|---|---|
| `family` | font family name string |
| `weight` | `thin` `extra-light` `light` `normal` `medium` `semibold` `bold` `extra-bold` `black` |
| `stretch` | `ultra-condensed` `extra-condensed` `condensed` `semi-condensed` `normal` `semi-expanded` `expanded` `extra-expanded` `ultra-expanded` |
| `style` | `normal` `italic` `oblique` |

## Colors

`rrggbb`, `rrggbbaa`, `#rrggbb`, `#rrggbbaa`, or `transparent`.

## Lengths

Element `w`/`h`: a number (fixed px), `fill`, `shrink`, or `portion`
followed by a number for proportional sizing (`w portion 2`).