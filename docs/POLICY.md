# Palisade Policy Format

This document is the normative specification of the palisade policy format. Both
the Rust launcher (`palisade`) and the Go inspector (`palisade-lint`) implement
this grammar identically.

## 1. Lexical structure

A policy file is UTF-8 text interpreted line by line (`\n`; a trailing `\r` is
trimmed as surrounding whitespace).

- **Comments** begin with `#` and run to the end of the line. Because the format
  has no quoting layer, a `#` **anywhere** on a line begins a comment. To use a
  literal `#` in a value, you cannot — choose values without `#`.
- **Blank lines** (after comment stripping and trimming) are ignored.
- **Whitespace** around keys, values, and section names is trimmed.

## 2. Grammar

```
line        := comment | blank | section | assignment
section     := '[' ident ']'
assignment  := ident ('=' | '+=') value
value       := any text (trimmed; may be empty)
ident       := [A-Za-z0-9_.-]+
```

- `=` sets a **scalar**. If a key is assigned multiple times with `=`, the last
  assignment wins.
- `+=` **appends** to a list. The order of `=`/`+=` occurrences is preserved for
  list-valued keys.

There is no quoting, escaping, or line continuation. This is deliberate: the
format is trivial to parse, diff, and audit.

## 3. Sections and keys

| Section  | Key           | Kind   | Meaning                                                                 |
|----------|---------------|--------|-------------------------------------------------------------------------|
| (root)   | `name`        | scalar | Human-readable policy name. Default: `unnamed`.                         |
| `exec`   | `program`     | scalar | **Required.** Bare name (PATH-resolved) or a path to the executable.   |
| `exec`   | `timeout_ms`  | scalar | Wall-clock timeout in milliseconds. `0` or absent = no timeout.        |
| `exec`   | `workdir`     | scalar | Working directory for the child process.                               |
| `args`   | `fixed`       | list   | Arguments always prepended, in order.                                  |
| `args`   | `allow`       | list   | Allow-list of additional caller-supplied arguments.                    |
| `args`   | `allow_extra` | scalar | Boolean. When true, any caller argument is permitted.                  |
| `env`    | `mode`        | scalar | `clean` (default) or `inherit`.                                        |
| `env`    | `pass`        | list   | (clean mode) Names copied from the parent environment if present.      |
| `env`    | `remove`      | list   | (inherit mode) Names removed from the inherited environment.           |
| `env`    | `set`         | list   | `KEY=VALUE` pairs applied last, overriding anything above.             |
| `path`   | `root`        | list   | Allowed roots; the resolved program must live under one of them.       |
| `net`    | `mode`        | scalar | `none` (default) or `allow`. **Advisory only** (see §6).               |

Booleans accept `true/yes/1/on` and `false/no/0/off` (case-insensitive).

## 4. Environment resolution

- **clean** (default): start from an empty environment, copy each `env.pass`
  name that exists in the parent, then apply `env.set`.
- **inherit**: start from the full parent environment, delete each `env.remove`
  name, then apply `env.set`.

The effective environment is sorted by key before being handed to the child, so
execution and receipts are deterministic.

## 5. Argument validation

When `args.allow_extra` is false (the default), every caller-supplied argument
must appear verbatim in `args.allow`; otherwise the launcher refuses to run. The
final argument vector is `args.fixed` followed by the validated caller
arguments.

## 6. Executable resolution and path roots

- If `program` contains a path separator, it is treated as a path.
- Otherwise it is resolved against `PATH` (honoring `PATHEXT` on Windows).
- If any `path.root` is declared, the resolved absolute path must be contained
  within at least one root, else the launch is refused. Paths are canonicalized
  where possible for a robust prefix comparison.

## 7. Security boundary (important)

