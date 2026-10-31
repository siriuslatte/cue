# Language semantics

These rules define the behavior to preserve as Cue grows. The current compiler
supports the subset described in [Language](language.md). The other constructs
below are specified for implementation; they are not available in the CLI yet.

## Values

| Type | Meaning |
| --- | --- |
| `bool` | `true` or `false` |
| `number` | A finite, double-precision floating-point number |
| `string` | UTF-8 text |
| List | An ordered sequence of values of one type |
| Record | Named fields with known types |
| Enum | A named type with distinct variants and optional payloads |
| `T?` | A value of type `T`, or `none` |
| Handle | A typed reference to something exposed by the game contract |

Local types are inferred from their initializers unless an annotation is given.
A binding's type does not change when its value changes. There are no implicit
conversions between numbers, strings, and booleans. An optional `T?` can hold a
`T` without an explicit wrapper; `none` needs an annotation or another expression
that determines `T`.

Records and lists have value semantics. Assignment and argument passing do not
create shared mutable aliases. Game handles retain their identity when copied;
copying a handle does not copy the object it identifies.

### Numbers

Numeric literals use decimal notation, with an optional fractional part and
exponent. A leading minus is an operator, not part of the literal.

Arithmetic uses IEEE 754 binary64 values. Exact integer arithmetic is only
guaranteed within the safe integer range, `-(2^53 - 1)` through `2^53 - 1`.
Use game handles or strings for opaque identifiers. Decimal fractions can round:
`0.1 + 0.2 == 0.3` is `false`.

Division by zero, remainder by zero, and arithmetic producing a non-finite value
are runtime errors when evaluated. A literal outside the finite range is a
compile error. `NaN` and infinity are not Cue values, including values delivered
by an adapter.

`%` is the floor remainder: `a - floor(a / b) * b`. For example, `-5 % 3` is `1`.
Negative zero compares equal to zero. Constant folding must preserve the numeric
results, evaluation order, and errors of the generated program.

### Strings and collections

Strings use the escapes described in [Language](language.md). Equality compares
their exact UTF-8 contents, without case folding or Unicode normalization.
`..` concatenates two strings. `+` only adds numbers.

Lists use one-based integer indices. Reading outside the list returns `none`,
so a list lookup has type `T?`. A non-integer index is an error. Writing requires
a mutable binding and an existing index; it does not grow the list implicitly.

Accessing an unknown record field is a compile error. An optional field can be
omitted when constructing a record; its value is then `none`.

## Operators

The table is ordered from highest to lowest precedence. Parentheses override it.
Assignment is a statement and does not produce a value.

| Operators | Associativity | Operands and result |
| --- | --- | --- |
| Calls, `.field`, `[index]` | Left | Defined by the called capability or value type |
| `^` | Right | Numbers to a number |
| Unary `-`, `not` | Right | Number to number; boolean to boolean |
| `*`, `/`, `%` | Left | Numbers to a number |
| `+`, `-` | Left | Numbers to a number |
| `..` | Right | Strings to a string |
| `<`, `<=`, `>`, `>=`, `==`, `!=` | No chaining | Compatible values to a boolean |
| `and` | Left | Booleans to a boolean |
| `or` | Left | Booleans to a boolean |
| `??` | Right | An optional value and a fallback to a value |

Power binds more tightly than unary minus, but its right operand may be negated:
`-2 ^ 2` is `-4`, `(-2) ^ 2` is `4`, and `2 ^ -2` is `0.25`.

Comparison chains such as `1 < count < 5` are compile errors. Write
`1 < count and count < 5` instead. Ordering operators accept numbers only.

Equality requires compatible types. Numbers, booleans, and strings compare by
value. Lists compare in order. Records require the same field names and field
types, and compare their field values regardless of declaration order.
Enums must have the same enum type, variant, and equal payloads.
Handles compare by contract type and stable identity. No comparison uses the
identity of a generated Luau table.

Optional values compare as `none` or as their contained values. Comparing an
optional value to a compatible `T` or to `none` is allowed. Comparing unrelated
types, such as a number and a string, is a compile error.

## Conditions and missing values

Conditions require `bool`. This includes `if`, event filters, and the operands
of `not`, `and`, and `or`. Numbers, strings, handles, collections, and `none`
cannot stand in for a boolean. `0` and an empty string are ordinary values.

`none` represents an expected absence, such as a missing optional field or an
out-of-range list lookup. An unknown name, misspelled field, uninitialized
binding, or missing required adapter field is an error, not `none`.

