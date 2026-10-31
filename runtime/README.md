# Cue runtime

Runs compiled Cue programs and delivers presentation events to a game adapter.
It has no Roblox service dependencies, so the same module runs in Studio and Lune.

```luau
local session = Cue.new(program, {
    emit = function(event, context)
        -- Route the event to the game's bus or UI.
    end,
}, player)

session:start("INTRO")
```

`start` emits the first `dialogue` event. Call `session:advance(event.step)` when
that line finishes. A step can be acknowledged once; stale acknowledgements
return `false`. Each session has its own cursor and context.

The bus receives `dialogue`, `completed`, and `cancelled` events. A dialogue event
contains `step`, `actor`, and `text`. Bus and evaluation errors propagate to the caller and put the session in the
`failed` state. Arithmetic and index errors identify the scene and Cue line and
column. Failure and cancellation close the suspended scene.

Use `session:cancel()` to end a session and `session:status()` to inspect it.
A session plays one scene; create another session to play it again. Treat the
compiled program as read-only; it can be shared across sessions.

The default work budget is 10,000 steps between dialogue waits. Statements,
loop iterations, and structural equality each consume work. Set a different
limit with the fourth argument: `Cue.new(program, bus, context, { maxSteps = 20000 })`.
A budget violation fails the session. The budget renews on acknowledgement.

This runtime accepts program format 2. Rebuild older generated modules with the
matching compiler; format 1 dialogue tables are no longer accepted.

Game adapters own player identity checks and client/server transport.
The runtime never creates remotes or accesses player data itself.

Build a Studio module with:

```sh
rojo build runtime/default.project.json -o build/Cue.rbxm
```

The Wally manifest is ready for a future registry release. It is not published yet.
