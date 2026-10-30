local fs = require("@lune/fs")
local luau = require("@lune/luau")
local process = require("@lune/process")
local Cue = require("../runtime/src")

local path = assert(process.args[1], "usage: lune run tests/integration.luau <compiled module>")
local program = luau.load(fs.readFile(path), { debugName = path })()
local events = {}
local context = { playerId = 42 }
local session = Cue.new(program, {
    emit = function(event, receivedContext)
        assert(receivedContext == context)
        table.insert(events, event)
    end,
}, context)

session:start("INTRO")
assert(events[1].actor == "mechanic")
assert(events[1].text == "The lighthouse needs a few repairs.")
assert(session:advance(events[1].step))
assert(events[2].text == "Come back when you've found the parts.")
assert(session:advance(events[2].step))
assert(events[3].kind == "completed")
assert(#events == 3 and session:status() == "completed")

print("compiler/runtime integration passed")
