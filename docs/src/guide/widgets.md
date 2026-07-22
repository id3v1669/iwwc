# Windows & Surfaces

A `widget` node declares a window: a Wayland layer-shell surface with a single root element.

```kdl
widget bar layer=top anchor="t | l | r" h=30 exclusive=#true child=barrow {
  margin 5 8 0 8
}
```

Widgets are opened and closed by name with `iwwc open`, `iwwc close`, and `iwwc toggle`.

## Anchors and size

`anchor` is a `|`-separated list of edges: `t`/`top`, `b`/`bottom`, `l`/`left`, `r`/`right`.
Unset, it defaults to top-left. `w` and `h` give the surface size in pixels.

Anchors and size interact:

- If `w` is not set, the surface is additionally anchored to **both** left and right -
it spans the full width. Same for no `h` means it spans the full height.
- Anchoring to only one of left/right *without* setting `w` is rejected at startup:
the surface would have no width. The same applies to top/bottom and `h`.
- Setting `w` *and* anchoring both left and right is a conflict - remove one. Same
for `h` with top and bottom.

## Layers and exclusive zones

`layer` places the surface at `background`, `bottom`, `top` (default), or `overlay`.

`exclusive=#true` reserves screen space so tiled windows don't overlap the widget. The reserved
amount is `h` for a bar anchored to top *or* bottom, and `w` for a sidebar anchored to left *or*
right, other anchor combinations reserve nothing.

`margin` offsets the surface from its anchored edges - one value as a property (`margin=12`,
all sides) or a block with 1, 2 (vertical, horizontal), or 4 (top, right, bottom, left) values.

## Output, keyboard, and click-through

- `output` picks the monitor: `active`, `last` (default), an output name like `"HDMI-A-1"`, or
  `"@other-widget"` to open on whatever monitor another widget is currently on.
- `keyboard=#true` grabs keyboard focus exclusively while the widget is open (for popup-style widgets).
- `transparent=#true` makes the surface ignore pointer events - clicks pass through to whatever is underneath.
