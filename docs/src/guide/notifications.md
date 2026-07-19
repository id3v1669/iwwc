# Notifications

The daemon implements `org.freedesktop.Notifications` on D-Bus (spec 1.2).

Notifications work with zero configuration. The optional top-level `notification` node
overrides the defaults:

```kdl
notification width=400 \
      primary_text="#e7d4a2" \
      secondary_text="#e3cd92" \
      bg="#3c3836" \
      border="notif_border" \
      font=notif_font \
      anchor="t | r" \
      gap=8 \
      max=5 \
      timeout_low="5s" \
      timeout_normal="5s" \
      timeout_critical="0s" \
      urgency_low="#888786" \
      urgency_normal="#2b6d19" \
      urgency_critical="#8c1d10" \
      respect_notification_icon=#true \
      margin=12

border notif_border radius=10 color="#d65d0e" w=2
font notif_font family="JetBrains Mono"
```

## Urgency

The `urgency` hint (0 = low, 1 = normal, 2 = critical; missing = normal) controls two things:

- **Timeout**: `timeout_low` / `timeout_normal` / `timeout_critical` take duration strings
(`500ms`, `5s`, `2m`, `1h`) and apply when the sender does not request its own expiry.
`0s` means never expire; the default for critical is never, per spec.
- **Indicator color**: the corner dot uses `urgency_low` / `urgency_normal` / `urgency_critical`.

## Do not disturb

`dnd` is daemon state, always available at runtime - no declaration needed:

```sh
iwwc update dnd 1
iwwc get dnd
```

`0` shows all notifications, `1` shows only critical, `2` suppresses everything.
Suppressed notifications are dropped, not queued(TODO - will be queued ). Senders still get a valid reply.

The optional `dnd` attribute on the `notification`. `iwwc reload` does **not** touch the runtime
value - only a daemon restart re-reads the config default.

## Properties

Placement:

- `width` (default 400) - popup width in pixels. Height is computed from content.
- `anchor` (default `t | r`) - screen corner, same flag syntax as widgets.
- `margin` (default 12) - distance from the anchored screen edges. One or four numbers.
- `gap` (default 8) - spacing between stacked popups.
- `max` (default 5) - popups shown at once. Oldest is evicted when a new one arrives.
- `layer` (default `overlay`) - layershell layer: `top`, `bottom`, `background`, `overlay`.
- `output` (default `active`) - monitor to show popups on: `active`, `last`, or an output name like `HDMI-A-1`.

Appearance:

- `bg` (default `#3c3836`) - popup background.
- `primary_text` (default `#e7d4a2`) - summary color.
- `secondary_text` (default `#e3cd92`) - body color.
- `border` - reference to a `border` node (default: 1.5px `#d65d0e`, radius 10).
- `font` - reference to a `font` node (default: system font).
- `urgency_low` / `urgency_normal` / `urgency_critical` (defaults `#bdae93` / `#98971a` / `#cc241d`) - urgency dot colors.

Behavior:

- `timeout_low` / `timeout_normal` / `timeout_critical` (defaults `10s` / `5s` / `0s`) - see [Urgency](#urgency).
- `dnd` (default 0) - do-not-disturb level the daemon starts with, see above.
- `freeze_on_hover` (default `#true`) - hovering a popup pauses its expiry; the timer restarts when the cursor leaves.
- `respect_notification_icon` (default `#true`) - use the icon the notification supplies, with `#false` every popup
shows the fallback icon `default.svg`.
- `ok:style` / `no:style` and variants - action button styles, see [Clicks and action buttons](#clicks-and-action-buttons).

## Clicks and action buttons

- **Left click** on a popup invokes the notification's *default* action (if present) and closes it.
- **Right click** dismiss popup.
- Any other actions the app provided are rendered as **buttons**. The first button is styled by
`ok:style` (with `ok:style:hover`, `ok:style:active`, `ok:style:disabled`), all further buttons by
`no:style` and its variants - each referencing `style` nodes:

```kdl
style okbtn bg="3c3836" text="b8bb26"
style nobtn bg="3c3836" text="fb4934"
notification ok:style="okbtn" no:style="nobtn"
```
