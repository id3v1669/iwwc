# System Tray

iwwc includes a StatusNotifierItem host: apps that expose a tray icon (network applet,
media players, etc) show up as clickable icons with their menus.

## Embedding the tray

The tray is an element with the fixed name `apptray`. Reference it like any other element:

```kdl
row rightgrp {
  spacing 6
  align c
  children music apptray sound battery
}
```

It works without any configuration. An optional top-level `apptray` node tunes it:

```kdl
apptray {
  icon_size 22
  spacing 4
  swap_buttons #true
  padding 0 4
}
```

- `icon_size` (default 22) and `spacing` (default 4) size the icon strip.
- `vertical #true` stacks icons vertically, for sidebars.
- `swap_buttons #true` swaps left and right click: left click then opens the item's menu
and right click triggers its primary action.
- `padding`, `bg`, `border` style the strip itself.

Icon left click activates the item, right click opens its menu (unless `swap_buttons` flips
them). Menus support submenus, checkmarks and icons.

## Styling the popup menu

The menu is configured separately from the strip, and there are two nodes to pick from.

`apptraymenu` is the short one: a font size and three colors. Everything else - icon size,
row spacing, paddings, corner radii - is derived from the font size, so the menu stays
proportional when you scale it:

```kdl
apptraymenu {
  font_size 18
  menu_bg "3c3836"
  button_fg "bdae93"
  button_bg "3c3836"
}
```

It takes no `font` - reach for `apptraymenu_advanced` when you need to pick the family.

`button_fg` and `button_bg` color the normal row; hover and active colors are derived
from them (lightened or darkened depending on how bright `button_bg` is), and disabled
rows dim the text.

`apptraymenu_advanced` is the long one: nothing is derived, every value is yours, and the
ones you leave out keep their defaults.

```kdl
style menu_surface { bg "3c3836"; border menu_edge }
style menu_row     { bg "3c3836"; text "bdae93" }
style menu_row_hot { bg "504945"; text "d65d0e" }

apptraymenu_advanced {
  font ff
  font_size 14
  icon_size 16
  row_spacing 3
  menu_container_padding 6
  menu_container_style menu_surface
  button_padding 5 8
  button_style menu_row
  button_style_hover menu_row_hot
  button_style_active menu_row_hot
  button_style_disabled menu_row
}
```

`button_style_active` colors both a pressed row and the parent row of an open submenu.
Menu width and row height are measured from the font, icon size and paddings, so the menu
never needs an explicit size.

Define both nodes and `apptraymenu` is ignored with a warning - `apptraymenu_advanced`
wins. Define neither and the menu uses its defaults.

## Icon themes

Tray icons that arrive as names (rather than pixmaps) are looked up in a freedesktop icon
theme. Pick one with the top-level node:

```kdl
icon_theme "Gruvbox-Plus-Dark"
```
