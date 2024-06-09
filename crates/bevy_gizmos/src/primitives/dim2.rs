//! A module for rendering each of the 2D [`bevy_math::primitives`] with [`Gizmos`].

use std::f32::consts::{FRAC_PI_2, PI, SQRT_2, TAU};

use super::helpers::*;

use bevy_color::Color;
use bevy_math::primitives::{
    Annulus, Arc2d, BoxedPolygon, BoxedPolyline2d, Capsule2d, Circle, CircularSector,
    CircularSegment, Ellipse, Line2d, Plane2d, Polygon, Polyline2d, Primitive2d, Rectangle,
    RegularPolygon, Rhombus, Segment2d, Triangle2d,
};
use bevy_math::{Dir2, Vec2};

use crate::circles::DEFAULT_CIRCLE_RESOLUTION;
use crate::prelude::{GizmoConfigGroup, Gizmos};

// some magic number since using directions as offsets will result in lines of length 1 pixel
const MIN_LINE_LEN: f32 = 50.0;
const HALF_MIN_LINE_LEN: f32 = 25.0;
// length used to simulate infinite lines
const INFINITE_LEN: f32 = 100_000.0;

pub trait GizmoPrimitive2d: Primitive2d {
    type Output: GizmoBuilder2d;

    fn gizmos(&self) -> Self::Output;
}
pub trait GizmoBuilder2d {
    fn linestrips(&self) -> Vec<Vec<Vec2>>;
}
pub struct GizmoPrimitive2dBuilder<'a, 'w, 's, P, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
    P: GizmoPrimitive2d,
{
    pub builder: P::Output,
    gizmos: &'a mut Gizmos<'w, 's, Config, Clear>,
    position: Vec2,
    angle: f32,
    color: Color,
}

impl<'w, 's, Config, Clear> Gizmos<'w, 's, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    pub fn primitive_2d<'a, P: GizmoPrimitive2d>(
        &'a mut self,
        primitive: &P,
        position: Vec2,
        angle: f32,
        color: impl Into<Color>,
    ) -> GizmoPrimitive2dBuilder<'a, 'w, 's, P, Config, Clear> {
        GizmoPrimitive2dBuilder {
            builder: primitive.gizmos(),
            gizmos: self,
            position,
            angle,
            color: color.into(),
        }
    }
}

impl<'a, 'w, 's, Config, Clear, P> Drop for GizmoPrimitive2dBuilder<'a, 'w, 's, P, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
    P: GizmoPrimitive2d,
{
    fn drop(&mut self) {
        if !self.gizmos.enabled {
            return;
        }

        let transform = rotate_then_translate_2d(self.angle, self.position);
        for linestrip in self.builder.linestrips() {
            self.gizmos
                .linestrip_2d(linestrip.into_iter().map(&transform), self.color);
        }
    }
}

// direction 2d

struct Dir2Builder {
    direction: Dir2,
}

impl GizmoBuilder2d for Dir2Builder {
    fn linestrips(&self) -> Vec<Vec<Vec2>> {
        arrow_2d(Vec2::ZERO, self.direction * MIN_LINE_LEN)
    }
}

impl GizmoPrimitive2d for Dir2 {
    type Output = Dir2Builder;

    fn gizmos(&self) -> Self::Output {
        Dir2Builder { direction: *self }
    }
}

// arc 2d

enum ArcKind {
    Arc,
    Sector,
    Segment,
}

struct Arc2dBuilder {
    arc: Arc2d,
    arc_kind: ArcKind,
    resolution: Option<usize>,
}

impl Arc2dBuilder {
    pub fn resolution(mut self, resolution: usize) -> Self {
        self.resolution.replace(resolution);
        self
    }
}

impl GizmoBuilder2d for Arc2dBuilder {
    fn linestrips(&self) -> Vec<Vec<Vec2>> {
        let resolution = self.resolution.unwrap_or_else(|| {
            ((self.arc.half_angle.abs() / PI) * DEFAULT_CIRCLE_RESOLUTION as f32).ceil() as usize
        });

        let mut arc_positions = {
            let mut positions = Vec::with_capacity(resolution + 2);
            let delta = self.arc.half_angle / (resolution as f32);
            let start = FRAC_PI_2 - self.arc.half_angle;
            positions.extend((0..resolution + 1).map(|i| {
                let angle = start + (i as f32 * delta);
                let (sin, cos) = angle.sin_cos();
                Vec2::new(cos, sin) * self.arc.radius
            }));

            positions
        };

        match self.arc_kind {
            ArcKind::Arc => {}
            ArcKind::Sector => {
                arc_positions.push(Vec2::ZERO);
                arc_positions.push(arc_positions[0]);
            }
            ArcKind::Segment => {
                arc_positions.push(arc_positions[0]);
            }
        };

        vec![arc_positions]
    }
}

