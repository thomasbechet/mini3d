from 'utils' import *
from 'scene' import query, create, add

export const TEST: int = 2

let transform: @transform = scene.create('transform', { position={ x=1, y=2, z=3 } })

let player: entity = scene.query('player')
let transform: @transform = player.transform
transform.position = vec3(0.0)
