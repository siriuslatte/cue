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
