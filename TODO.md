# TODO list for repo

Before Beta:

- [ ] fix overlap on exclusive zones for vertical widget. Doesn't recalc to shrink on foreign exclusive widget entry
- [ ] fix issue with conflicting anchors and 0 h or w should default to max availible(local patch, Furute fix lib)
- [ ] expose absolute mosue position as it's needed for correct apptray work (exwlshelleventloop patch)
- [ ] animation support
- [ ] iwwc.cpu.(load/temp/per-core)
- [ ] iwwc.gpu.(load/temp/vram)(amd)
- [ ] fix styles for apptray
- [ ] Add proper font validation and tests
- [ ] logic for notifications that prevents timeout on hover
- [ ] fix tests under clippy
- [ ] Tray menu direction is an ugly workaround that doesn't work well, replicate xdg_popup for layershell like one the eww uses.(exwlshelleventloop patch)
- [ ] figure out how to properly calculate menu and submenu width and rm temp statics (calc based on text and font or make proper eval via iced, test what is more efficient and stable).

Beta:

- [ ] re-review nix packaging
- [ ] create nix module
- [ ] documentation
- [ ] rework examples

Future:

- [ ] better handling of attempt to create layer with 0 h or w (exwlshelleventloop patch)
- [ ] fix issue with conflicting anchors and 0 h or w should default to max availible (exwlshelleventloop patch)
- [ ] predefined widget calendar
- [ ] iwwc.gpu.(load/temp/vram)(nvidia/intel)
- [ ] iwwc.battery??
- [ ] notification storage
- [ ] notification centre to view history
- [ ] option to pause notifications: ipc and gui button trigger?
- [ ] logic for notifications to reply to message notifications right from notification

