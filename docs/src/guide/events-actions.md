# Events & Actions

## Actions

An *action* is a shell command line, run with `sh -c` whenever its trigger fires. Actions
inherit the environment plus `$IWWC`, the path of the running iwwc binary - handy for
driving iwwc from itself:

```kdl
button power child=power_txt action="$IWWC toggle powermenu"
```

The simplest trigger is a `button`'s left click. Everything else goes through `event` declarations.

## Pointer events

`event` with type `onhover`, `onhoverexit`, or `rightclick` wraps a `child` element and runs
`action` when the pointer event happens over it:

```kdl
event volume_scroll type="rightclick" child=sound action="pavucontrol"
row rightgrp {
  children volume_scroll mic battery
}
```

Pointer events require `child` and must not have `var`.

## Watch events

Types `watchon`, `watchoff`, and `timeout` watch a **bool variable** instead of wrapping an element.
They require `var` and `action`, and must not have `child`:

```kdl
var actvar=#false
event actvar_on  type="watchon"  var=actvar action="notify-send "watchon""
event actvar_off type="watchoff" var=actvar action="notify-send "watchon""
event actvar_end type="timeout"  var=actvar duration="30m" action="$IWWC update actvar toggle"
```

- `watchon` fires when the variable flips to `#true`
- `watchoff` fires when it flips to `#false`
- `timeout` starts a timer of `duration` when the variable flips to `#true` and runs the action
when it expires while flipping back to `#false` first cancels the timer. `duration` is required
for `timeout` and invalid on the other types.

Variables flip via `iwwc update <var> toggle` (or an explicit `#true`/`#false` value), so a single
bool can drive a revealer's `active`, run enter/exit actions, and auto-reset on a timeout.
