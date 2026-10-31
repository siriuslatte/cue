actor NPC "npc"

scene A01 {
    [NPC] (string(2 * 3 + 4))
}

scene A02 {
    [NPC] (string(2 + 3 * 4))
}

scene A03 {
    [NPC] (string(10 - 3 - 2))
}

scene A04 {
    [NPC] (string(20 / 2 / 2))
}

scene A05 {
    [NPC] (string(2 ^ 3 ^ 2))
}

scene A06 {
    [NPC] (string(-2 ^ 2))
}

scene A07 {
    [NPC] (string((-2) ^ 2))
}

scene A08 {
    [NPC] (string(2 ^ -2))
}

scene A09 {
    [NPC] (string(-5 % 3))
}

scene A14 {
    [NPC] (string(0.1 + 0.2 == 0.3))
}

scene B01 {
    [NPC] (string(false or true and false))
}

scene B02 {
    [NPC] (string(false and (1 / 0 > 0)))
}

scene B03 {
    [NPC] (string(true or (1 / 0 > 0)))
}

scene B09 {
    [NPC] (string(1 < 2 and 2 < 3))
}

scene B11 {
    [NPC] (string("Hello, " .. "mechanic"))
}

scene P01 {
    [NPC] (string(2 ^ 2 ^ -1))
}

scene P02 {
    [NPC] (string(0 - -2))
}

scene P04 {
    [NPC] (string(10 >= 10 and 9 <= 10 and 10 > 9))
}

scene P05 {
    [NPC] (string("a" .. "b" .. "c"))
}

scene P06 {
    [NPC] (string(not false == true))
}

scene P07 {
    [NPC] (string([1][0] == none))
}

scene P08 {
    [NPC] (string([1][-1] == none))
}

scene A10 {
    [NPC] "before"
    let result = 1 / 0
    [NPC] "after"
}

scene A11 {
    [NPC] "before"
    let result = 1 % 0
    [NPC] "after"
}

scene A12 {
    [NPC] "before"
    let result = 1e308 * 1e308
    [NPC] "after"
}

scene A15 {
    [NPC] "before"
    let result = (-1) ^ 0.5
    [NPC] "after"
}

scene I01 {
    [NPC] "before"
    let result = [1][1.5]
    [NPC] "after"
}

scene WRITE_BOUNDS {
    var values = [1]
    values[2] = 1 / 0
    [NPC] "after"
}
scene WRITE_FRACTION {
    var values = [1]
    values[1.5] = 2
}
scene FIELD_ORDER {
    let result = { first: 1 / 0, second: [1][1.5] }
}
scene LOOP {
    for n in [1, 2, 3] {
        [NPC] (string(n))
    }
}
scene BUDGET {
    for outer in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] {
        for inner in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] {}
    }
}
