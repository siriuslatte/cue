--!strict

local Lest = require("@lest")
local describe, it, expect = Lest.describe, Lest.it, Lest.expect
local fs = require("@lune/fs")
local luau = require("@lune/luau")
local process = require("@lune/process")
local Cue = require("../../runtime/src")

local binary = process.env.CUE_BIN
    or if process.os == "windows" then "target/debug/cue.exe" else "target/debug/cue"

local function compile(name: string): Cue.Program
    local source = `crates/cue-compiler/tests/fixtures/pass/{name}.cue`
    local output = `build/tests/{name}.luau`
    local result = process.exec(binary, { "build", source, "-o", output })
    expect(result.stderr).toBe("")
    expect(result.code).toBe(0)
    return luau.load(fs.readFile(output), { debugName = output })()
end

local function record(program: Cue.Program)
    local events: { Cue.Event } = {}
    local session = Cue.new(program, {
        emit = function(event)
            table.insert(events, event)
        end,
    }, nil)
    return session, events
end

describe("compiled programs", function()
    it("resolves forward actors and runs dialogue in order", function()
        local session, events = record(compile("forward-actors"))
        session:start("INTRO")
        expect(events).toEqual({
            { kind = "dialogue", step = 1, actor = "mechanic", text = "First line" },
        })
        expect(session:advance(1)).toBe(true)
        expect(session:advance(2)).toBe(true)
        expect(events).toEqual({
            { kind = "dialogue", step = 1, actor = "mechanic", text = "First line" },
            { kind = "dialogue", step = 2, actor = "mechanic", text = "Second line" },
            { kind = "completed" },
        })
        expect(session:status()).toBe("completed")
    end)

    it("keeps actor and scene namespaces separate", function()
        local session, events = record(compile("namespaces"))
        session:start("INTRO")
        expect(events[1]).toEqual({
            kind = "dialogue",
            step = 1,
            actor = "mechanic",
            text = "Hello",
        })
    end)

    it("preserves strings through compilation and execution", function()
        local session, events = record(compile("strings"))
        session:start("INTRO")
        expect(events[1]).toEqual({
            kind = "dialogue",
            step = 1,
            actor = "mechanic",
            text = 'Café #1: "hello"\\\n',
        })
        session:advance(1)
        expect(events[2]).toEqual({
            kind = "dialogue",
            step = 2,
            actor = "mechanic",
            text = "Null: \0" .. "123",
        })
    end)
end)

return nil
