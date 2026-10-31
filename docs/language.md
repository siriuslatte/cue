# Language

The initial syntax supports actor declarations and sequential dialogue scenes.
This is a small executable starting point, not the complete language design.

```text
actor MECHANIC "mechanic"

scene INTRO {
    [MECHANIC] "The lighthouse needs a few repairs."
    [MECHANIC] "Come back when you've found the parts."
}
```

`actor` maps a content name to a game identifier. `scene` groups dialogue lines.
The host starts a scene by name; declaring a scene does not run it.
Actor declarations can appear before or after scenes. Actor and scene names
must be unique within their respective namespaces.

Identifiers use ASCII letters, digits, and underscores, and cannot start with a
digit. Strings use double quotes and support `\"`, `\\`, `\n`, `\r`, `\t`, and
`\0`. Dialogue text supports UTF-8. A `#` starts a comment outside a string.

Malformed syntax, duplicate declarations, and unknown actors fail compilation.
No partial program is written. Diagnostics report the source file, line, and
column.

## Commands

```sh
cue check examples/hello.cue
cue build examples/hello.cue -o build/hello.luau
```

The output is a Luau data module accepted by the runtime. Version 1 contains a
`scenes` table whose entries are ordered `{ actor, text }` records. This format
is internal and can change as the language develops; compiler and runtime
versions are released together.

## Semantics

[Language semantics](semantics.md) defines values, operators, scope, evaluation
order, and errors for the language as it develops. [Semantics examples](semantics-examples.md)
records the expected outcomes and distinguishes implemented cases from future
conformance cases.
