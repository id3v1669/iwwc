# CLI Commands & IPC

`iwwc daemon` runs the widget center itself and notification daemon if not disabled.

Every other subcommand is a client that connects to daemon over a Unix socket, sends one
command, prints the reply, and exits.

```sh
iwwc daemon &
iwwc open bar
iwwc update myintvar 80
```

## Global flags

- `-d`/`--debug`: enable debug logging. Valid on any subcommand.
- `--check [CONFIG]`: validate a config file and exit without touching the daemon. Without
path it checks the default config (see [Config File Basics](../guide/config-basics.md)).
Prints `<path>: ok` on success. On every fault problem is printed and the exit code is 1.
Warnings are treated as errors.

## Subcommands

### `iwwc daemon`

Starts the daemon. `-c <path>`/`--config <path>` to use custom config file. Errors in the config
abort startup, warnings are printed and startup continues. If a daemon is already listening
on the socket, a second `iwwc daemon` refuses to start. A stale socket is removed automatically.

### `iwwc update <name> <value>`

Sets a [variable](../guide/variables.md). Booleans accept `toggle` beside explicit
`#true`/`#false`. Unresolved values are rejected and nothing changes. The reserved name `dnd` sets
the daemon's do-not-disturb level (`0`, `1` or `2`, see [Notifications](../guide/notifications.md#do-not-disturb)).

### `iwwc get <name>`

Prints a variable's current value to stdout. `iwwc get dnd` reads the do-not-disturb level.
For the built-in namespace, `iwwc get iwwc` lists the available namespaces and
`iwwc get iwwc.cpu` the entries under one. An unknown name is an error.

### `iwwc open <window>…` / `iwwc close <window>…` / `iwwc toggle <window>…`

Open, close, or toggle [widgets](../guide/widgets.md) by name. Each command takes one or more
names, handled in order. On the first failure the command exits and the remaining names are
skipped. Opening an already-open widget succeeds without doing anything. Naming a widget the
config doesn't declare is a `no such widget` error, and closing a widget that isn't open is a
`window is not open` error.

### `iwwc reload`

Re-reads the daemon's config file. Open widgets are re-created and all variables reset to
their declared values. The runtime `dnd` level is kept. If the new file has errors, they're
printed and the daemon keeps the previous config. Warnings are printed and the reload goes
through.

## Exit status

- `0` - success.
- `1` - the daemon is not running, the daemon rejected the command, or `--check` found problems.
- `2` - no subcommand given (help is printed).

Errors and warnings go to stderr; only `iwwc get` output goes to stdout.
