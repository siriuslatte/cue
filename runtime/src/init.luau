--!strict

-- Compiled values contain only primitives and frozen, acyclic tables. Updates
-- rebuild the path to the root, so bindings can share everything else safely.
local Values = {}
Values.none = table.freeze({})

function Values.number(value: number, location: string): number
    if value ~= value or math.abs(value) == math.huge then
        error(`{location}: arithmetic result must be finite`, 0)
    end
    return value
end

function Values.divisor(value: number, location: string)
    if value == 0 then
        error(`{location}: division or remainder by zero`, 0)
    end
end

-- The compiler checks shape and element types. Keep dynamic table operations
-- here rather than introducing a second type checker into the runtime.
function Values.index(list: { any }, index: number, writing: boolean, location: string): any
    if index % 1 ~= 0 then
        error(`{location}: list index must be an integer`, 0)
    end
    if index < 1 or index > #list then
        if writing then
            error(`{location}: list assignment index is out of bounds`, 0)
        end
        return Values.none
    end
    return list[index]
end

function Values.set(container: { [any]: any }, key: any, value: any): { [any]: any }
    local copy = table.clone(container)
    copy[key] = value
    return table.freeze(copy)
end

function Values.equal(left: any, right: any, location: string, tick: (string) -> ()): boolean
    if left == right then
        return true
    end
    local pending = { left, right }
    local seen: { [any]: { [any]: boolean } } = {}
    while #pending > 0 do
        local b = table.remove(pending)
        local a = table.remove(pending)
        if type(a) ~= "table" or type(b) ~= "table" or a == Values.none or b == Values.none then
            return false
        end
        if seen[a] and seen[a][b] then
            continue
        end
        tick(location)
        if not seen[a] then
            seen[a] = {}
        end
        seen[a][b] = true
        for key, value in a do
            if b[key] == nil then
                return false
            end
            if value ~= b[key] then
                table.insert(pending, value)
                table.insert(pending, b[key])
            end
        end
        for key in b do
            if a[key] == nil then
                return false
            end
        end
    end
    return true
end

table.freeze(Values)

export type Execution = {
    read none: {},
    read number: (value: number, location: string) -> number,
    read divisor: (value: number, location: string) -> (),
    read index: (list: { any }, index: number, writing: boolean, location: string) -> any,
    read set: (container: { [any]: any }, key: any, value: any) -> { [any]: any },
    read equal: (left: any, right: any, location: string) -> boolean,
    read tick: (location: string) -> (),
    read dialogue: (actor: string, text: string) -> (),
}

export type Program = {
    version: number,
    scenes: { [string]: (execution: Execution) -> () },
}

export type Event =
    { kind: "dialogue", step: number, actor: string, text: string }
    | { kind: "completed" }
    | { kind: "cancelled" }
export type Bus = { emit: (event: Event, context: unknown) -> () }
export type Status = "idle" | "running" | "waiting" | "completed" | "cancelled" | "failed"
export type Options = { maxSteps: number? }

type State = {
    _program: Program,
    _bus: Bus,
    _context: unknown,
    _thread: thread?,
    _cursor: number,
    _remaining: number,
    _maxSteps: number,
    _status: Status,
}
local Session = {}
Session.__index = Session
export type Session = typeof(setmetatable({} :: State, Session))

local function release(self: Session)
    local active = self._thread
    self._thread = nil
    if active ~= nil then
        coroutine.close(active)
    end
end

local function fail(self: Session, message: unknown): never
    self._status = "failed"
    release(self)
    error(message, 0)
end

local function emit(self: Session, event: Event)
    local ok, message = pcall(self._bus.emit, event, self._context)
    if not ok then
        fail(self, message)
    end
end

local function resume(self: Session)
    local active = self._thread
    assert(active ~= nil, "session has no active scene")
    self._status = "running"
    self._remaining = self._maxSteps
    local ok, line = coroutine.resume(active)
    if not ok then
        fail(self, line)
    end
    if coroutine.status(active) == "dead" then
        release(self)
        self._status = "completed"
        emit(self, { kind = "completed" })
    else
        self._cursor += 1
        self._status = "waiting"
        emit(self, { kind = "dialogue", step = self._cursor, actor = line.actor, text = line.text })
    end
end

function Session.start(self: Session, scene: string)
    assert(self._status == "idle", "session has already started")
    local entry = self._program.scenes[scene]
    assert(entry ~= nil, `unknown scene '{scene}'`)
    local function tick(location: string)
        self._remaining -= 1
        if self._remaining < 0 then
            error(`{location}: execution budget exceeded`, 0)
        end
    end
    local execution: Execution = {
        none = Values.none,
        number = Values.number,
        divisor = Values.divisor,
        index = Values.index,
        set = Values.set,
        equal = function(left, right, location)
            return Values.equal(left, right, location, tick)
        end,
        tick = tick,
        dialogue = function(actor, text)
            coroutine.yield({ actor = actor, text = text })
        end,
    }
    self._thread = coroutine.create(function()
        entry(table.freeze(execution))
    end)
    resume(self)
end

function Session.advance(self: Session, step: number): boolean
    if self._status ~= "waiting" or step ~= self._cursor then
        return false
    end
    resume(self)
    return true
end

function Session.cancel(self: Session): boolean
    local status = self._status
    if status ~= "idle" and status ~= "waiting" then
        return false
    end
    self._status = "cancelled"
    release(self)
    emit(self, { kind = "cancelled" })
    return true
end

function Session.status(self: Session): Status
    return self._status
end

local Cue = {}
function Cue.new(program: Program, bus: Bus, context: unknown, options: Options?): Session
    assert(program.version == 2, `unsupported Cue program version: {program.version}`)
    assert(type(bus.emit) == "function", "bus.emit must be a function")
    local maxSteps = if options and options.maxSteps then options.maxSteps else 10000
    assert(
        maxSteps > 0 and maxSteps < math.huge and maxSteps % 1 == 0,
        "maxSteps must be a positive finite integer"
    )
    return setmetatable({
        _program = program,
        _bus = bus,
        _context = context,
        _thread = nil :: thread?,
        _cursor = 0,
        _remaining = 0,
        _maxSteps = maxSteps,
        _status = "idle" :: Status,
    }, Session)
end
return table.freeze(Cue)
