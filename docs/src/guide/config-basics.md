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

text greeting_txt text="Hello World" font="${ff}"

button greeting child=greeting_txt action="notify-send hi" {
  padding 5 13
}
```

The pieces of a declaration:

- **Node name** (`var`, `text`, `button`, `widget`, …) says what is being declared.
Unknown node names are an error.
- **First bare argument** is the declaration's *id* that will be used to call that element.
- **Properties** are `key=value` pairs. Booleans are written `#true` and `#false` (KDL 2 syntax).
Strings are quoted; a trailing `\` continues a node on the next line.
- **A block `{ … }`** holds multi-value fields that don't fit in a single property: `padding 5 13`,
`margin 5 8 0 8`, `radius 10 10 0 0`, `offset 2 2`, and `children a b c`.

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
the file. All references are checked when the config loads: a `child=`, `style=`, `border=`,
`shadow=` or `font=` that names a missing id is an `unresolved reference` error, and rejected.

## Reloading

`iwwc reload` re-reads the same config file while the daemon runs. Open widgets are re-created
with the new config. If the new file has errors, they're printed and the daemon keeps the
previous config.
