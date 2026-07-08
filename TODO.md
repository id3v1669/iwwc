# TODO list for repo

Before Beta:

- [x] fix overlap on exclusive zones for vertical widget. Doesn't recalc to shrink on foreign exclusive widget entry. (I didn't fix it, just works now, lib fix?)
- [ ] fix issue with conflicting anchors and 0 h or w should default to max availible(local patch, Furute fix lib)
- [x] expose absolute mosue position as it's needed for correct apptray work (exwlshelleventloop patch): NOT VALID, IMPLEMENTED VIA xdg_popup
- [ ] animation support
- [ ] iwwc.cpu.(load/temp/per-core)
- [ ] iwwc.gpu.(load/temp/vram)(amd)
- [x] fix styles for apptray
- [x] Add proper font validation, conversion to static and tests
- [x] logic for notifications that prevents timeout on hover
- [x] fix tests under clippy
- [x] Tray menu direction is an ugly workaround that doesn't work well, replicate xdg_popup for layershell like one the eww uses.(exwlshelleventloop patch)
- [x] figure out how to properly calculate menu and submenu width and rm temp statics (calc based on text and font or make proper eval via iced, test what is more efficient and stable).
- [x] Rm bunch of leftover structures in favor of orig iced stucts (old code from iced fork and when layershell lib wasn't used.) #oldmigration

Beta:

- [ ] re-review nix packaging
- [ ] create nix module
- [ ] documentation
- [ ] rework examples

Future:

- [ ] re-review all tests
- [ ] better handling of attempt to create layer with 0 h or w (exwlshelleventloop patch)
- [ ] fix issue with conflicting anchors and 0 h or w should default to max availible (exwlshelleventloop patch)
- [ ] predefined widget calendar
- [ ] iwwc.gpu.(load/temp/vram)(nvidia/intel)
- [ ] iwwc.battery??
- [ ] notification storage
- [ ] notification centre to view history
- [ ] option to pause notifications: ipc and gui button trigger?
- [ ] logic for notifications to reply to message notifications right from notification
- [ ] #test1 since style is kinda default for buttons, handle on cfg side?
- [ ] get rid of functions that are used only once now
- [ ] re-review math for apptray, it works, but ugly and has duplicates. I just wanted to make it work
- [ ] tray separator make option to make it visible

