# Variables & Polling

A variable is a named value any property can reference.

Values change three ways:

- over IPC via `iwwc update`
- `pull` runs a shell command on an interval
- the built-in `iwwc.*` namespace

## Declaring variables

```kdl
var myintvar=40
var myfloatvar=1.5
var myboolvar=#false
var myunquotedstringvar=Proto
var myquotedstringvar="0xProto"
```

The type comes from the literal: integer, float, `#true`/`#false` bool, quoted or unquoted string.
Several pairs may share one node (`var a=1 b=2`). Declaring the same name twice is a warning, the first
declaration wins.

## Using variables: `${…}`

Reference a variable inside any quoted property value. Text around and between `${…}` blocks passes through:

```kdl
widget bar h="${myintvar}" child=t1
text t1 text="${round(iwwc.ram.used / 1073741824).1}G used, height ${myintvar / 2 + 10}"
```

Inside `${…}` full math is available: `+ - * / % ^`, comparisons (`< > <= >= == !=`), and the functions
`min(a, b)`, `max(a, b)`, and `round(x)` - the suffix `round(x).N` keeps `N` decimals. Referencing an
unknown variable is a config error at load time.

## Runtime updates

```sh
iwwc update myintvar 80
iwwc update myboolvar toggle
iwwc get myintvar
```

Every update re-resolves the config and re-renders open widgets. A value the config can't resolve with
is rejected and nothing changes.

`iwwc reload` resets all variables to their declared values. The name `dnd` is reserved - it is daemon
state, not a variable (see [Notifications](./notifications.md#do-not-disturb)).

## Pull variables

A `pull` declares a string variable and refreshes it by running a command:

```kdl
pull battery="cat /sys/class/power_supply/BAT0/capacity" i="30s" default="0%"
```

Every interval (`i=` or `interval=`, a duration string like `500ms`, `30s`, `2m`, `1h`) the command runs
via `sh -c`. Trimmed stdout becomes the value. If the command fails, the value falls back to `default`,
which is also the value shown before the first run.

## Built-in system variables

TODO will be rewritten after more added and functionality established.

- `iwwc.ram.total` / `iwwc.ram.used` - ram bytes.
- `iwwc.cpu.<n>.usage` - per-core usage percent
- `iwwc.cpu.<n>.frequency` - freq in MHz.
- `iwwc.cpu.avg.usage` - avg usage percent across all cores.
- `iwwc.activesong` - title of the currently playing MPRIS song, updated as it changes (requires `playerctld`
running). Empty when nothing plays. Declare `var iwwc.activesong="…"` to substitute a fallback.

`iwwc get iwwc` lists the available namespaces, `iwwc get iwwc.cpu` the entries under one.
