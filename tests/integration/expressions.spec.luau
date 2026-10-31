--!strict

local Lest = require("@lest")
local describe, it, expect = Lest.describe, Lest.it, Lest.expect
local Support = require("./support")
local Cue = require("../../runtime/src")

local function record(name: string, options: Cue.Options?)
    return Support.record(Support.compile(`conformance/{name}`), options)
end

local function texts(scene: string): { string }
    local session, events = record("values")
    session:start(scene)
    local step = 1
    while session:status() == "waiting" do
        expect(session:advance(step)).toBe(true)
        step += 1
    end
    expect(session:status()).toBe("completed")
    local result = {}
    for _, event in events do
        if event.kind == "dialogue" then
            table.insert(result, event.text)
        end
    end
    return result
end

describe("expression conformance", function()
    local cases = {
        { "A01", "10" },
        { "A02", "14" },
        { "A03", "5" },
        { "A04", "5" },
        { "A05", "512" },
        { "A06", "-4" },
        { "A07", "4" },
        { "A08", "0.25" },
        { "A09", "1" },
        { "A14", "false" },
        { "B01", "false" },
        { "B02", "false" },
        { "B03", "true" },
        { "B09", "true" },
        { "B11", "Hello, mechanic" },
        { "P01", "1.4142135623730951" },
        { "P02", "2" },
        { "P04", "true" },
        { "P05", "abc" },
        { "P06", "true" },
        { "P07", "true" },
        { "P08", "true" },
    }
    for _, case in cases do
        it(`evaluates {case[1]}`, function()
            local session, events = record("expressions")
            session:start(case[1])
            expect(events[1]).toEqual({ kind = "dialogue", step = 1, actor = "npc", text = case[2] })
            session:advance(1)
            expect(session:status()).toBe("completed")
        end)
    end

    local failures = {
        { "A10", "A10:93:18: division or remainder by zero" },
        { "A11", "A11:99:18: division or remainder by zero" },
        { "A12", "A12:105:18: arithmetic result must be finite" },
        { "A15", "A15:111:18: arithmetic result must be finite" },
        { "I01", "I01:117:18: list index must be an integer" },
    }
    for _, case in failures do
        it(`fails {case[1]} at the Cue source location`, function()
            local session, events = record("expressions")
            session:start(case[1])
            expect(function()
                session:advance(1)
            end).toThrow(case[2])
            expect(session:status()).toBe("failed")
            expect(events).toHaveLength(1)
            expect(session:advance(1)).toBe(false)
        end)
    end

    it("validates an assignment target before its right side", function()
        local session, events = record("expressions")
        expect(function()
            session:start("WRITE_BOUNDS")
        end).toThrow("out of bounds")
        expect(session:status()).toBe("failed")
        expect(events).toHaveLength(0)
    end)

    it("rejects fractional write indices", function()
        local session = record("expressions")
        expect(function()
            session:start("WRITE_FRACTION")
        end).toThrow("integer")
        expect(session:status()).toBe("failed")
    end)

    it("evaluates record fields in source order", function()
        local session = record("expressions")
        expect(function()
            session:start("FIELD_ORDER")
        end).toThrow("division or remainder by zero")
    end)

    it("keeps suspended locals and loop steps separate between sessions", function()
        local first, events = record("expressions")
        local second, otherEvents = record("expressions")
        first:start("LOOP")
        second:start("LOOP")
        first:advance(1)
        expect(first:advance(1)).toBe(false)
        expect(events[2]).toEqual({ kind = "dialogue", step = 2, actor = "npc", text = "2" })
        expect(otherEvents).toHaveLength(1)
        first:cancel()
        second:advance(1)
        second:advance(2)
        expect(otherEvents[3]).toEqual({ kind = "dialogue", step = 3, actor = "npc", text = "3" })
        expect(first:advance(2)).toBe(false)
    end)

    it("bounds empty and nested loops between waits", function()
        local session = record("expressions", { maxSteps = 20 })
        expect(function()
            session:start("BUDGET")
        end).toThrow("execution budget exceeded")
        expect(session:status()).toBe("failed")
        local other = record("expressions")
        other:start("BUDGET")
        expect(other:status()).toBe("completed")
    end)
end)

describe("value semantics", function()
    it("copies nested values and iterates a snapshot across dialogue waits", function()
        expect(texts("INTRO")).toEqual({
            "1:bolt",
            "9:spring",
            "bolt",
            "gear",
            "Ready with 2 parts",
        })
    end)
    it("preserves false, zero, empty strings, and optional list items", function()
        expect(texts("OPTIONALS")).toEqual({ "5", "0", "false", "", "omitted", "3", "true", "true" })
    end)
    it("compares records, lists, and enum payloads by value", function()
        expect(texts("EQUALITY")).toEqual({
            "true",
            "false",
            "true",
            "true",
            "true",
            "true",
            "true",
        })
    end)
    it("keeps lexical scopes, optional refinements, and match payloads", function()
        expect(texts("SCOPES")).toEqual({ "5", "3", "3", "else if", "7", "missing" })
    end)
end)

describe("large content", function()
    it("runs scenes beyond Luau's local register limit", function()
        local source = { 'actor NPC "npc"', "scene TEST {" }
        for index = 1, 400 do
            table.insert(source, `let item{index} = {index}`)
        end
        table.insert(source, "[NPC] (string(item1 + item400))")
        table.insert(source, "}")
        local session, events =
            Support.record(Support.source("many-locals", table.concat(source, "\n")))
        session:start("TEST")
        expect(events[1]).toEqual({ kind = "dialogue", step = 1, actor = "npc", text = "401" })
    end)

    it("constructs wide lists without retaining every expression temporary", function()
        local items = {}
        for index = 1, 400 do
            table.insert(items, `{index} + 1`)
        end
        local source = 'actor NPC "npc" scene TEST { let items = ['
            .. table.concat(items, ",")
            .. "]; [NPC] (string(items[400] ?? 0)) }"
        local session, events = Support.record(Support.source("wide-list", source))
        session:start("TEST")
        expect(events[1]).toEqual({ kind = "dialogue", step = 1, actor = "npc", text = "401" })
    end)
end)

return nil
