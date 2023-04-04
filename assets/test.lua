-- Test system
local e = QuerySingle(PLAYER)
local position = GetTransformPosition(e)
local health = GetFloat(e, HEALTH, "value")
local score = GetInt(e, PLAYER, "score")

local items = ListKeys(e, "player", "items")
local length = GetLength(e, "player", "items")
local item = GetFloat(e, "player", "items.0")

DefineComponent("player", {
    { "score", INT, 0 },
    { "items", LIST, {
        { "health", FLOAT, 100 },
        { "ammo", INT, 10 },
    }},
})