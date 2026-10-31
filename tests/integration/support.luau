--!strict

local Lest = require("@lest")
local expect = Lest.expect
local fs = require("@lune/fs")
local luau = require("@lune/luau")
local process = require("@lune/process")
local Cue = require("../../runtime/src")

local binary = process.env.CUE_BIN
    or if process.os == "windows" then "target/debug/cue.exe" else "target/debug/cue"
local programs: { [string]: Cue.Program } = {}
local Support = {}

function Support.compile(name: string): Cue.Program
    if programs[name] then
        return programs[name]
    end
    local source = `crates/cue-compiler/tests/fixtures/{name}.cue`
    local output = `build/tests/{name}.luau`
    local result = process.exec(binary, { "build", source, "-o", output })
    expect(result.stderr).toBe("")
    expect(result.code).toBe(0)
    local program = luau.load(fs.readFile(output), { debugName = output })()
    programs[name] = program
    return program
end

function Support.record(program: Cue.Program, options: Cue.Options?)
    local events: { Cue.Event } = {}
    local session = Cue.new(program, {
        emit = function(event)
            table.insert(events, event)
        end,
    }, nil, options)
    return session, events
end

function Support.source(name: string, source: string): Cue.Program
    local input = `build/tests/{name}.cue`
    local output = `build/tests/{name}.luau`
    if not fs.isDir("build/tests") then
        fs.writeDir("build/tests")
    end
    fs.writeFile(input, source)
    local result = process.exec(binary, { "build", input, "-o", output })
    expect(result.stderr).toBe("")
    expect(result.code).toBe(0)
    return luau.load(fs.readFile(output), { debugName = output })()
end

return Support
