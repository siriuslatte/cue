local Cue = require("../runtime/src")

local program = {
    version = 1,
    scenes = {
        INTRO = {
            { actor = "mechanic", text = "First line" },
            { actor = "mechanic", text = "Second line" },
        },
        EMPTY = {},
    },
}

local function record(context)
    local events = {}
    local session = Cue.new(program, {
        emit = function(event, actualContext)
            assert(actualContext == context, "session context was lost")
            table.insert(events, event)
        end,
    }, context)
    return session, events
end

local first, firstEvents = record("player-a")
local second, secondEvents = record("player-b")
first:start("INTRO")
second:start("INTRO")
assert(#firstEvents == 1 and firstEvents[1].text == "First line")
assert(first:status() == "waiting")
assert(not first:advance(99), "unknown acknowledgement advanced the session")
assert(first:advance(firstEvents[1].step))
assert(firstEvents[2].text == "Second line")
assert(#secondEvents == 1, "sessions share a cursor")
assert(not first:advance(firstEvents[1].step), "duplicate acknowledgement advanced the session")
assert(first:advance(firstEvents[2].step))
assert(first:status() == "completed")
assert(firstEvents[3].kind == "completed")
assert(not first:advance(2))
assert(not first:cancel())

assert(second:cancel())
assert(second:status() == "cancelled")
assert(secondEvents[2].kind == "cancelled")
assert(not second:advance(1))
assert(not second:cancel())

local empty, emptyEvents = record(nil)
empty:start("EMPTY")
assert(empty:status() == "completed" and emptyEvents[1].kind == "completed")

local unknown = record(nil)
assert(not pcall(function()
    unknown:start("MISSING")
end))
assert(unknown:status() == "idle")

local failed = Cue.new(program, {
    emit = function()
        error("adapter failed")
    end,
}, nil)
assert(not pcall(function()
    failed:start("INTRO")
end))
assert(failed:status() == "failed")
assert(not failed:advance(1))

assert(not pcall(function()
    Cue.new({ version = 99, scenes = {} }, { emit = function() end }, nil)
end))

print("runtime tests passed")
