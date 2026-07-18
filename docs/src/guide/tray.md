# System Tray

iwwc includes a StatusNotifierItem host: apps that expose a tray icon (network applet,
media players, etc) show up as clickable icons with their menus.

## Embedding the tray

The tray is an element with the fixed name `apptray`. Reference it like any other element:

```kdl
row rightgrp spacing=6 align=c {
  children music apptray sound battery
}
```

It works without any configuration. An optional top-level `apptray` node tunes it:

```kdl
apptray icon_size=22 spacing=4 swap_buttons=#true {
  padding 0 4
}
```

- `icon_size` (default 22) and `spacing` (default 4) size the icon strip.
- `vertical=#true` stacks icons vertically, for sidebars.
- `swap_buttons=#true` swaps left and right click: left click then opens the item's menu
and right click triggers its primary action.
- `padding`, `bg`, `border` style the strip itself.
- `menu_bg`, `menu_text`, `menu_disabled` recolor the popup menu (background, item text,
disabled item text). Unset, the menu uses built-in colors.

Icon left click activates the item, right click opens its menu (unless `swap_buttons` flips
them). Menus support submenus, checkmarks and icons.

## Icon themes

Tray icons that arrive as names (rather than pixmaps) are looked up in a freedesktop icon
theme. Pick one with the top-level node:

```kdl
icon_theme "Gruvbox-Plus-Dark"
```
