# TODO list for repo

Before Beta:

- [x] fix overlap on exclusive zones for vertical widget. Doesn't recalc to shrink on foreign exclusive widget entry. (I didn't fix it, just works now, lib fix?)
- [x] fix issue with conflicting anchors and 0 h or w should default to max availible(local patch, Furute fix lib)
- [x] expose absolute mosue position as it's needed for correct apptray work (exwlshelleventloop patch): NOT VALID, IMPLEMENTED VIA xdg_popup
- [x] animation support
- [x] iwwc.cpu.(load/per-core)
- [x] fix styles for apptray
- [x] Add proper font validation, conversion to static and tests
- [x] logic for notifications that prevents timeout on hover
- [x] fix tests under clippy
- [x] Tray menu direction is an ugly workaround that doesn't work well, replicate xdg_popup for layershell like one the eww uses.(exwlshelleventloop patch)
- [x] figure out how to properly calculate menu and submenu width and rm temp statics (calc based on text and font or make proper eval via iced, test what is more efficient and stable).
- [x] Rm bunch of leftover structures in favor of orig iced stucts (old code from iced fork and when layershell lib wasn't used.) #oldmigration
- [x] add action support to have actions on hover enter, exit, rightclick
- [ ] update defaults
- [ ] fix menu text pos for tray menu to depend on rounded corners

Beta:

- [ ] re-review nix packaging
- [ ] create nix module
- [ ] documentation
- [ ] rework examples

Future:

- [ ] option for widgets to inherit output from another widet(useful for cases when first widget spawns via active)
- [ ] iwwc.gpu.(load/temp/vram)(amd)
- [ ] iwwc.temps.(*)
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
- [ ] animation types for revealer??
- [ ] more tests for revealer when/if decide on animation type handling
- [ ] animation guards for oversize?
- [ ] duration int fallback to ms or keeps strings to parse?
- [ ] animation options other than EaseInOut #rmstatic1
- [ ] add min,max custom logic for dimentions of objects
- [ ] learn about fluid and enclose for Length
- [ ] since vars for iwwc.cpu are dynamic how to test them? call via cpu.0? research values
- [ ] change default ration? #ratio
- [ ] double declaration of action doesn't trigger any warning on reload
- [x] update var for bool filip option
- [ ] add margins - create outer container and set paddings as margins for inner object
