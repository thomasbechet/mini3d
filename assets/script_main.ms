from 'utils' import *

export const TEST: int = 2

let player = scene.find('player')
for e in scene.query(@player) do
    let t: @transform = e.transform
end
let transform = player.transform
let texture = asset.get(@texture, 'alfred')

print('{}')

for e in scene.query()