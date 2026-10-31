--!strict

local Lest = require("@lest")
local describe, it, expect = Lest.describe, Lest.it, Lest.expect
local Cue = require("../../runtime/src")

local function run(body: (Cue.Execution) -> (), options: Cue.Options?): Cue.Session
    local session = Cue.new(
        { version = 2, scenes = { TEST = body } },
        { emit = function() end },
        nil,
        options
    )
    session:start("TEST")
    return session
end

describe("compiled value operations", function()
    it("keeps finite arithmetic and rejects infinity and NaN", function()
        run(function(cue)
            expect(cue.number(0.1 + 0.2, "TEST:1:1")).toBe(0.1 + 0.2)
            expect(cue.number(-0, "TEST:1:1")).toBe(0)
            expect(function()
                cue.number(math.huge, "TEST:2:3")
            end).toThrow("TEST:2:3: arithmetic result must be finite")
            expect(function()
                cue.number(-math.huge, "TEST:1:1")
            end).toThrow("finite")
            expect(function()
                cue.number(0 / 0, "TEST:1:1")
            end).toThrow("finite")
            cue.divisor(2, "TEST:1:1")
            expect(function()
                cue.divisor(0, "TEST:1:1")
            end).toThrow("division or remainder by zero")
        end)
    end)

    it("distinguishes missing list items from present false values", function()
        run(function(cue)
            local values = table.freeze({ false, cue.none, 0 })
            expect(cue.index(values, 1, false, "TEST:1:1")).toBe(false)
            expect(cue.index(values, 2, false, "TEST:1:1")).toBe(cue.none)
            expect(cue.index(values, 3, true, "TEST:1:1")).toBe(0)
            expect(cue.index(values, 0, false, "TEST:1:1")).toBe(cue.none)
            expect(cue.index(values, 4, false, "TEST:1:1")).toBe(cue.none)
            expect(function()
                cue.index(values, 1.5, false, "TEST:2:4")
            end).toThrow("TEST:2:4: list index must be an integer")
            expect(function()
                cue.index(values, 4, true, "TEST:2:4")
            end).toThrow("out of bounds")
        end)
    end)

    it("shares unchanged children and freezes updated containers", function()
        run(function(cue)
            local child = table.freeze({ 1, 2 })
            local original = table.freeze({ count = 1, child = child })
            local updated = cue.set(original, "count", 2)
            expect(updated.count).toBe(2)
            expect(original.count).toBe(1)
            expect(updated.child).toBe(child)
            expect(table.isfrozen(updated)).toBe(true)
        end)
    end)

    it("compares nested tables structurally without treating none as an empty record", function()
        run(function(cue)
            expect(cue.equal(1, 1, "TEST:1:1")).toBe(true)
            expect(cue.equal(cue.none, cue.none, "TEST:1:1")).toBe(true)
            expect(cue.equal({ a = { 1, false } }, { a = { 1, false } }, "TEST:1:1")).toBe(true)
            expect(cue.equal({ a = 1 }, { a = 2 }, "TEST:1:1")).toBe(false)
            expect(cue.equal({}, { a = 1 }, "TEST:1:1")).toBe(false)
            expect(cue.equal({ a = 1 }, {}, "TEST:1:1")).toBe(false)
            expect(cue.equal({}, cue.none, "TEST:1:1")).toBe(false)
            expect(cue.equal(cue.none, {}, "TEST:1:1")).toBe(false)
            expect(cue.equal(1, true, "TEST:1:1")).toBe(false)
            expect(cue.equal("é", "é", "TEST:1:1")).toBe(false)
        end)
    end)
    it("compares deep shared values without recursion or repeated subtree walks", function()
        run(function(cue)
            local a: any = 1
            local b: any = 1
            for _ = 1, 1000 do
                a = table.freeze({ first = a, second = a })
                b = table.freeze({ first = b, second = b })
            end
            expect(cue.equal(a, b, "TEST:1:1")).toBe(true)
        end)
    end)
end)

describe("execution limits", function()
    it("fails before work exceeds the configured budget", function()
        local events = {}
        local session = Cue.new({
            version = 2,
            scenes = {
                TEST = function(cue)
                    cue.tick("TEST:1:1")
                    cue.tick("TEST:2:1")
                    cue.dialogue("npc", "unreachable")
                end,
            },
        }, {
            emit = function(event)
                table.insert(events, event)
            end,
        }, nil, { maxSteps = 1 })
        expect(function()
            session:start("TEST")
        end).toThrow("TEST:2:1: execution budget exceeded")
        expect(session:status()).toBe("failed")
        expect(session:advance(1)).toBe(false)
        expect(events).toHaveLength(0)
    end)

    it("counts table comparisons against the same execution budget", function()
        expect(function()
            run(function(cue)
                local a: any = 1
                local b: any = 1
                for _ = 1, 10 do
                    a = { child = a }
                    b = { child = b }
                end
                cue.equal(a, b, "TEST:2:4")
            end, { maxSteps = 5 })
        end).toThrow("TEST:2:4: execution budget exceeded")
    end)

    it("renews the budget after each acknowledged dialogue", function()
        local session = run(function(cue)
            for _ = 1, 3 do
                cue.tick("TEST:1:1")
                cue.dialogue("npc", "next")
            end
        end, { maxSteps = 1 })
        expect(session:advance(1)).toBe(true)
        expect(session:advance(2)).toBe(true)
        expect(session:advance(3)).toBe(true)
        expect(session:status()).toBe("completed")
    end)

    for _, invalid in { 0, -1, 1.5, math.huge, 0 / 0 } do
        it(`rejects an invalid budget {invalid}`, function()
            expect(function()
                run(function() end, { maxSteps = invalid })
            end).toThrow("positive finite integer")
        end)
    end

    it("closes a suspended scene on cancellation", function()
        local active: thread? = nil
        local reached = false
        local session = run(function(cue)
            active = coroutine.running()
            cue.dialogue("npc", "wait")
            reached = true
        end)
        session:cancel()
        expect(coroutine.status(active :: thread)).toBe("dead")
        expect(reached).toBe(false)
    end)

    it("closes a suspended scene when the adapter fails", function()
        local active: thread? = nil
        local session = Cue.new({
            version = 2,
            scenes = {
                TEST = function(cue)
                    active = coroutine.running()
                    cue.dialogue("npc", "wait")
                end,
            },
        }, {
            emit = function()
                error("adapter failed")
            end,
        }, nil)
        expect(function()
            session:start("TEST")
        end).toThrow("adapter failed")
        expect(coroutine.status(active :: thread)).toBe("dead")
        expect(session:status()).toBe("failed")
    end)
end)

return nil