Use a check against `none` to narrow an optional local within a branch, or use
`??` for a fallback. In `value ?? fallback`, the left side must be `T?` and the
fallback must be `T`. The fallback runs only when the left side is `none`.
A present `0`, `false`, or empty string is preserved.

An optional value must be narrowed or given a fallback before arithmetic, field
access, or another operation requiring `T`. A game adapter translates an absent
optional Luau value to `none` at the contract boundary. It validates required
fields and types before delivering values to Cue.

## Bindings and scope

`let name = expression` creates an immutable local. `var name = expression`
creates a mutable local; `name = expression` replaces its value. Assignment to
a `let` binding or to a name that was never declared is a compile error.

A local belongs to its enclosing scene or block. It becomes visible after its
initializer finishes. An initializer can read an outer binding with the same
name, but cannot read the local being initialized. A name cannot be declared
twice in one block. An inner block may shadow an outer local; leaving the block
restores the outer binding.

Each scene call has fresh locals and its own parameters. A callee resolves names
in its defining module, not in the caller's locals. Arguments pass by value,
including lists and records. The call inherits the session context. Persistent
or shared state must use an explicit scope; a local never becomes player, party,
or world state implicitly.

Actor and scene declarations belong to their module and are visible throughout
it, regardless of declaration order. Their names must be unique within their
respective namespaces. An actor and a scene may share a name. Local bindings use
the value namespace; they do not replace declarations used by `[ACTOR]` or scene
calls. An ambiguous imported name must be qualified.

## Evaluation order

Statements in a scene run in source order. A statement finishes or suspends
before the next statement starts. Declaring an actor or scene does not run game
behavior. The host chooses the initial scene.

Within an expression:

1. Evaluate the receiver or callee first.
2. Evaluate operands, arguments, list items, and record fields from left to right
   in source order, subject to the operator's grouping and short-circuit rules.
3. Perform the operation after its inputs have been evaluated.

Named arguments still evaluate in source order, not parameter declaration order.
For assignment, resolve the target and any index before evaluating the new value.
Update the binding only after evaluation succeeds.

`and`, `or`, and `??` short-circuit:

- `false and expression` returns `false` without evaluating `expression`.
- `true or expression` returns `true` without evaluating `expression`.
- A present optional value followed by `??` returns its contents without
  evaluating the fallback.

All branches still undergo name and type checking. For example,
`false and missing_name` is a compile error, but
`false and (1 / 0 > 0)` evaluates to `false`. The division is well-typed and is
never executed. An optimizer must preserve this distinction.

### Queries, commands, and scene calls

Queries are read-only and return immediately. Expressions cannot yield. Query
calls keep their source order and call count; the compiler must not reorder,
duplicate, or merge them.

Commands are explicit operations on the game contract. A command that can suspend
uses `await`, either as a statement or as the whole initializer of a local.
It cannot be hidden inside arithmetic or another call's arguments. Its arguments
are evaluated once before dispatch. The scene resumes once with the declared
result, or ends because of cancellation or a runtime error.

A scene call waits for the callee to return, including any waits inside it.
The caller continues afterward. A scene with no declared result returns no
value and cannot initialize a binding. A scene that declares a result must
produce it on every returning path. Starting independent work requires an
explicit concurrency construct.

Dialogue emits one line, then waits for acknowledgement or choice selection.
The following statement cannot run first. Stale or duplicate acknowledgements
do not advance the scene. Reaching the end of the initial scene completes its
session; reaching the end of a called scene returns to its caller.

Incoming events do not re-enter an expression already being evaluated. They are
processed at session step boundaries. Game state can change while a scene is
suspended, so recheck live conditions after a wait when needed. An ordinary local
is a snapshot and does not update itself when the game changes.

## Errors and cancellation

Compile errors include invalid syntax, unknown names, duplicate declarations,
incompatible types, and missing required return values. A program with a compile
error produces no new output and never starts execution. Unreachable code still
undergoes these checks.

A command's declared failure result is a normal value. Content can branch on it
before using a success payload. It does not fail the session by itself.

Runtime errors include invalid arithmetic, adapter exceptions, invalid values
from a trusted adapter, and execution budget violations. An unhandled runtime
error unwinds the active scene calls and fails that session. Remaining statements
do not run. The error reports its Cue source location and call context; it is
not logged and ignored. Other independent sessions remain unaffected.

Failure or cancellation releases the session's waits and cancels unfinished work
it owns. Cancellation ends the session without a return value and is distinct
from a runtime error. A command timeout follows its declared contract: it either
returns a typed timeout result or fails the session. Late results cannot resume
an ended session.

Completed game actions are not rolled back automatically. No command is retried
implicitly. Retries and duplicate protection follow the capability contract,
including idempotency keys where needed.
