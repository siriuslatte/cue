# Architecture

Cue is the layer between designer-authored content and a game's capabilities.
Programmers expose reusable events, queries, and commands. Designers compose
them into world interactions, dialogue, and quests.

## Compiler

The Rust library scans and parses source, checks names and types, then generates a
Luau module. The CLI handles files and reports diagnostics. Both `build` and
`check` run the same checks.

The current pipeline is:

```text
source -> tokens -> syntax tree -> checked program -> Luau module
```

Syntax nodes retain byte spans for diagnostics. The checked tree contains types,
resolved actor IDs, and unique local IDs. Type checking finishes before emission;
generated code has no runtime type inference. Output includes a format version;
incompatible runtimes reject it before starting a session.

Expressions become scoped temporaries in source order. Branches and loops use
native Luau control flow. Small scenes use Luau locals; scenes with more than
32 bindings use an indexed frame to leave registers for expression evaluation.
Temporary scopes keep wide collections from exhausting Luau registers.

Capability schemas, scene calls, choices, event waits, and quest state are
subsequent language work.

## Runtime

One compiled program can be shared across independent sessions. Each session
owns its coroutine, step counter, context, and status. It emits a dialogue event and waits for
that step to be acknowledged before continuing. Duplicate or stale steps do
not advance execution.

The runtime has no transport or Roblox service dependencies. Game adapters
route events to presentation and validate requests arriving from clients.
Dialogue yields to the session driver. Acknowledgement resumes the coroutine;
failure and cancellation close it and release its locals. Arithmetic and index
errors include the scene name and Cue line and column.

Collection values are frozen. Assigning a binding shares its value; an update
clones only containers along the path from the changed field to the root.
Unchanged children stay shared. Equality walks distinct table pairs iteratively,
so deep values and shared subtrees do not cause recursive stack growth or
repeated comparisons.

A configurable execution budget counts statements, loop iterations, and table
pairs visited by equality between dialogue waits. It stops empty nested loops
from blocking the host indefinitely. It is a work limit for compiled content,
not a sandbox for arbitrary Luau modules.

## Game contract

The current bus only carries built-in presentation events. The next boundary is
a typed capability contract with:

- events for facts delivered by the game;
- queries for current game state;
- commands for game actions and their results.

That contract should drive compiler checks, adapter interfaces, and editor
completion. Context identifies the player, party, or world session. New content
using existing capabilities should not require a new game adapter.

Reactive queries will need declared dependencies, and waits will need explicit
subscription ownership and cancellation. Persistence will use stable content
IDs and checkpoints rather than raw generated-code offsets.
