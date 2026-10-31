actor NPC "mechanic"
scene INTRO {
    var parts = 1
    parts = parts + 2
    if parts >= 3 {
        [NPC] ("Found " .. string(parts) .. " parts")
    } else {
        [NPC] "Keep looking."
    }
}
