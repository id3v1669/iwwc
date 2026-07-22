# Style Properties

## style

| Property | Type | Notes |
|---|---|---|
| `bg` | color | background |
| `text` | color | text/foreground |
| `border` | id of `border` | |
| `shadow` | id of `shadow` | |
| `snap` | bool | pixel snapping |

Used by `container` (`style=`), `button` (`style=`, `style:hover=`,
`style:active=`, `style:disabled=`), and the `notification` node's
`ok:style*` / `no:style*` button styles.

## border

| Property | Type | Notes |
|---|---|---|
| `color` | color | |
| `w` | number | line width |
| `radius` | number, or block with 1 or 4 values | 4 values: top-left, top-right, bottom-right, bottom-left |

## shadow

| Property | Type | Notes |
|---|---|---|
| `color` | color | |
| `blur_radius` | number | |
| `offset` block | exactly 2 values | x, y |

## font

| Property | Values |
|---|---|
| `family` | font family name string |
| `weight` | `thin` `extra-light` `light` `normal` `medium` `semibold` `bold` `extra-bold` `black` |
| `stretch` | `ultra-condensed` `extra-condensed` `condensed` `semi-condensed` `normal` `semi-expanded` `expanded` `extra-expanded` `ultra-expanded` |
| `style` | `normal` `italic` `oblique` |

## Colors

`rrggbb`, `rrggbbaa`, `#rrggbb`, `#rrggbbaa`, or `transparent`.

## Lengths

Element `w`/`h`: a number (fixed px), `fill`, `shrink`, or a
proportional `portion` block.