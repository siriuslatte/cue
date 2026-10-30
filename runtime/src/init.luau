--!strict

export type Dialogue = {
    actor: string,
    text: string,
}

export type Program = {
    version: number,
    scenes: { [string]: { Dialogue } },
}

export type Event =
    {
        kind: "dialogue",
        step: number,
        actor: string,
        text: string,
    }
    | { kind: "completed" }
    | { kind: "cancelled" }

export type Bus = {
    emit: (event: Event, context: unknown) -> (),
}

export type Status = "idle" | "waiting" | "completed" | "cancelled" | "failed"

type State = {
    _program: Program,
    _bus: Bus,
    _context: unknown,
    _scene: { Dialogue },
    _cursor: number,
    _status: Status,
}

local Session = {}
Session.__index = Session

export type Session = typeof(setmetatable({} :: State, Session))

local function emit(self: Session, event: Event)
    local ok, message = pcall(self._bus.emit, event, self._context)
    if not ok then
        self._status = "failed"
        error(message, 0)
    end
end

local function nextLine(self: Session)
    self._cursor += 1
    local line = self._scene[self._cursor]
    if line == nil then
        self._status = "completed"
        emit(self, { kind = "completed" })
        return
    end

    self._status = "waiting"
    emit(self, {
        kind = "dialogue",
        step = self._cursor,
        actor = line.actor,
        text = line.text,
    })
end

function Session.start(self: Session, scene: string)
    assert(self._status == "idle", "session has already started")
    local lines = self._program.scenes[scene]
    assert(lines ~= nil, `unknown scene '{scene}'`)
    self._scene = lines
    nextLine(self)
end

function Session.advance(self: Session, step: number): boolean
    if self._status ~= "waiting" or step ~= self._cursor then
        return false
    end
    nextLine(self)
    return true
end

function Session.cancel(self: Session): boolean
    local status = self._status
    if status ~= "idle" and status ~= "waiting" then
        return false
    end
    self._status = "cancelled"
    emit(self, { kind = "cancelled" })
    return true
end

function Session.status(self: Session): Status
    return self._status
end

local Cue = {}

function Cue.new(program: Program, bus: Bus, context: unknown): Session
    assert(program.version == 1, `unsupported Cue program version: {program.version}`)
    assert(type(bus.emit) == "function", "bus.emit must be a function")
    return setmetatable({
        _program = program,
        _bus = bus,
        _context = context,
        _scene = {},
        _cursor = 0,
        _status = "idle" :: Status,
    }, Session)
end

return table.freeze(Cue)
