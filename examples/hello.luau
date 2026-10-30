local fs = require("@lune/fs")
local luau = require("@lune/luau")
local Cue = require("../runtime/src")

local program = luau.load(fs.readFile("build/hello.luau"))()
local pendingStep: number? = nil

local session = Cue.new(program, {
    emit = function(event, _context)
        if event.kind == "dialogue" then
            print(`{event.actor}: {event.text}`)
            pendingStep = event.step
        end
    end,
}, nil)

session:start("INTRO")
while session:status() == "waiting" do
    session:advance(assert(pendingStep))
end
