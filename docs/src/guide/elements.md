# Elements

Elements are the building blocks inside a widget. Each is declared at the top level with
an id and composed via a `child` field (single child) or a `children` field (multiple).

```kdl
text clock_txt "${datetime}"
button clock {
  child clock_txt
  action "gnome-calendar"
  style pill
  padding 5 16
}
row centergrp {
  align c
  children clock
}
```

An element may be used as a child in several places; each use renders its own copy.

## text

Renders a string. The content is the node's second argument (or a `text` field). Optional: `color`,
`font` (id of a `font` node), `align_x` (`l`/`c`/`r`/`j`), `align_y` (`t`/`c`/`b`),
`w`, `h`.

## button

Clickable wrapper around one `child`. On left click it runs `action` as a shell command.
Styling is per state: `style`, `style:hover`, `style:active`, `style:disabled`, each
naming a `style` node. Also takes `w`, `h`, `padding`, `clip`.

## row and column

Lay out `children` horizontally (`row`) or vertically (`column`), with `spacing` between
them. `align` positions children on the cross axis: `t`/`c`/`b` for rows, `l`/`c`/`r`
for columns. Also `w`, `h`, `padding`, `clip`.

## repeat

Generates children instead of listing them: `child` is instantiated once per index from
`from` to `to`, and the instances are spliced into the row or column `children` list
that references the repeat. The container decides direction, spacing and alignment,
and generated instances can sit between static children (`children logo wsrow clock`).
The `as` alias gives each instance its slice of the `over` variable namespace - the
canonical use is a workspace bar over `iwwc.ws.*`:

```kdl
row bar { children wsrow; spacing 6 }
repeat wsrow {
  over "iwwc.ws.all.slot"
  from "${iwwc.ws.all.first}"
  to   "${iwwc.ws.all.last}"
  as w
  child wsslot
}
button wsslot {
  action "${w.exists && w.can_switch ? '$IWWC ws switch ' + w.id : ''}"
  child wslabel
  style "${w.exists ? (w.active ? 'wsactive' : 'wsplain') : 'wsdot'}"
}
text wslabel "${w.exists ? w.name : ''}"
```

## container

Wraps a single `child` to give it size, padding, alignment (`align_x`, `align_y`) and a
`style`. The classic pattern for a three-part bar is three `w fill` containers aligned
left, center, and right inside one row:

```kdl
container leftbox {
  w fill
  align_x l
  child x
}
container centerbox {
  w fill
  align_x c
  child y
}
container rightbox {
  w fill
  align_x r
  child z
}
row barrow {
  w fill
  align c
  children leftbox centerbox rightbox
}
```

## revealer

Shows or hides its `child` with an animation. `transition` is one of `none`, `slideup`,
`slidedown`, `slideleft`, `slideright`.

`duration` (default 300 ms) is a duration
string like `"200ms"` or `"1s"`.

`active` (default `#true`) is the current state - typically an expression on a bool
variable, flipped with `iwwc update <var> toggle`:

```kdl
var show_extras=#true
revealer extras {
  transition slideleft
  duration "250ms"
  active "${show_extras}"
  child extras_row
}
```

## event

Invisible wrapper that reacts to pointer events - see [Events & Actions](events-actions.md).

## apptray

The built-in system tray, referenced by the fixed name `apptray` in a `children` list or
`child apptray` - see [System Tray](tray.md).

## Sizes

`w` and `h` on elements accept a number (fixed pixels), `fill`, `shrink`, or `portion` followed
by a number for proportional sizing:

```kdl
container half {
  child stuff
  w portion 1
}
```
