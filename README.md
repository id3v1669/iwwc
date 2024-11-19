# rs-nc
A simple iced based notification daemon for linux

WIP

TODO:
1) rework folder structure and filenames
2) add icons(get path with gtk)
3) add config for styles(json?)
5) create notification center
6) adjust max ammount of notifications logic to haldle limit per screen
7) variable to choose screen for notifications
8) adjust Notification struct to support hints.
9) animations?
10) kill the app on unsuccessfull 
11) ...


# Notes

* nvidia moment - use WGPU_BACKEND=opengl to avoid crashes with vulkan. Transparent bg won't work, so behind rounded corners black bg ((