impl GizmoPrimitive2d for Arc2d {
    type Output = Arc2dBuilder;

    fn gizmos(&self) -> Self::Output {
        Arc2dBuilder {
            arc: *self,
            arc_kind: ArcKind::Arc,
            resolution: None,
        }
    }
}

// circular sector 2d

impl GizmoPrimitive2d for CircularSector {
    type Output = Arc2dBuilder;

    fn gizmos(&self) -> Self::Output {
        Arc2dBuilder {
            arc: self.arc,
            arc_kind: ArcKind::Sector,
            resolution: None,
        }
    }
}

// circular segment 2d

impl GizmoPrimitive2d for CircularSegment {
    type Output = Arc2dBuilder;

    fn gizmos(&self) -> Self::Output {
        Arc2dBuilder {
            arc: self.arc,
            arc_kind: ArcKind::Segment,
            resolution: None,
        }
    }
}

// circle 2d

impl GizmoPrimitive2d for Circle {
    type Output = EllipseBuilder;

    fn gizmos(&self) -> Self::Output {
        EllipseBuilder {
            half_size: Vec2::splat(self.radius),
            resolution: DEFAULT_CIRCLE_RESOLUTION,
        }
    }
}

// ellipse 2d

struct EllipseBuilder {
    half_size: Vec2,
    resolution: usize,
}

impl EllipseBuilder {
    pub fn resolution(mut self, resolution: usize) -> Self {
        self.resolution = resolution;
        self
    }
}

impl GizmoBuilder2d for EllipseBuilder {
    fn linestrips(&self) -> Vec<Vec<Vec2>> {
        vec![ellipse_inner(self.half_size, self.resolution).collect()]
    }
}

impl GizmoPrimitive2d for Ellipse {
    type Output = EllipseBuilder;

    fn gizmos(&self) -> Self::Output {
        EllipseBuilder {
            half_size: self.half_size,
            resolution: DEFAULT_CIRCLE_RESOLUTION,
        }
    }
}

// annulus 2d

/// Builder for configuring the drawing options of [`Annulus`].
pub struct AnnulusBuilder {
    inner_radius: f32,
    outer_radius: f32,
    inner_resolution: usize,
    outer_resolution: usize,
}

impl AnnulusBuilder {
    /// Set the number of line-segments for each circle of the annulus.
    pub fn resolution(mut self, resolution: usize) -> Self {
        self.outer_resolution = resolution;
        self.inner_resolution = resolution;
        self
    }

    /// Set the number of line-segments for the outer circle of the annulus.
    pub fn outer_resolution(mut self, resolution: usize) -> Self {
        self.outer_resolution = resolution;
        self
    }

    /// Set the number of line-segments for the inner circle of the annulus.
    pub fn inner_resolution(mut self, resolution: usize) -> Self {
        self.inner_resolution = resolution;
        self
    }
}

impl GizmoBuilder2d for AnnulusBuilder {
    fn linestrips(&self) -> Vec<Vec<Vec2>> {
        let inner_positions =
            ellipse_inner(Vec2::splat(self.inner_radius), self.inner_resolution).collect();
        let outer_positions =
            ellipse_inner(Vec2::splat(self.outer_radius), self.outer_resolution).collect();

        vec![inner_positions, outer_positions]
    }
}

impl GizmoPrimitive2d for Annulus {
    type Output = AnnulusBuilder;

    fn gizmos(&self) -> Self::Output {
        AnnulusBuilder {
            inner_radius: self.inner_circle.radius,
            outer_radius: self.outer_circle.radius,
            inner_resolution: DEFAULT_CIRCLE_RESOLUTION,
            outer_resolution: DEFAULT_CIRCLE_RESOLUTION,
        }
    }
}

// rhombus 2d

struct RhombusBuilder {
    half_diagonals: Vec2,
}

impl GizmoBuilder2d for RhombusBuilder {
    fn linestrips(&self) -> Vec<Vec<Vec2>> {
        vec![vec![
            Vec2::new(self.half_diagonals.x, 0.0),
            Vec2::new(0.0, self.half_diagonals.y),
            Vec2::new(-self.half_diagonals.x, 0.0),
            Vec2::new(0.0, -self.half_diagonals.y),
            Vec2::new(self.half_diagonals.x, 0.0),
        ]]
    }
}

impl GizmoPrimitive2d for Rhombus {
    type Output = RhombusBuilder;

    fn gizmos(&self) -> Self::Output {
        RhombusBuilder {
            half_diagonals: self.half_diagonals,
        }
    }
}

// capsule 2d

struct Capsule2dBuilder {
    radius: f32,
    half_length: f32,
    resolution: usize,
}

