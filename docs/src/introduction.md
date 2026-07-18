# Introduction

**iwwc** (Iced Wayland Widget Center) is a widget daemon for Wayland
compositors. It draws bars, sidebars and popup widgets as layer-shell
surfaces using [iced](https://github.com/iced-rs/iced) lib.

Everything is driven by [KDL](https://kdl.dev/) configs.

Beside user-defined widgets, iwwc ships built-in subsystems:

- a **system tray** (StatusNotifierItem) with clickable icons that provide menus.
- a **notification daemon** implementing `org.freedesktop.Notifications`

## How docs are organized

- **Getting Started** - installing iwwc, running the daemon, and building your first widget.
- **Configuration Guide** - one concept per chapter, in reading order: config basics, windows,
elements, styling, variables, expressions, events, tray, notifications.
- **Reference** - property tables and CLI documentation for looking things up later.
