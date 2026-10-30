# Architecture

Cue is the layer between designer-authored content and a game's capabilities.
Programmers expose reusable events, queries, and commands. Designers compose
them into world interactions, dialogue, and quests.

## Compiler

The Rust library scans and parses source, checks references, then generates a
Luau module. The CLI handles files and reports diagnostics. Both `build` and
`check` run the same checks.

The current pipeline is:

```text
source -> tokens -> syntax tree -> checked program -> Luau module
```

Syntax nodes retain byte spans for diagnostics. The checked program contains
resolved actor IDs. Output includes a format version; incompatible runtimes
reject it before starting a session.

The first implementation covers actors and dialogue scenes. Capability schemas,
expressions, choices, event waits, and quest state are subsequent language work.

## Runtime

One compiled program can be shared across independent sessions. Each session
owns its cursor, context, and status. It emits a dialogue event and waits for
that step to be acknowledged before continuing. Duplicate or stale steps do
not advance execution.

The runtime has no transport or Roblox service dependencies. Game adapters
route events to presentation and validate requests arriving from clients.
Runtime errors are surfaced to the caller rather than logged and ignored.

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