impl GizmoBuilder2d for Capsule2dBuilder {
    fn linestrips(&self) -> Vec<Vec<Vec2>> {
        let arc_points = &Arc2d::new(self.radius, FRAC_PI_2)
            .gizmos()
            .resolution(self.resolution)
            .linestrips()[0];
        let mut positions = Vec::with_capacity(arc_points.len() * 2 + 1);
        positions.extend(
            arc_points
                .iter()
                .map(|p| Vec2::new(0.0, self.half_length) + *p),
        );
        positions.extend(
            arc_points
                .into_iter()
                .map(|p| Vec2::new(0.0, -self.half_length) - *p),
        );
        positions.push(positions[0]);

        vec![positions]
    }
}

impl GizmoPrimitive2d for Capsule2d {
    type Output = Capsule2dBuilder;

    fn gizmos(&self) -> Self::Output {
        Capsule2dBuilder {
            radius: self.radius,
            half_length: self.half_length,
            resolution: DEFAULT_CIRCLE_RESOLUTION,
        }
    }
}

// line 2d
//
/// Builder for configuring the drawing options of [`Line2d`].
pub struct Line2dBuilder {
    direction: Dir2, // Direction of the line

    draw_arrow: bool, // decides whether to indicate the direction of the line with an arrow
}

impl Line2dBuilder {
    /// Set the drawing mode of the line (arrow vs. plain line)
    pub fn draw_arrow(mut self, is_enabled: bool) -> Self {
        self.draw_arrow = is_enabled;
        self
    }
}

impl GizmoBuilder2d for Line2dBuilder {
    fn linestrips(&self) -> Vec<Vec<Vec2>> {
        let start = self.direction * INFINITE_LEN;
        let end = -start;

        if self.draw_arrow {
            vec![
                vec![start, end],
                arrow_head(Vec2::ZERO, self.direction, MIN_LINE_LEN),
            ]
        } else {
            vec![vec![start, end]]
        }
    }
}

impl GizmoPrimitive2d for Line2d {
    type Output = Line2dBuilder;

    fn gizmos(&self) -> Self::Output {
        Self::Output {
            direction: self.direction,
            draw_arrow: false,
        }
    }
}
// plane 2d

struct Plane2dBuilder {
    normal: Dir2,
}

impl GizmoBuilder2d for Plane2dBuilder {
    fn linestrips(&self) -> Vec<Vec<Vec2>> {
        let line_dir = Dir2::new_unchecked(-self.normal.perp());

        // The normal of the plane (orthogonal to the plane itself)
        let mut linestrips = arrow_2d(Vec2::ZERO, self.normal * MIN_LINE_LEN);
        // The plane line
        linestrips.push(vec![line_dir * INFINITE_LEN, -line_dir * INFINITE_LEN]);

        // An arrow such that the normal is always left side of the plane with respect to the
        // planes direction. This is to follow the "counter-clockwise" convention
        linestrips.push(arrow_head(
            line_dir * MIN_LINE_LEN,
            line_dir,
            MIN_LINE_LEN / 10.,
        ));

        linestrips
    }
}

impl GizmoPrimitive2d for Plane2d {
    type Output = Plane2dBuilder;

    fn gizmos(&self) -> Self::Output {
        Self::Output {
            normal: self.normal,
        }
    }
}

// segment 2d

/// Builder for configuring the drawing options of [`Segment2d`].
pub struct Segment2dBuilder {
    direction: Dir2,  // Direction of the line segment
    half_length: f32, // Half-length of the line segment

    draw_arrow: bool, // decides whether to draw just a line or an arrow
}

impl Segment2dBuilder {
    /// Set the drawing mode of the line (arrow vs. plain line)
    pub fn draw_arrow(mut self, is_enabled: bool) -> Self {
        self.draw_arrow = is_enabled;
        self
    }
}

impl GizmoBuilder2d for Segment2dBuilder {
    fn linestrips(&self) -> Vec<Vec<Vec2>> {
        let end = self.direction.as_vec2() * self.half_length;
        let start = -end;
        if self.draw_arrow {
            arrow_2d(start, end)
        } else {
            let segment = vec![start, end];
            vec![segment]
        }
    }
}

impl GizmoPrimitive2d for Segment2d {
    type Output = Segment2dBuilder;

    fn gizmos(&self) -> Self::Output {
        Self::Output {
            direction: self.direction,
            half_length: self.half_length,
            draw_arrow: false,
        }
    }
}

// polyline 2d

struct Polyline2dBuilder<'a> {
    vertices: &'a [Vec2],
}

impl<'a> GizmoBuilder2d for Polyline2dBuilder<'a> {
    fn linestrips(&self) -> Vec<Vec<Vec2>> {
        match self.vertices.len() {
            0 | 1 => vec![vec![]],
            _ => vec![self.vertices.into()],
        }
    }
}

