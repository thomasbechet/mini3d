use std::{path::{Path, PathBuf}, fs::File, io::Read};

use mini3d::{event::{asset::{ImportAssetEvent, AssetImportEntry}, AppEvents}, asset::{mesh::{Mesh, Vertex, SubMesh}, material::Material, model::Model}, glam::{Vec3, Vec2, Vec4}};
use wavefront_obj::obj::{Primitive, self};

fn vec3_from_vertex(v: &obj::Vertex) -> Vec3 {
    Vec3::new(v.x as f32, v.y as f32, v.z as f32)
}

fn vec2_from_tvertex(v: &obj::TVertex) -> Vec2 {
    Vec2::new(v.u as f32, v.v as f32)
}

#[derive(Default)]
pub struct ModelImport {
    meshes: Vec<AssetImportEntry<Mesh>>,
    materials: Vec<AssetImportEntry<Material>>,
    models: Vec<AssetImportEntry<Model>>,
}

impl ModelImport {
    pub fn push(self, events: &mut AppEvents) {
        self.meshes.into_iter().for_each(|asset| {
            events.asset.push(ImportAssetEvent::Mesh(asset));
        });
        self.materials.into_iter().for_each(|material| {
            events.asset.push(ImportAssetEvent::Material(material));
        });
        self.models.into_iter().for_each(|model| {
            events.asset.push(ImportAssetEvent::Model(model));
        });
    }
}

#[derive(Default)]
pub struct ModelImporter {
    name: Option<String>,
    path: Option<PathBuf>,
    flat_normals: bool,
}

impl ModelImporter {

    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_obj(&mut self, path: &Path) -> &mut Self {
        self.path = Some(path.into());
        self
    }

    pub fn with_name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn with_flat_normals(&mut self, flat: bool) -> &mut Self {
        self.flat_normals = flat;
        self
    }

    pub fn import(&self) -> Result<ModelImport, String> {

        // Ensure a path is provided
        let path = self.path.as_ref().ok_or("No source provided.")?;
        
        // Load object file
        let mut buf = String::new();
        File::open(path).map_err(|err| format!("Failed to load object file: {err}"))?
            .read_to_string(&mut buf).map_err(|err| format!("Failed to read object file: {err}"))?;

        // Parse object file
        let obj = wavefront_obj::obj::parse(buf)
            .map_err(|err| format!("Failed to parse object file: {err}"))?;

        // Prepare model import
        let mut model_import = ModelImport::default();

        // Iterate over objects
        for (object_index, object) in obj.objects.iter().enumerate() {

            // Create object mesh
            let mut mesh = Mesh {
                submeshes: Default::default(),
            };

            // Iterate over geometries
            for geometry in &object.geometry {

                // Build vertices list
                let mut vertices: Vec<Vertex> = Vec::new();
                for shape in &geometry.shapes {
                    if let Primitive::Triangle(v0, v1, v2) = shape.primitive {
                       
                        // Extract triangle vertices
                        let mut triangle = [v0, v1, v2].map(|v| {
                            let (v_index, t_index, n_index) = v;
                            let position = vec3_from_vertex(&object.vertices[v_index]);
                            let uv = t_index.map(|i| vec2_from_tvertex(&object.tex_vertices[i]));
                            let normal = n_index.map(|i| vec3_from_vertex(&object.normals[i]));
                            let tangent = Vec4::new(0.0, 0.0, 0.0, 1.0);
                            (position, uv, normal, tangent)
                        });

                        // Compute flat normal if missing
                        let missing_normal = triangle.iter().any(|(_, _, n, _)| n.is_none()); 
                        if missing_normal || self.flat_normals {

                            // Compute triangle normal
                            let a = triangle[1].0 - triangle[0].0;
                            let b = triangle[2].0 - triangle[0].0;
                            let normal = a.cross(b).normalize();

                            // Update triangle normals
                            triangle.iter_mut().for_each(|(_, _, n, _)| {
                                *n = Some(normal);
                            });
                        }
                        
                        // Compute uvs if missing
                        let missing_uv = triangle.iter().any(|(_, uv, _, _)| uv.is_none());
                        if missing_uv {
                            triangle[0].1 = Some(Vec2::ZERO);
                            triangle[1].1 = Some(Vec2::X);
                            triangle[2].1 = Some(Vec2::Y);
                        }

                        // Compute tangent and bitangent
                        let delta_pos1 = triangle[1].0 - triangle[0].0;
                        let delta_pos2 = triangle[2].0 - triangle[0].0;
                        let delta_uv1 = triangle[1].1.unwrap() - triangle[0].1.unwrap();
                        let delta_uv2 = triangle[2].1.unwrap() - triangle[0].1.unwrap();
                        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                        let tangent = (delta_pos1 * delta_uv2.y   - delta_pos2 * delta_uv1.y) * r;
                        let bitangent = (delta_pos2 * delta_uv1.x   - delta_pos1 * delta_uv2.x) * r;
                        
                        // Compute final tangent and handedness
                        triangle.iter_mut().for_each(|(_, _, n, t)| {
                            let w = if tangent.cross(bitangent).dot(n.unwrap()) > 0.0 { 1.0 } else { -1.0 };
                            *t = Vec4::from((tangent.reject_from_normalized(n.unwrap()), w));
                        });

                        // Append vertices
                        for (p, u, n, t) in triangle {
                            vertices.push(Vertex { 
                                position: p, 
                                uv: u.unwrap(), 
                                normal: n.unwrap(), 
                                tangent: t 
                            });
                        }
                    }
                }
            
                // Append submesh
                mesh.submeshes.push(SubMesh { vertices });
            }

            // Find object name
            let name = self.name.clone().unwrap_or({
                if !object.name.is_empty() {
                    object.name.clone()
                } else {
                    format!("{}_{}", path.file_stem().unwrap().to_str().unwrap(), object_index).to_string()
                }
            });

            // Append object mesh
            model_import.meshes.push(AssetImportEntry { data: mesh, name });
        }

        Ok(model_import)
    }
}