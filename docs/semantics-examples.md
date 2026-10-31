# Semantics examples

These cases give concrete outcomes for the [language semantics](semantics.md).
Keep their IDs when turning them into compiler or runtime conformance tests.
Error expectations describe the cause, not a required diagnostic message.

## Current compiler

The following fixtures use the implemented actor and scene syntax:

| ID | Fixture | Expected outcome |
| --- | --- | --- |
| C01 | [Forward actors](../crates/cue-compiler/tests/fixtures/pass/forward-actors.cue) | Checking succeeds; the scene resolves an actor declared later in the file |
| C02 | [Separate namespaces](../crates/cue-compiler/tests/fixtures/pass/namespaces.cue) | Checking succeeds; the actor and scene may share a name |
| C03 | [Unknown actor](../crates/cue-compiler/tests/fixtures/fail/unknown-actor.cue) | Checking fails at the unknown actor; no output is produced |
| C04 | [Duplicate declarations](../crates/cue-compiler/tests/fixtures/fail/duplicates.cue) | Checking fails for the second actor and scene declarations |
| C05 | [Escaped text](../crates/cue-compiler/tests/fixtures/pass/strings.cue) | Compilation preserves UTF-8, quotes, backslashes, newlines, and a null byte followed by digits |

When a scene contains two lines, starting it emits only the first line.
Acknowledging that step emits the second. A stale acknowledgement does not skip
a line; acknowledging the second step completes the session.

## Specified expressions

The remaining cases describe features that are not implemented yet. Test an
expression as the initializer of `let result = expression` in a scene. Check
errors reject the program; runtime errors occur only if the expression runs.

### Arithmetic

| ID | Expression | Expected outcome |
| --- | --- | --- |
| A01 | `2 * 3 + 4` | `number`, `10` |
| A02 | `2 + 3 * 4` | `number`, `14` |
| A03 | `10 - 3 - 2` | `number`, `5` |
| A04 | `20 / 2 / 2` | `number`, `5` |
| A05 | `2 ^ 3 ^ 2` | `number`, `512` |
| A06 | `-2 ^ 2` | `number`, `-4` |
| A07 | `(-2) ^ 2` | `number`, `4` |
| A08 | `2 ^ -2` | `number`, `0.25` |
| A09 | `-5 % 3` | `number`, `1` |
| A10 | `1 / 0` | Runtime error: division by zero |
| A11 | `1 % 0` | Runtime error: remainder by zero |
| A12 | `1e308 * 1e308` | Runtime error: non-finite result |
| A13 | `1e309` | Check error: literal outside the finite range |
| A14 | `0.1 + 0.2 == 0.3` | `bool`, `false` |

### Conditions and strings

| ID | Expression | Expected outcome |
| --- | --- | --- |
| B01 | `false or true and false` | `bool`, `false` |
| B02 | `false and (1 / 0 > 0)` | `bool`, `false`; division is not evaluated |
| B03 | `true or (1 / 0 > 0)` | `bool`, `true`; division is not evaluated |
| B04 | `false and missing_name` | Check error: unknown name |
| B05 | `false and ("three" + 1 > 0)` | Check error: incompatible operands, even in a skipped branch |
| B06 | `0 and true` | Check error: boolean operand required |
| B07 | `not ""` | Check error: boolean operand required |
| B08 | `1 < 2 < 3` | Check error: comparison chain |
| B09 | `1 < 2 and 2 < 3` | `bool`, `true` |
| B10 | `"3" == 3` | Check error: incompatible types |
| B11 | `"Hello, " .. "mechanic"` | `string`, `"Hello, mechanic"` |
| B12 | `"Hello, " + "mechanic"` | Check error: numeric operands required |

### Optional values

Use these bindings for O01 through O06:

```text
let missing: number? = none
let count: number? = 0
let flag: bool? = false
let name: string? = ""
```

| ID | Expression | Expected outcome |
| --- | --- | --- |
| O01 | `missing ?? 5` | `number`, `5` |
| O02 | `count ?? 5` | `number`, `0` |
| O03 | `flag ?? true` | `bool`, `false` |
| O04 | `name ?? "fallback"` | `string`, `""` |
| O05 | `missing + 1` | Check error: optional value must be handled |
| O06 | `count ?? (1 / 0)` | `number`, `0`; fallback is not evaluated |

## Specified scope rules

### S01: block shadowing

```text
let count = 3
if true {
    let count = count + 2
}
let result = count
```

The inner initializer reads the outer `count`. The inner binding is `5`, and
`result` is `3` after the block ends.

### S02: value copies

```text
var first = [1, 2]
var second = first
second[1] = 9
let result = first[1] ?? 0
```

`result` is `1`. Updating `second` does not change `first`.

| ID | Scenario | Expected outcome |
| --- | --- | --- |
| S03 | Declare `let count = 3`, then `let count = 5` in the same block | Check error: duplicate local |
| S04 | Initialize `let count = count + 1` with no outer `count` | Check error: unknown name; no implicit zero |
| S05 | Assign `count = 5` after declaring `let count = 3` | Check error: immutable binding |
| S06 | A called scene reads a caller's local without receiving it as a parameter | Check error: unknown name in the callee |

## Specified evaluation order

Use a mock query `Probe.read(label: string) -> number` that records its calls in
the test trace. This trace is test instrumentation, not a game-state mutation.

| ID | Expression and mock values | Expected outcome |
| --- | --- | --- |
| E01 | `Probe.read("a") - Probe.read("b") - Probe.read("c")`, with `a=10`, `b=3`, `c=2` | `5`; reads occur in order `a`, `b`, `c` |
| E02 | `Probe.read("a") ^ Probe.read("b") ^ Probe.read("c")`, with `a=2`, `b=3`, `c=2` | `512`; reads still occur in order `a`, `b`, `c` |
| E03 | `Probe.read("a") + Probe.read("a")`, with `a=1` | `2`; two reads, not one |

For E04, expose `Probe.combine(a: number, b: number) -> number`, returning
`a - b`. Let the mock query return `first=1` and `second=2`.

```text
let result = Probe.combine(
    b: Probe.read("first"),
    a: Probe.read("second")
)
```

The trace is `read("first")`, `read("second")`, then `combine(a: 2, b: 1)`.
`result` is `1`; named arguments do not reorder evaluation.

## Specified execution outcomes

These cases need a runtime harness with controllable commands and events.

| ID | Scenario | Expected outcome |
| --- | --- | --- |
| X01 | A command returns its declared failure result, and the scene handles it | The failure branch runs; the result alone does not fail the session |
| X02 | A scene acknowledges one line, then evaluates `1 / 0` before another line | The session fails at the division; the next line is never emitted |
| X03 | A called scene raises a runtime error | The error fails its session; later caller statements do not run |
| X04 | A session is cancelled while awaiting a command, then receives its result | Owned waits are released; the session remains cancelled and does not resume |
| X05 | A command completes before a later statement fails | The earlier game action is not rolled back or retried |
| X06 | One of two independent sessions fails | The other session continues with its own locals and context |

Assert error locations, call context, final status, emitted events, and command
traces. Check missing events and commands explicitly rather than treating a lack
of exceptions as success.