impl<const N: usize> GizmoPrimitive2d for Polyline2d<N> {
    type Output<'a> = Polyline2dBuilder<'a>;

    fn gizmos(&self) -> Self::Output {
        Self::Output {
            vertices: &self.vertices,
        }
    }
}

// boxed polyline 2d

impl GizmoPrimitive2d for BoxedPolyline2d {
    type Output<'a> = Polyline2dBuilder<'a>;

    fn gizmos(&self) -> Self::Output {
        Self::Output {
            vertices: &self.vertices,
        }
    }
}

// triangle 2d

struct Triangle2dBuilder {
    vertices: [Vec2; 3],
}

impl GizmoBuilder2d for Triangle2dBuilder {
    fn linestrips(&self) -> Vec<Vec<Vec2>> {
        vec![vec![
            self.vertices[0],
            self.vertices[1],
            self.vertices[2],
            self.vertices[0],
        ]]
    }
}

impl GizmoPrimitive2d for Triangle2d {
    type Output = Triangle2dBuilder;

    fn gizmos(&self) -> Self::Output {
        Triangle2dBuilder {
            vertices: self.vertices,
        }
    }
}

// rectangle 2d

struct RectangleBuilder {
    half_size: Vec2,
}

impl GizmoBuilder2d for RectangleBuilder {
    fn linestrips(&self) -> Vec<Vec<Vec2>> {
        vec![vec![
            self.half_size,
            Vec2::new(self.half_size.x, -self.half_size.y),
            Vec2::new(-self.half_size.x, -self.half_size.y),
            Vec2::new(-self.half_size.x, self.half_size.y),
            self.half_size,
        ]]
    }
}

impl GizmoPrimitive2d for Rectangle {
    type Output = RectangleBuilder;

    fn gizmos(&self) -> Self::Output {
        RectangleBuilder {
            half_size: self.half_size,
        }
    }
}

// polygon 2d

struct PolygonBuilder<'a> {
    vertices: &'a [Vec2],
}

impl<'a> GizmoBuilder2d for PolygonBuilder<'a> {
    fn linestrips(&self) -> Vec<Vec<Vec2>> {
        match self.vertices.len() {
            0 | 1 => vec![vec![]],
            2 => vec![self.vertices.into()],
            _ => vec![self
                .vertices
                .iter()
                .copied()
                .chain([self.vertices[0]])
                .collect()],
        }
    }
}

impl<const N: usize> GizmoPrimitive2d for Polygon<N> {
    type Output<'a> = PolygonBuilder<'a>;

    fn gizmos(&self) -> Self::Output {
        PolygonBuilder {
            vertices: &self.vertices,
        }
    }
}

// boxed polygon 2d

impl GizmoPrimitive2d for BoxedPolygon {
    type Output<'a> = PolygonBuilder<'a>;

    fn gizmos(&self) -> Self::Output {
        PolygonBuilder {
            vertices: &self.vertices,
        }
    }
}

// regular polygon 2d

struct RegularPolygonBuilder {
    circumradius: f32,
    sides: usize,
}

impl GizmoBuilder2d for RegularPolygonBuilder {
    fn linestrips(&self) -> Vec<Vec<Vec2>> {
        let points = (0..=self.sides)
            .map(|p| single_circle_coordinate(self.circumradius, self.sides, p))
            .collect();

        vec![points]
    }
}

impl GizmoPrimitive2d for RegularPolygon {
    type Output = RegularPolygonBuilder;

    fn gizmos(&self) -> Self::Output {
        RegularPolygonBuilder {
            circumradius: self.circumradius(),
            sides: self.sides,
        }
    }
}

fn arrow_2d(start: Vec2, end: Vec2) -> Vec<Vec<Vec2>> {
    let segment = vec![start, end];

    let tip_length = (end - start).length() / 10.;
    let direction = Dir2::new_unchecked((end - start).normalize());

    vec![segment, arrow_head(end, direction, tip_length)]
}

fn arrow_head(position: Vec2, direction: Dir2, tip_length: f32) -> Vec<Vec2> {
    let left_offset = direction.rotate(Vec2::new(-SQRT_2, SQRT_2)) * tip_length;
    let right_offset = direction.rotate(Vec2::new(-SQRT_2, -SQRT_2)) * tip_length;

    vec![position + left_offset, position, position + right_offset]
}

fn ellipse_inner(half_size: Vec2, resolution: usize) -> impl Iterator<Item = Vec2> {
    (0..resolution + 1).map(move |i| {
        let angle = i as f32 * TAU / resolution as f32;
        let (x, y) = angle.sin_cos();
        Vec2::new(x, y) * half_size
    })
}
