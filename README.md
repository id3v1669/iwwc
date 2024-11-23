# rs-nc
A simple iced based notification daemon for linux

WIP

TODO:
1) rework folder structure and filenames
2) optimise way of retrieving icons
3) create notification center
4) adjust max ammount of notifications logic to handle limit from screen size
5) adjust Notification struct to support hints.
6) create a default logo for the app(for testing borrowed random linux svg)
7) nix module
8) fix handling notifications on multiple screens(either separate margins for each screen or pick single screen for notifications)
9) ...


# Notes

* Nvidia moment - use flag -n to avoid crashing on vulkan (corners will not be round)
* Ram usage depends on hardware. For example intel+nvidia 450-500mb, amd+amd 150-200mb. As far as I know cannot be fixed as it is something with iced itself and wgpu backend.