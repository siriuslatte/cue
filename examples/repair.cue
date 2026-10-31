actor MECHANIC "mechanic"

enum Repair {
    Ready({ parts: number }),
    Missing,
}

scene INTRO {
    let parts = ["bolt", "gear", "spring"]
    for part in parts {
        [MECHANIC] ("Found a " .. part)
    }

    let repair = Repair.Ready({ parts: len(parts) })
    match repair {
        Repair.Ready(details) {
            [MECHANIC] ("Ready with " .. string(details.parts) .. " parts.")
        }
        Repair.Missing {
            [MECHANIC] "Come back when you've found the parts."
        }
    }
}
