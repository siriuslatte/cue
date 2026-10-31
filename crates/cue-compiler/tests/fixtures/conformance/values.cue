actor NPC "mechanic"

enum Repair {
    Ready({ parts: number, note: string? }),
    Missing,
}

scene INTRO {
    var original = [{ count: 1, labels: ["bolt", "gear"] }]
    var copy = original
    copy[1].count = 9
    copy[1].labels[1] = "spring"
    let before = original[1] ?? { count: 0, labels: [] }
    let after = copy[1] ?? { count: 0, labels: [] }
    [NPC] (string(before.count) .. ":" .. (before.labels[1] ?? ""))
    [NPC] (string(after.count) .. ":" .. (after.labels[1] ?? ""))

    var parts = ["bolt", "gear"]
    for part in parts {
        parts[2] = "spring"
        [NPC] (part)
    }

    let result = Repair.Ready({ parts: len(parts) })
    match result {
        Repair.Ready(repair) {
            if repair.note == none {
                [NPC] ("Ready with " .. string(repair.parts) .. " parts")
            } else {
                [NPC] "Unexpected note"
            }
        }
        Repair.Missing { [NPC] "Missing parts" }
    }
}

scene OPTIONALS {
    let missing: number? = none
    let count: number? = 0
    let flag: bool? = false
    let name: string? = ""
    [NPC] (string(missing ?? 5))
    [NPC] (string(count ?? (1 / 0)))
    [NPC] (string(flag ?? true))
    [NPC] (name ?? "fallback")
    let record: { label: string? } = {}
    [NPC] (record.label ?? "omitted")
    let entries: number?[] = [none, 2, none]
    [NPC] (string(len(entries)))
    [NPC] (string(entries[1] == none))
    [NPC] (string(entries[9] == none))
}

scene EQUALITY {
    [NPC] (string([1, 2] == [1, 2]))
    [NPC] (string([1, 2] == [2, 1]))
    [NPC] (string({ a: [1], b: true } == { b: true, a: [1] }))
    [NPC] (string(Repair.Ready({ parts: 2 }) == Repair.Ready({ parts: 2, note: none })))
    [NPC] (string(Repair.Missing != Repair.Ready({ parts: 2 })))
    let a: { label: string? } = {}
    let b: { label: string? } = { label: none }
    [NPC] (string(a == b))
    let present: {}? = {}
    [NPC] (string(present != none))
}

scene SCOPES {
    var count = 3
    if true {
        let count = count + 2
        [NPC] (string(count))
    }
    [NPC] (string(count))
    let optional: number? = 2
    if optional != none and optional > 0 {
        [NPC] (string(optional + 1))
    }
    if false { [NPC] "wrong" } else if true { [NPC] "else if" }
    {
        let count = 7
        [NPC] (string(count))
    }
    match Repair.Missing {
        Repair.Ready(payload) { [NPC] (string(payload.parts)) }
        Repair.Missing { [NPC] "missing" }
    }
    let empty: number[] = []
    for item in empty {}
}
