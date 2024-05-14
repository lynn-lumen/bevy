use std::f32::consts::{FRAC_PI_2, PI};

use bevy_color::Color;
use bevy_math::{
    primitives::{
        BoxedPolygon, BoxedPolyline2d, Capsule2d, Extrusion, Polygon, Polyline2d, Primitive2d,
        Rectangle, Triangle2d,
    },
    Quat, Vec2, Vec3,
};

use crate::{
    prelude::{GizmoConfigGroup, Gizmos},
    primitives::dim3::GizmoPrimitive3d,
};

pub(crate) enum GizmoPart2d {
    Line {
        a: Vec2,
        b: Vec2,
    },
    Arc {
        center: Vec2,
        radius: f32,
        start: f32,
        angle: f32,
    },
    LineStrip {
        points: Vec<Vec2>,
    },
}
pub(crate) trait GizmoPolygon2d: Primitive2d {
    fn get_data(&self) -> Vec<GizmoPart2d>;
}

impl GizmoPolygon2d for Capsule2d {
    fn get_data(&self) -> Vec<GizmoPart2d> {
        vec![
            GizmoPart2d::Line {
                a: Vec2::new(self.radius, self.half_length),
                b: Vec2::new(self.radius, -self.half_length),
            },
            GizmoPart2d::Arc {
                center: Vec2::new(0., -self.half_length),
                radius: self.radius,
                start: 0.,
                angle: -PI,
            },
            GizmoPart2d::Line {
                a: Vec2::new(-self.radius, -self.half_length),
                b: Vec2::new(-self.radius, self.half_length),
            },
            GizmoPart2d::Arc {
                center: Vec2::new(0., self.half_length),
                radius: self.radius,
                start: PI,
                angle: -PI,
            },
        ]
    }
}

impl GizmoPolygon2d for Rectangle {
    fn get_data(&self) -> Vec<GizmoPart2d> {
        vec![GizmoPart2d::LineStrip {
            points: vec![
                self.half_size,
                Vec2::new(self.half_size.x, -self.half_size.y),
                -self.half_size,
                Vec2::new(-self.half_size.x, self.half_size.y),
                self.half_size,
            ],
        }]
    }
}
impl GizmoPolygon2d for Triangle2d {
    fn get_data(&self) -> Vec<GizmoPart2d> {
        vec![GizmoPart2d::LineStrip {
            points: vec![
                self.vertices[0],
                self.vertices[1],
                self.vertices[2],
                self.vertices[0],
            ],
        }]
    }
}
impl<const N: usize> GizmoPolygon2d for Polyline2d<N> {
    fn get_data(&self) -> Vec<GizmoPart2d> {
        vec![GizmoPart2d::LineStrip {
            points: self.vertices.into(),
        }]
    }
}
impl GizmoPolygon2d for BoxedPolyline2d {
    fn get_data(&self) -> Vec<GizmoPart2d> {
        vec![GizmoPart2d::LineStrip {
            points: self.vertices.clone().into(),
        }]
    }
}
impl<const N: usize> GizmoPolygon2d for Polygon<N> {
    fn get_data(&self) -> Vec<GizmoPart2d> {
        let points = self
            .vertices
            .iter()
            .chain(self.vertices.first())
            .copied()
            .collect();
        vec![GizmoPart2d::LineStrip { points }]
    }
}
impl GizmoPolygon2d for BoxedPolygon {
    fn get_data(&self) -> Vec<GizmoPart2d> {
        let points = self
            .vertices
            .iter()
            .chain(self.vertices.first())
            .copied()
            .collect();
        vec![GizmoPart2d::LineStrip { points }]
    }
}

impl<'w, 's, T: GizmoConfigGroup, P: GizmoPolygon2d> GizmoPrimitive3d<Extrusion<P>>
    for Gizmos<'w, 's, T>
{
    type Output<'a> = () where Self: 'a;

    fn primitive_3d(
        &mut self,
        primitive: Extrusion<P>,
        position: Vec3,
        rotation: Quat,
        color: impl Into<Color>,
    ) -> Self::Output<'_> {
        if !self.enabled {
            return;
        }

        let color = color.into();
        let parts = primitive.base_shape.get_data();

        let extrusion_d = rotation * Vec3::new(0., 0., -primitive.depth);
        let half_depth = primitive.depth / 2.;
        let arc_rotation = rotation * Quat::from_rotation_x(FRAC_PI_2);
        for p in parts {
            match p {
                GizmoPart2d::Line { a, b } => {
                    let a = position + rotation * a.extend(half_depth);
                    let b = position + rotation * b.extend(half_depth);

                    self.linestrip(vec![b, a, a + extrusion_d, b + extrusion_d], color);
                }
                GizmoPart2d::Arc {
                    center,
                    radius,
                    start,
                    angle,
                } => {
                    let arc_rotation = arc_rotation * Quat::from_rotation_y(start);
                    let center = position + rotation * center.extend(half_depth);
                    self.arc_3d(angle, radius, center, arc_rotation, color);
                    self.arc_3d(angle, radius, center + extrusion_d, arc_rotation, color);

                    let arc_start = center + arc_rotation * Vec3::new(radius, 0., 0.);
                    self.line(arc_start, arc_start + extrusion_d, color);
                }
                GizmoPart2d::LineStrip { points } => {
                    for pair in points
                        .into_iter()
                        .map(|p| position + rotation * p.extend(half_depth))
                        .collect::<Vec<Vec3>>()
                        .windows(2)
                    {
                        let a = pair[0];
                        let b = pair[1];
                        self.linestrip(vec![b, a, a + extrusion_d, b + extrusion_d], color);
                    }
                }
            }
        }
    }
}
