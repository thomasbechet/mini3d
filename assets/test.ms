import 'myscript' as myscript

function add(a: int, b: float): int
    return a + int(b)
end

-- Change the player mode if required
function check_player_mode(player: entity): bool
    if player.info.score > 0 then
        player.info.mode = 2
        procedure.invoke('player_mode_changed')
        return true
    end
    return false
end

if player.transform.position.y > 100.0 then
    let player = world.query('player')
    if input.action('move').pressed then
        let v = vec3(0)
    elif player.life.value != 0 then
        let font = asset.get('font', 'default') -- return obj (as handle)
        gfx.print(0, 1, 'Hello World !', font)
    end
    check_player_mode(player)
end

let ui = world.query('main_ui')
for event in ui.events do

end

-- reserved keywords
if
nil
elif
else
then
end
let
do
while
return
true
false
and
not
import
function
-- reserved identifiers
log
gfx
asset
network
disk
registry
input
world
procedure
scheduler
time
-- primitives
uid
obj
dict
bool
entity
string
int
float
vec2
vec3
vec4
mat
quat