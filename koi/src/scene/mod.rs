use std::{path::Path, str::FromStr};

use gltf::{
    self,
    mesh::util::{ReadColors, ReadIndices, ReadTexCoords},
};
use koi_gpu::Vertex;
use spirv_std::glam::{Vec3, Vec4, Vec4Swizzles};

#[derive(Default, Clone, Copy)]
pub struct Surface {
    pub start_index: u32,
    pub count: u32,
}

#[derive(Default)]
pub struct Mesh {
    pub name: String,
    pub indices: Vec<u32>,
    pub vertices: Vec<Vertex>,
    pub surfaces: Vec<Surface>,
}

#[derive(Default)]
pub struct Scene {
    pub meshes: Vec<Mesh>,
}

pub fn load(path: &Path) -> Scene {
    let (gltf, buffers, _images) = gltf::import(path).expect("koi::scene - failed to load Scene");

    let mut scene = Scene::default();

    let mut indices = vec![];
    let mut vertices = vec![];

    for gltf_mesh in gltf.meshes() {
        let mut mesh = Mesh::default();

        mesh.name = String::from_str(gltf_mesh.name().unwrap_or("")).unwrap();

        indices.clear();
        vertices.clear();

        for primitive in gltf_mesh.primitives() {
            let mut surface = Surface::default();

            let start_index = indices.len();

            surface.start_index = start_index as u32;

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            if let Some(idxs) = reader.read_indices() {
                match idxs {
                    ReadIndices::U8(iter) => {
                        for index in iter {
                            indices.push(index as u32);
                        }
                    }
                    ReadIndices::U16(iter) => {
                        for index in iter {
                            indices.push(index as u32);
                        }
                    }
                    ReadIndices::U32(iter) => {
                        for index in iter {
                            indices.push(index);
                        }
                    }
                }
            }

            if let Some(iter) = reader.read_positions() {
                for vertex_position in iter {
                    let mut vertex = Vertex::default();
                    vertex.position_uv_x = Vec4::from((Vec3::from_array(vertex_position), 0.0));
                    vertices.push(vertex);
                }
            }

            if let Some(iter) = reader.read_normals() {
                for (i, normal) in iter.enumerate() {
                    vertices[start_index + i].normal_uv_y =
                        Vec4::from((Vec3::from_array(normal), 0.0));
                }
            }

            if let Some(coords) = reader.read_tex_coords(0) {
                match coords {
                    ReadTexCoords::U8(iter) => {
                        for (i, uv) in iter.enumerate() {
                            vertices[start_index + i].position_uv_x.w = uv[0] as f32;
                            vertices[start_index + i].normal_uv_y.w = uv[1] as f32;
                        }
                    }
                    ReadTexCoords::U16(iter) => {
                        for (i, uv) in iter.enumerate() {
                            vertices[start_index + i].position_uv_x.w = uv[0] as f32;
                            vertices[start_index + i].normal_uv_y.w = uv[1] as f32;
                        }
                    }
                    ReadTexCoords::F32(iter) => {
                        for (i, uv) in iter.enumerate() {
                            vertices[start_index + i].position_uv_x.w = uv[0];
                            vertices[start_index + i].normal_uv_y.w = uv[1];
                        }
                    }
                }
            }

            if let Some(colors) = reader.read_colors(0) {
                match colors {
                    ReadColors::RgbU8(iter) => {
                        for (i, color) in iter.enumerate() {
                            vertices[start_index + i].color =
                                Vec4::new(color[0] as f32, color[1] as f32, color[2] as f32, 1.0);
                        }
                    }
                    ReadColors::RgbU16(iter) => {
                        for (i, color) in iter.enumerate() {
                            vertices[start_index + i].color =
                                Vec4::new(color[0] as f32, color[1] as f32, color[2] as f32, 1.0);
                        }
                    }
                    ReadColors::RgbF32(iter) => {
                        for (i, color) in iter.enumerate() {
                            vertices[start_index + i].color =
                                Vec4::from((Vec3::from_array(color), 1.0));
                        }
                    }
                    ReadColors::RgbaU8(iter) => {
                        for (i, color) in iter.enumerate() {
                            vertices[start_index + i].color = Vec4::new(
                                color[0] as f32,
                                color[1] as f32,
                                color[2] as f32,
                                color[3] as f32,
                            );
                        }
                    }
                    ReadColors::RgbaU16(iter) => {
                        for (i, color) in iter.enumerate() {
                            vertices[start_index + i].color = Vec4::new(
                                color[0] as f32,
                                color[1] as f32,
                                color[2] as f32,
                                color[3] as f32,
                            );
                        }
                    }
                    ReadColors::RgbaF32(iter) => {
                        for (i, color) in iter.enumerate() {
                            vertices[start_index + i].color = Vec4::from_array(color);
                        }
                    }
                }
            }

            surface.count = (indices.len() - start_index) as u32;
            mesh.surfaces.push(surface);
        }

        // display vertex normals
        let override_colors = true;
        if override_colors {
            for vertex in &mut vertices {
                vertex.color = Vec4::from((vertex.normal_uv_y.xyz(), 1.0));
            }
        }

        mesh.indices = indices.clone();
        mesh.vertices = vertices.clone();
        scene.meshes.push(mesh);
    }
    scene
}
