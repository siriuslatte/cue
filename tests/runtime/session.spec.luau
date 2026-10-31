--!strict

local Lest = require("@lest")
local describe, it, expect, beforeEach = Lest.describe, Lest.it, Lest.expect, Lest.beforeEach
local Cue = require("../../runtime/src")

local program: Cue.Program = {
    version = 1,
    scenes = {
        INTRO = {
            { actor = "mechanic", text = "First line" },
            { actor = "mechanic", text = "Second line" },
        },
        EMPTY = {},
    },
}

describe("session", function()
    local session: Cue.Session
    local events: { Cue.Event }
    local context = { playerId = 42 }

    beforeEach(function()
        events = {}
        session = Cue.new(program, {
            emit = function(event, actualContext)
                expect(actualContext).toBe(context)
                table.insert(events, event)
            end,
        }, context)
    end)

    it("starts idle without emitting events", function()
        expect(session:status()).toBe("idle")
        expect(events).toEqual({})
    end)

    it("emits one line and waits for acknowledgement", function()
        session:start("INTRO")
        expect(session:status()).toBe("waiting")
        expect(events).toEqual({
            { kind = "dialogue", step = 1, actor = "mechanic", text = "First line" },
        })
    end)

    it("advances to the next line", function()
        session:start("INTRO")
        expect(session:advance(1)).toBe(true)
        expect(session:status()).toBe("waiting")
        expect(events).toEqual({
            { kind = "dialogue", step = 1, actor = "mechanic", text = "First line" },
            { kind = "dialogue", step = 2, actor = "mechanic", text = "Second line" },
        })
    end)

    for _, step in { -1, 0, 99 } do
        it(`rejects an unexpected step {step}`, function()
            session:start("INTRO")
            expect(session:advance(step)).toBe(false)
            expect(events).toHaveLength(1)
            expect(session:status()).toBe("waiting")
        end)
    end

    it("rejects a duplicate acknowledgement", function()
        session:start("INTRO")
        session:advance(1)
        expect(session:advance(1)).toBe(false)
        expect(events).toHaveLength(2)
        expect(session:status()).toBe("waiting")
    end)

    it("completes after the last acknowledgement", function()
        session:start("INTRO")
        session:advance(1)
        session:advance(2)
        expect(session:status()).toBe("completed")
        expect(events[3]).toEqual({ kind = "completed" })
        expect(session:advance(2)).toBe(false)
        expect(session:cancel()).toBe(false)
        expect(events).toHaveLength(3)
    end)

    it("completes an empty scene immediately", function()
        session:start("EMPTY")
        expect(session:status()).toBe("completed")
        expect(events).toEqual({ { kind = "completed" } })
    end)

    it("leaves an unknown scene idle", function()
        expect(function()
            session:start("MISSING")
        end).toThrow("unknown scene 'MISSING'")
        expect(session:status()).toBe("idle")
        expect(events).toEqual({})
    end)

    it("rejects starting a session twice", function()
        session:start("INTRO")
        expect(function()
            session:start("INTRO")
        end).toThrow("session has already started")
        expect(events).toHaveLength(1)
    end)

    it("cancels an idle session", function()
        expect(session:cancel()).toBe(true)
        expect(session:status()).toBe("cancelled")
        expect(events).toEqual({ { kind = "cancelled" } })
        expect(function()
            session:start("INTRO")
        end).toThrow("session has already started")
    end)

    it("cancels a waiting session once", function()
        session:start("INTRO")
        expect(session:cancel()).toBe(true)
        expect(session:cancel()).toBe(false)
        expect(session:advance(1)).toBe(false)
        expect(session:status()).toBe("cancelled")
        expect(events[2]).toEqual({ kind = "cancelled" })
        expect(events).toHaveLength(2)
    end)

    it("keeps sessions independent", function()
        local otherEvents = {}
        local otherContext = { playerId = 99 }
        local other = Cue.new(program, {
            emit = function(event, actualContext)
                expect(actualContext).toBe(otherContext)
                table.insert(otherEvents, event)
            end,
        }, otherContext)

        session:start("INTRO")
        other:start("INTRO")
        session:advance(1)
        session:cancel()
        expect(other:status()).toBe("waiting")
        expect(otherEvents).toEqual({
            { kind = "dialogue", step = 1, actor = "mechanic", text = "First line" },
        })
        expect(other:advance(1)).toBe(true)
    end)

    it("rejects an unsupported program version", function()
        expect(function()
            Cue.new({ version = 99, scenes = {} }, { emit = function() end }, nil)
        end).toThrow("unsupported Cue program version: 99")
    end)

    it("rejects a bus without an emit function", function()
        expect(function()
            (Cue.new :: any)(program, { emit = false }, nil)
        end).toThrow("bus.emit must be a function")
    end)
end)

describe("adapter failures", function()
    it("fails the session and propagates the error", function()
        local session = Cue.new(program, {
            emit = function()
                error("adapter failed")
            end,
        }, nil)

        expect(function()
            session:start("INTRO")
        end).toThrow("adapter failed")
        expect(session:status()).toBe("failed")
        expect(session:advance(1)).toBe(false)
        expect(session:cancel()).toBe(false)
    end)

    it("does not report success when the completion event fails", function()
        local session = Cue.new(program, {
            emit = function(event)
                if event.kind == "completed" then
                    error("completion failed")
                end
            end,
        }, nil)

        session:start("INTRO")
        session:advance(1)
        expect(function()
            session:advance(2)
        end).toThrow("completion failed")
        expect(session:status()).toBe("failed")
    end)

    it("does not affect another session using the same program", function()
        local failed = Cue.new(program, {
            emit = function()
                error("adapter failed")
            end,
        }, nil)
        local otherEvents = {}
        local other = Cue.new(program, {
            emit = function(event)
                table.insert(otherEvents, event)
            end,
        }, nil)

        expect(function()
            failed:start("INTRO")
        end).toThrow("adapter failed")
        other:start("INTRO")
        expect(other:advance(1)).toBe(true)
        expect(otherEvents).toHaveLength(2)
        expect(other:status()).toBe("waiting")
    end)
end)

return nil
