# Styling

Styling is split into small named nodes that elements reference by id, so one look can
be shared across many elements.

```kdl
border round20 radius=20
shadow soft color="00000080" blur_radius=4 {
  offset 0 2
}
style pill bg="282828" text="e3cd92" border=round20 shadow=soft
style pillhover bg="3c3836" text="e7d4a2" border=round20

button lang child=lang_txt style=pill style:hover=pillhover
```

## style

A `style` node bundles:

- `bg`: background color
- `text`: text color
- `border`: id of a `border` node
- `shadow`: id of a `shadow` node
- `snap`: bool, snaps the element to the pixel grid

Every property is optional. Styles apply to `container` (via `style=`), `button` (via
`style=`, `style:hover=`, `style:active=`, `style:disabled=`), and notification action
buttons (via `ok:style` / `no:style`, see [Notifications](./notifications.md#clicks-and-action-buttons)).

## border

- `color`
- `w`: line width
- `radius`: radius takes one value as a property (`radius=10`) or a block with 1 or 4 values (top-left,
top-right, bottom-right, bottom-left)

```kdl
border topround {
  radius 10 10 0 0
}
```

## shadow

- `color`: shadow color
- `blur_radius`: blur amount in pixels
- `offset`: block with exactly 2 values (x, y)

```kdl
shadow soft color="00000080" blur_radius=4 {
  offset 0 2
}
```

## font

Named font definitions, referenced from `text` elements and the `notification` block by id:

- `family`: font family name
- `weight`: one of `thin`, `extra-light`, `light`, `normal`, `medium`, `semibold`, `bold`, `extra-bold`, `black`
- `stretch`: one of `ultra-condensed`, `extra-condensed`, `condensed`, `semi-condensed`, `normal`, `semi-expanded`,
`expanded`, `extra-expanded`, `ultra-expanded`
- `style`: `normal`, `italic`, `oblique`

Values are case-insensitive and the hyphens are optional (`extralight`, `semi-bold`, …).

```kdl
font ff family="JetBrainsMonoNL Nerd Font" weight="bold" style="italic"
text lang_txt text="en" font=ff
```

## Colors

Anywhere a color is expected:

- `rrggbb` or `rrggbbaa` hex
- short 3/4-digit forms (`f80` = `ff8800`)
- a leading `#` is optional on all of them
- the keyword `transparent`

## Padding

`container`, `button`, `row`, `column`, and the tray take padding:

- one value as a property (`padding=5`): all four sides
- a block with 1, 2 (vertical, horizontal), or 4 (top, right, bottom, left) values

```kdl
button b child=t {
  padding 5 13
}
```
