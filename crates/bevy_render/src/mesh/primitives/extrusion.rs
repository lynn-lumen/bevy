use bevy_math::{
    primitives::{Extrusion, Primitive2d},
    Vec3,
};

use crate::mesh::{Indices, Mesh, VertexAttributeValues};

use super::Meshable;

pub trait ExtrudableMesh: Meshable {
    fn perimeter_indices(&self) -> Vec<Indices>;
}

impl<T: Clone + Primitive2d + ExtrudableMesh<Output = Mesh>> Meshable for Extrusion<T> {
    type Output = ExtrusionMeshBuilder<T>;

    fn mesh(&self) -> Self::Output {
        ExtrusionMeshBuilder {
            meshable: self.clone(),
            segments: 1,
        }
    }
}

pub struct ExtrusionMeshBuilder<T: Primitive2d + ExtrudableMesh> {
    meshable: Extrusion<T>,
    segments: usize,
}
impl<T: Primitive2d + ExtrudableMesh<Output = Mesh>> ExtrusionMeshBuilder<T> {
    pub const fn new(extrusion: Extrusion<T>, segments: usize) -> Self {
        Self {
            meshable: extrusion,
            segments,
        }
    }

    pub fn segments(mut self, segments: usize) -> Self {
        self.segments = segments;
        self
    }

    pub fn build(&self) -> Mesh {
        let mut cap = self.meshable.base_shape.mesh().translated_by(Vec3::new(
            0.,
            0.,
            self.meshable.half_depth,
        ));
        if let Some(VertexAttributeValues::Float32x2(uvs)) = cap.attribute_mut(Mesh::ATTRIBUTE_UV_0)
        {
            for uv in uvs {
                *uv = uv.map(|coord| coord * 0.5);
            }
        }

        let opposite_cap = {
            let topology = cap.primitive_topology();
            let mut cap = cap.clone().scaled_by(Vec3::new(1., 1., -1.));
            if let Some(VertexAttributeValues::Float32x2(uvs)) =
                cap.attribute_mut(Mesh::ATTRIBUTE_UV_0)
            {
                for uv in uvs {
                    *uv = [uv[0] + 0.5, uv[1]]
                }
            }

            if let Some(indices) = cap.indices_mut() {
                match topology {
                    wgpu::PrimitiveTopology::TriangleList => match indices {
                        Indices::U16(indices) => {
                            indices.chunks_exact_mut(3).for_each(|arr| arr.swap(1, 0))
                        }
                        Indices::U32(indices) => {
                            indices.chunks_exact_mut(3).for_each(|arr| arr.swap(1, 0))
                        }
                    },
                    _ => {
                        panic!("Meshes used with Extrusions must have a primitive topology of either `PrimitiveTopology::TriangleList`");
                    }
                };
            }
            cap
        };

        let barrel_skin = {
            let Some(VertexAttributeValues::Float32x3(cap_verts)) =
                cap.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("The cap mesh did not have a vertex attribute");
            };

            let perimeter = self.meshable.base_shape.perimeter_indices();
            let vert_count = perimeter
                .iter()
                .fold(0, |acc, indices| acc + indices.len() - 1);
            let mut positions = Vec::with_capacity(vert_count * 4);
            let mut normals = Vec::with_capacity(vert_count * 4);
            let mut indices = Vec::with_capacity(vert_count * 6);
            let mut uvs = Vec::with_capacity(vert_count * 4);

            let uv_delta = 1. / (vert_count + perimeter.len() - 1) as f32;
            let mut uv_x = 0.;
            let mut index = 0;
            for skin in perimeter {
                let skin_indices = match skin {
                    Indices::U16(ind) => ind.into_iter().map(|i| i as u32).collect(),
                    Indices::U32(ind) => ind,
                };
                for i in 0..(skin_indices.len() - 1) {
                    let a = cap_verts[skin_indices[i] as usize];
                    let b = cap_verts[skin_indices[i + 1] as usize];

                    positions.push(a);
                    positions.push(b);
                    positions.push([a[0], a[1], -a[2]]);
                    positions.push([b[0], b[1], -b[2]]);

                    uvs.append(&mut vec![
                        [uv_x, 0.5],
                        [uv_x + uv_delta, 0.5],
                        [uv_x, 1.],
                        [uv_x + uv_delta, 1.],
                    ]);

                    let n = Vec3::from_array([b[1] - a[1], a[0] - b[0], 0.])
                        .normalize_or_zero()
                        .to_array();
                    normals.append(&mut vec![n; 4]);

                    indices.append(&mut vec![
                        index,
                        index + 2,
                        index + 1,
                        index + 1,
                        index + 2,
                        index + 3,
                    ]);

                    index += 4;
                    uv_x += uv_delta;
                }

                uv_x += uv_delta;
            }

            Mesh::new(cap.primitive_topology(), cap.asset_usage)
                .with_inserted_indices(Indices::U32(indices))
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        };

        cap.merge(opposite_cap);
        cap.merge(barrel_skin);
        cap
    }
}
