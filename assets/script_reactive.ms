-- this script execute when the transform component is added

for e in scene.added() do
    e.transform.position = vec3(0, 0, 0)
end