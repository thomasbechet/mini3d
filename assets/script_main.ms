from 'utils' import *

export const TEST: int = 2

let player = scene.find('player')
for e in scene.query(@player) do
    let t: @transform = e.transform
end
let transform = player.transform
let texture = resource.get(@texture, 'alfred')

print('{}')

for e in scene.query()

let inventory = player.inventory

for e in finalized()