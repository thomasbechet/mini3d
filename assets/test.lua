player = QueryFirst(player, transform, stats)
if player.stats.health <= 0 then
    player.stats.health = 100
    player.transform.position = {0, 0, 0}
end