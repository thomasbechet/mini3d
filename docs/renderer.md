# Renderer Specification

## Goals

The renderer must reflect a retro 3D style.

## Retro Effects

- Dittering
- Nearest sampling
- Low fixed resolution
- Low poly models
- Low resolution textures (max 512x512)
- Per vertex lighting

## Compatible effects

- Reflection
- Global illumination
- Volumetric rendering
- Transparency
- Shadows

## Global architecture

The renderer is build upon two main APIs: resources and commands.

Commands are generated in a stateless way and are bind to a specific
renderpass.

Resources:
- Texture
- Mesh
- Material
- Transform
- Camera

RenderPasses:
- CanvasPass
  
GraphicsPass:
draw_mesh(mesh, material, transform)
draw_mesh_skinned(mesh, material, transform, bones)
draw_particles(particles, material, transform)
draw_billboard(billboard, material, transform)

ShadowPass:
- ShadowMap

      
Graphics Materials:
- TransparentMaterial
- OpaqueMaterial
- ReflectiveMaterial
- VolumeMaterial