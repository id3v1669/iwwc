# rs-nc
A simple iced based notification daemon for linux

WIP

TODO:
1) rework folder structure and filenames
2) optimise way of retrieving icons
3) add config file(json?)
4) create notification center
5) adjust max ammount of notifications logic to handle limit from screen size
6) adjust Notification struct to support hints.
7) animations?
8) create a default logo for the app(for testing borrowed random linux svg)
9) nix module
10) figure out why intel+nvidia/amd+amd 450/170 ram usage
11) fix handling notifications on multiple screens(either separate margines for each screen or pick single screen for notifications)
12) optimise memory usage for iced itself?
13) ...


# Notes

* nvidia moment - use flag -n to avoid crashing on vulkan (corners will not be round)
