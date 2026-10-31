# Language

Cue scenes combine dialogue with typed values and control flow. The host starts
a scene by name; declaring a scene does not run it.

```text
actor MECHANIC "mechanic"

scene INTRO {
    let parts = ["bolt", "gear"]
    for part in parts {
        [MECHANIC] ("Found a " .. part)
    }
    [MECHANIC] "That should do it."
}
```

`actor` maps a content name to a game identifier. Actor and scene declarations
can appear in any order, with unique names within their respective namespaces.
Dialogue accepts a quoted string or a parenthesized string expression. Each line
waits for acknowledgement before the next statement runs.

## Locals and types

```text
let required = 3
var found = 0
found = found + 1
let note: string? = none
let empty: number[] = []
let repair: { count: number, label: string? } = { count: found }
```

`let` is immutable; `var` permits assignment. Types are inferred unless annotated.
Bindings require an initializer, keep their type, and belong to their enclosing
block. Inner blocks may shadow outer names. An initializer sees the outer scope.

Types are `number`, `bool`, `string`, `T[]`, `T?`, structural records, and named
enums. Use annotations when `none` or an empty list leaves a type unknown.
Optional record fields can be omitted; required fields cannot.

Lists have one-based integer indices. A read outside the list returns `none`;
a write must find an existing item. Records and lists have value semantics:

```text
var first = [{ count: 1 }]
var second = first
second[1].count = 9
let original = first[1] ?? { count: 0 } # original.count is still 1
```

## Expressions and conditions

Arithmetic uses finite double-precision numbers. `/` and `%` reject zero divisors;
non-finite results fail the session. `..` joins strings. There are no implicit
conversions. `string(value)` explicitly formats a number, boolean, or string;
`len(list)` returns a list's length.

`and`, `or`, and `not` require booleans. `??` supplies a fallback for an optional
value, preserving present `false`, `0`, and empty strings. These operators
short-circuit. Both branches still undergo type checking.

```text
let count: number? = 2
if count != none and count >= 2 {
    [MECHANIC] ("Found " .. string(count) .. " parts")
} else if (count ?? 0) == 0 {
    [MECHANIC] "Keep looking."
} else {
    [MECHANIC] "One more part."
}
```

A check against `none` narrows a local in the appropriate branch. Replacing that
local invalidates the narrowing. Lists, records, and enum payloads compare by
value. Ordering accepts numbers only; comparison chains are rejected.
See [semantics](semantics.md#operators) for the full precedence table.

## Enums

Enums are named types. A variant can carry one payload, including a record.
`match` evaluates its value once and requires every variant exactly once.
Payload bindings are immutable and scoped to their arm.

```text
enum Repair {
    Ready({ parts: number }),
    Missing,
}

scene CHECK_REPAIR {
    let repair = Repair.Ready({ parts: 3 })
    match repair {
        Repair.Ready(details) {
            [MECHANIC] ("Ready with " .. string(details.parts) .. " parts")
        }
        Repair.Missing {
            [MECHANIC] "Come back with the missing parts."
        }
    }
}
```

Enum declarations and payload types can refer to types declared later. Variants
without a payload use `Repair.Missing`, without parentheses. Different enum types
cannot be compared or assigned to each other.

## Bounded iteration

`for item in list { ... }` evaluates the list once and visits its items in order.
The item binding is immutable. Updates to the original list do not change that
snapshot. Locals and loop position survive dialogue waits. Empty lists run no
iterations. There is no `while` loop or implicit task spawning.

The runtime limits work between dialogue waits. See the
[runtime options](../runtime/README.md) to adjust that budget.

## Source and output

Identifiers use ASCII letters, digits, and underscores, and cannot start with a
digit. Strings contain UTF-8, use double quotes, and support `\"`, `\\`, `\n`,
`\r`, `\t`, and `\0`. `#` starts a comment outside a string.

Write statements on separate lines, or separate them with `;`. A postfix call,
field access, or index must start on the same line as its receiver. The contents
of lists, records, calls, and parentheses can span lines. Expressions may nest
up to 48 levels; the parser also limits syntax nesting to 128 levels.

```sh
cue check examples/hello.cue
cue build examples/hello.cue -o build/hello.luau
```

Both commands check names and types. Errors report the Cue file, line, and
column, and produce no partial output. The generated Luau module uses program
format 2: a version and a table of scene functions. The runtime provides the
execution helpers; game scripts should use the session API. Compiler and runtime
versions are released together.

Game queries, commands, handles, and scene calls are not implemented yet.
[Semantics examples](semantics-examples.md) track implemented and future cases.
