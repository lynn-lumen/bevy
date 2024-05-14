use std::f32::consts::FRAC_PI_2;

use bevy_ecs::system::IntoSystem;
use bevy_math::{
    primitives::{Extrusion, Primitive2d, Rectangle},
    Quat, Vec2, Vec3,
};
use wgpu::VertexAttribute;

use crate::mesh::{Indices, Mesh, MeshVertexAttribute, VertexAttributeValues};

use super::Meshable;

pub trait MeshPrimitive2d: Primitive2d {
    fn mesh_data(&self) -> (Mesh, Vec<Indices>);
}

impl MeshPrimitive2d for Rectangle {
    fn mesh_data(&self) -> (Mesh, Vec<Indices>) {
        (self.mesh(), vec![Indices::U32(vec![0, 1, 2, 3, 0])])
    }
}

pub struct ExtrusionMeshBuilder<T: MeshPrimitive2d> {
    pub extrusion: Extrusion<T>,
}

impl<T: MeshPrimitive2d> ExtrusionMeshBuilder<T> {
    pub fn build(&self) -> Mesh {
        let half_depth = self.extrusion.depth / 2.;

        let (mut surface1, perimeter) = self.extrusion.base_shape.mesh_data();

        let mantle = {
            let Some(VertexAttributeValues::Float32x3(positions)) =
                surface1.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("Failed to get the vertex positions");
            };
            let hd_vec = Vec3::Z * half_depth;

            let max_uv_index = perimeter.iter().fold(0, |acc, indices| acc + indices.len());
            let vert_count = 4 * (max_uv_index - perimeter.len());
            let max_uv_index = max_uv_index as f32;
            let mut verts = Vec::with_capacity(vert_count);
            let mut normals = Vec::with_capacity(vert_count);
            let mut uvs = Vec::with_capacity(vert_count);
            let mut mesh_indices = Vec::with_capacity(vert_count * 3 / 2);

            let mut uv_index = 0.;
            for indices in perimeter.into_iter() {
                for ij in indices.iter().collect::<Vec<usize>>().windows(2) {
                    let base_index = verts.len() as u32;
                    mesh_indices.extend_from_slice(&[
                        base_index,
                        base_index + 2,
                        base_index + 1,
                        base_index + 1,
                        base_index + 2,
                        base_index + 3,
                    ]);

                    let p1 = Vec3::from_array(positions[ij[0] as usize]);
                    let p2 = Vec3::from_array(positions[ij[1] as usize]);
                    let normal =
                        Quat::from_scaled_axis((p2 - p1).normalize() * FRAC_PI_2) * Vec3::Z;

                    verts.extend_from_slice(&[p1 + hd_vec, p2 + hd_vec, p1 - hd_vec, p2 - hd_vec]);
                    normals.extend_from_slice(&[normal, normal, normal, normal]);
                    uvs.extend_from_slice(&[
                        Vec2::new(uv_index / max_uv_index, 0.5),
                        Vec2::new((uv_index + 1.) / max_uv_index, 0.5),
                        Vec2::new(uv_index / max_uv_index, 0.),
                        Vec2::new((uv_index + 1.) / max_uv_index, 0.),
                    ]);
                    uv_index += 1.;
                }
            }

            Mesh::new(wgpu::PrimitiveTopology::TriangleList, surface1.asset_usage)
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verts)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
                .with_inserted_indices(Indices::U32(mesh_indices))
        };

        let surface2 = {
            let mut mesh = surface1.clone();

            // Invert winding order
            let Some(indices) = mesh.indices_mut() else {
                panic!("Failed to get the indices");
            };
            match indices {
                Indices::U16(indices) => {
                    indices.chunks_exact_mut(3).for_each(|abc| {
                        let temp = abc[1];
                        abc[1] = abc[2];
                        abc[2] = temp;
                    });
                }
                Indices::U32(indices) => {
                    indices.chunks_exact_mut(3).for_each(|abc| {
                        let temp = abc[1];
                        abc[1] = abc[2];
                        abc[2] = temp;
                    });
                }
            }

            mesh.translated_by(Vec3::new(0., 0., -half_depth))
        };
        surface1 = surface1.translated_by(Vec3::new(0., 0., half_depth));

        surface1.merge(surface2);
        surface1.merge(mantle);
        surface1
    }
}

impl<T: MeshPrimitive2d> Meshable for Extrusion<T> {
    type Output = ExtrusionMeshBuilder<T>;

    fn mesh(&self) -> Self::Output {
        ExtrusionMeshBuilder {
            extrusion: self.clone(),
        }
    }
}

impl<T: MeshPrimitive2d> From<Extrusion<T>> for Mesh {
    fn from(extrusion: Extrusion<T>) -> Self {
        extrusion.mesh().build()
    }
}

impl<T: MeshPrimitive2d> From<ExtrusionMeshBuilder<T>> for Mesh {
    fn from(extrusion: ExtrusionMeshBuilder<T>) -> Self {
        extrusion.build()
    }
}
