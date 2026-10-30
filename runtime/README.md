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
contains `step`, `actor`, and `text`. Bus errors propagate to the caller and put
the session in the `failed` state.

Use `session:cancel()` to end a session and `session:status()` to inspect it.
A session plays one scene; create another session to play it again. Treat the
compiled program as read-only; it can be shared across sessions.

Game adapters own player identity checks and client/server transport.
The runtime never creates remotes or accesses player data itself.

Build a Studio module with:

```sh
rojo build runtime/default.project.json -o build/Cue.rbxm
```

The Wally manifest is ready for a future registry release. It is not published yet.
