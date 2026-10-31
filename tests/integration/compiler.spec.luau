--!strict

local Lest = require("@lest")
local describe, it, expect = Lest.describe, Lest.it, Lest.expect
local Support = require("./support")
local compile, record = Support.compile, Support.record

describe("compiled programs", function()
    it("resolves forward actors and runs dialogue in order", function()
        local session, events = record(compile("pass/forward-actors"))
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
        local session, events = record(compile("pass/namespaces"))
        session:start("INTRO")
        expect(events[1]).toEqual({
            kind = "dialogue",
            step = 1,
            actor = "mechanic",
            text = "Hello",
        })
    end)

    it("preserves strings through compilation and execution", function()
        local session, events = record(compile("pass/strings"))
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
