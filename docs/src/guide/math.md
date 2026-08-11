# Interpolation & Math

Any quoted field value may embed `${...}` expressions. Text around and between
them passes through; each expression is evaluated at resolve time (see
[Variables & Polling](./variables.md)):

```kdl
text t1 "${round(iwwc.ram.used / 1073741824).1}G used"
```

Nesting `${...}` inside `${...}` is an error.

## Values

- integers and floats: `40`, `1.5`
- booleans: `#true` / `#false` (KDL syntax), and every comparison result
- strings: single-quoted literals, `'wsactive'`. There is **no escape
  mechanism** - a literal `'` cannot appear inside a string literal
- variables: `myvar`, `iwwc.cpu.avg.usage`; referencing an unknown variable
  is a config error at load time

Single quotes are used because the expression itself already lives inside a
KDL double-quoted string.

## Operators

From tightest to loosest binding:

| Operators | Meaning |
|---|---|
| `!` `-` (unary) | boolean not; numeric negation |
| `^` | power, right-associative |
| `*` `/` `%` | multiply, divide, modulo |
| `+` `-` | add, subtract; `+` on two strings concatenates |
| `<` `<=` `>` `>=` | comparison |
| `==` `!=` | equality; `Int` and `Float` compare numerically |
| `&&` | boolean and, short-circuits |
| `\|\|` | boolean or, short-circuits |
| `cond ? a : b` | ternary, right-associative |

`&&`, `\|\|`, `!` and the ternary condition require booleans - a bare number
is a type error, not truthy. Comparisons and equalities cannot be chained
(`1 < 2 < 3` and `a == b == c` are errors). `+` concatenates only when both
sides are strings; mixing a string with a number is an error. Numeric fields
reject non-finite results (`NaN`, division blow-ups).

The ternary evaluates only the taken branch, so the branches may differ in
type and the untaken branch may reference variables that would not resolve:

```kdl
bg "${w.active ? '#89b4fa' : '#45475a'}"
action "${w.exists && w.can_switch ? '$IWWC ws switch ' + w.id : ''}"
```

## Functions

`min(a, b)`, `max(a, b)`, and `round(x)` - the suffix `round(x).N` keeps `N`
decimals.
