# Config File Basics

iwwc defaults to `config.kdl` in following order:

<div style="text-align:center; line-height:1.8;">
<code>$XDG_CONFIG_HOME/iwwc/config.kdl</code><br>
↓<br>
<code>$HOME/.config/iwwc/config.kdl</code><br>
↓<br>
<code>/home/$USER/.config/iwwc/config.kdl</code>
</div>

In config each top-level node declares one thing: a variable, an element, a style, or a settings block.

```kdl
// comment
var ff="0xProto"

text greeting_txt "Hello World" {
  font "${ff}"
}

button greeting {
  child greeting_txt
  action "notify-send hi"
  padding 5 13
}
```

The pieces of a declaration:

- **Node name** (`var`, `text`, `button`, `widget`, …) says what is being declared.
Unknown node names are an error.
- **First bare argument** is the declaration's *id* that will be used to call that element.
- **A block `{ … }`** holds the fields, one child node per field: `child greeting_txt`,
`action "notify-send hi"`, `padding 5 13`, `margin 5 8 0 8`, `radius 10 10 0 0`, `offset 2 2`,
and `children a b c`.

## Fields are child nodes

An element node carries only its id in the header. Everything else is a child node inside the
block: the node name is the field name, its arguments are the value.

```kdl
button greeting {
  child greeting_txt
  action "notify-send hi"
  w 120
  style pill
  style:hover pillhover
  padding 5 13
}
```

Fields that take several values simply take several arguments - `padding 5 13`,
`margin 5 8 0 8`, `radius 10 10 0 0`, `offset 2 2`, `children a b c`, `w portion 2`. A block
spans as many lines as it needs, so nothing has to be continued with a trailing `\`.

`text` is the one element that also accepts its content as an optional second argument, which
keeps short labels on one line:

```kdl
text greeting_txt "Hello World"
text clock_txt "${datetime}" {
  font ff
}
```

Booleans are written `#true` and `#false` (KDL 2 syntax) - bare `true` and `false` are reserved
words and a syntax error. Strings are quoted; an unquoted value is fine as long as it is a valid
KDL identifier, so a color that starts with a digit has to be quoted (`bg "3c3836"`).

`var`, `pull`, `import`, and `icon_theme` are not elements and keep their own shape. `var` and
`pull` still write `name=value`, because there the name is data - the variable being declared -
and not a field of the node; `import` and `icon_theme` take a single string argument:

```kdl
var lang="en"
pull battery="cat /sys/class/power_supply/BAT0/capacity" i="30s" default="0%"
import "./colors.kdl"
icon_theme "Gruvbox-Plus-Dark"
```

Only `var` and `pull` take `name=value` properties. Writing one on an element - either on the
header (`button b child=t1`) or on a field (`w portion=2`) - is an error, and so is giving a
field the wrong number of values (`clip #true #false`, `padding 1 2 3`). A field name the node
doesn't recognise is a warning and is ignored, so a misspelled field silently does nothing -
run `iwwc --check` after editing and fix the warnings too, not just the errors.

## Splitting the config with `import`

`import "path"` includes another KDL file:

```kdl
import "./colors.kdl"
import "/home/$USER/widgets/bar.kdl"
```

A relative path is resolved relative to the file containing the `import`. Imported files are full
config files and may contain `import` to chainload. Each file is imported at most once: importing
an already-loaded file (including circular chains) is skipped with a warning.

## References and order

Declarations can appear in any order - an element may reference another that is defined later in
the file. All references are checked when the config loads: a `child`, `style`, `border`,
`shadow` or `font` field that names a missing id is an `unresolved reference` error, and rejected.

## Reloading

`iwwc reload` re-reads the same config file while the daemon runs. Open widgets are re-created
with the new config. If the new file has errors, they're printed and the daemon keeps the
previous config.
