use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use mini3d_core::{
    math::{
        fixed::{FixedPoint, RealFixedPoint, I32F16},
        vec::{V2, V2I32F16, V3, V3I32F16, V4},
    },
    platform::event::{AssetImportEntry, ImportAssetEvent},
    renderer::resource::{Material, Mesh, Model, SubMesh, Vertex},
};
use wavefront_obj::obj::{self, Primitive};

fn vec3_from_vertex(v: &obj::Vertex) -> V3I32F16 {
    V3::new(
        I32F16::from_f64(v.x),
        I32F16::from_f64(v.y),
        I32F16::from_f64(v.z),
    )
}

fn vec2_from_tvertex(v: &obj::TVertex) -> V2I32F16 {
    V2::new(I32F16::from_f64(v.u), I32F16::from_f64(v.v))
}

#[derive(Default)]
pub struct ModelImport {
    meshes: Vec<AssetImportEntry<Mesh>>,
    materials: Vec<AssetImportEntry<Material>>,
    models: Vec<AssetImportEntry<Model>>,
}

impl ModelImport {
    pub fn push(self, events: &mut Vec<ImportAssetEvent>) {
        self.meshes.into_iter().for_each(|resource| {
            events.push(ImportAssetEvent::Mesh(resource));
        });
        self.materials.into_iter().for_each(|material| {
            events.push(ImportAssetEvent::Material(material));
        });
        self.models.into_iter().for_each(|model| {
            events.push(ImportAssetEvent::Model(model));
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
        File::open(path)
            .map_err(|err| format!("Failed to load object file: {err}"))?
            .read_to_string(&mut buf)
            .map_err(|err| format!("Failed to read object file: {err}"))?;

        // Parse object file
        let obj = wavefront_obj::obj::parse(buf)
            .map_err(|err| format!("Failed to parse object file: {err}"))?;

        // Prepare model import
        let mut model_import = ModelImport::default();

        // Iterate over objects
        for (object_index, object) in obj.objects.iter().enumerate() {
            // Create object mesh
            let mut mesh = Mesh::default();

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
                            let tangent = V4::W;
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
                            triangle[0].1 = Some(V2::ZERO);
                            triangle[1].1 = Some(V2::X);
                            triangle[2].1 = Some(V2::Y);
                        }

                        // Compute tangent and bitangent
                        let delta_pos1 = triangle[1].0 - triangle[0].0;
                        let delta_pos2 = triangle[2].0 - triangle[0].0;
                        let delta_uv1 = triangle[1].1.unwrap() - triangle[0].1.unwrap();
                        let delta_uv2 = triangle[2].1.unwrap() - triangle[0].1.unwrap();
                        let r = (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x).recip();
                        let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                        let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

                        // Compute final tangent and handedness
                        triangle.iter_mut().for_each(|(_, _, n, t)| {
                            let w = if tangent.cross(bitangent).dot(n.unwrap()) > I32F16::ZERO {
                                I32F16::ONE
                            } else {
                                I32F16::NEG_ONE
                            };
                            *t = V4::from((tangent.reject_from_normalized(n.unwrap()), w));
                        });

                        // Append vertices
                        for (p, u, n, t) in triangle {
                            vertices.push(Vertex {
                                position: p,
                                uv: u.unwrap(),
                                normal: n.unwrap(),
                                tangent: t,
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
                    format!(
                        "{}_{}",
                        path.file_stem().unwrap().to_str().unwrap(),
                        object_index
                    )
                    .to_string()
                }
            });

            // Append object mesh
            model_import.meshes.push(AssetImportEntry {
                data: mesh,
                name: name.into(),
            });
        }

        Ok(model_import)
    }
}
