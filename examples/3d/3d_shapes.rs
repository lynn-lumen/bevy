//! This example demonstrates the built-in 3d shapes in Bevy.
//! The scene includes a patterned texture and a rotation for visualizing the normals and UVs.

use std::f32::consts::{PI, TAU};

use bevy::{
    color::palettes::css::{BLUE, GREEN, RED},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_resource::<Axes>()
        .add_systems(Startup, setup)
        .add_systems(Update, (input, draw_gizmos))
        .run();
}

impl Projectable for ShapeProjection<ConicalFrustum> {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        let r_bottom = self.primitive.radius_bottom;
        let r_top = self.primitive.radius_top;
        let local_y = (self.rotation * Vec3::Y).xy().normalize_or(Vec2::Y);
        let local_x = local_y.rotate(Vec2::NEG_Y);
        let dir = self.rotation.conjugate() * Vec3::NEG_Z;

        let b_bottom = dir.y.abs() * r_bottom;
        let b_top = dir.y.abs() * r_top;
        let y_offset = self.primitive.height / 2. * dir.xz().length();
        let rotation = local_x.to_angle();

        if b_bottom > 2. * y_offset + b_top {
            return vec![PerimeterSegment::EllipticArc {
                center: -y_offset * local_y,
                half_size: Vec2::new(r_bottom, b_bottom),
                rotation,
                start_angle: 0.,
                angle: TAU,
            }];
        }

        let cone_height = self.primitive.height / (1. - r_top / r_bottom) * dir.xz().length();
        let bottom_x = r_bottom * (1. - (b_bottom / cone_height).powi(2)).sqrt();
        let bottom_y = b_bottom * (1. - (bottom_x / r_bottom).powi(2)).sqrt() - y_offset;
        let bottom_angle =
            Vec2::new(b_bottom * bottom_x / r_bottom, bottom_y + y_offset).to_angle();

        let top_x = r_top * (1. - (b_top / (cone_height - 2. * y_offset)).powi(2)).sqrt();
        let top_y = b_top * (1. - (top_x / r_top).powi(2)).sqrt() + y_offset;
        let top_angle = Vec2::new(b_top * top_x / r_top, top_y - y_offset).to_angle();
        vec![
            PerimeterSegment::LineStrip {
                points: vec![
                    bottom_x * local_x + bottom_y * local_y,
                    top_x * local_x + top_y * local_y,
                ],
            },
            PerimeterSegment::LineStrip {
                points: vec![
                    -bottom_x * local_x + bottom_y * local_y,
                    -top_x * local_x + top_y * local_y,
                ],
            },
            PerimeterSegment::EllipticArc {
                center: -y_offset * local_y,
                half_size: Vec2::new(r_bottom, b_bottom),
                rotation: rotation + PI,
                start_angle: -bottom_angle,
                angle: PI + 2. * bottom_angle,
            },
            PerimeterSegment::EllipticArc {
                center: y_offset * local_y,
                half_size: Vec2::new(r_top, b_top),
                rotation,
                start_angle: top_angle,
                angle: PI - 2. * top_angle,
            },
        ]
    }
}
impl Projectable for ShapeProjection<Torus> {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        let m = self.primitive.minor_radius;
        let a = self.primitive.major_radius;
        let local_y = (self.rotation * Vec3::Y).xy().normalize_or(Vec2::Y);
        let local_x = local_y.rotate(Vec2::NEG_Y);
        let dir = self.rotation.conjugate() * Vec3::NEG_Z;

        let b = dir.y.abs() * a;
        let mut segments = vec![PerimeterSegment::Curve {
            sampler: Box::new(move |t: f32| {
                if t < 0.0001 || t > 0.999 {
                    return local_y * -(b + m);
                }

                let nphi = (t - 0.25) * TAU;
                let phi = (nphi.tan() * b / a).atan() + if t > 0.5 { PI } else { 0. };

                let (sin, cos) = phi.sin_cos();
                let p = Vec2::new(a * cos, b * sin);
                let (sin, cos) = nphi.sin_cos();
                let n = Vec2::new(cos, sin) * m;

                let p = n + p;
                local_x * p.x + local_y * p.y
            }),
        }];

        if b <= m {
            return segments;
        }

        let start_phi = {
            let phi = ((b * b - m * m).sqrt() / (a * a - b * b).sqrt() * a / b).acos();
            if phi.is_nan() {
                0.
            } else {
                phi
            }
        };
        segments.push(PerimeterSegment::Curve {
            sampler: Box::new(move |t: f32| {
                let phi = start_phi + (PI - 2. * start_phi) * t;
                let (sin, cos) = phi.sin_cos();
                let p = Vec2::new(a * cos, b * sin);
                let n = m * Vec2::new(b * cos, a * sin).normalize();
                (p - n).x * local_x + (p - n).y * local_y
            }),
        });
        segments.push(PerimeterSegment::Curve {
            sampler: Box::new(move |t: f32| {
                let phi = start_phi + (PI - 2. * start_phi) * t;
                let (sin, cos) = phi.sin_cos();
                let p = Vec2::new(a * cos, b * sin);
                let n = m * Vec2::new(b * cos, a * sin).normalize();
                (p - n).x * local_x - (p - n).y * local_y
            }),
        });
        segments
    }
}
impl Projectable for ShapeProjection<Cuboid> {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        let mut points = [
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
        ]
        .map(|p| (self.rotation * (p * self.primitive.half_size)).xy());
        points.sort_by(|a, b| (&a.to_angle()).total_cmp(&b.to_angle()));
        let mut final_positions = vec![];
        for i in 0..points.len() {
            let a = points[(i as i32 - 1).rem_euclid(points.len() as i32) as usize];
            let b = points[(i).rem_euclid(points.len())];
            let c = points[(i + 1).rem_euclid(points.len())];

            let ac = c - a;
            let n = Vec2::new(-ac.y, ac.x).normalize();
            if (n * b).element_sum() - (n * a).element_sum() > 0. {
                continue;
            }
            final_positions.push(b);
        }
        final_positions.push(final_positions[0]);

        vec![PerimeterSegment::LineStrip {
            points: final_positions,
        }]
    }
}
impl Projectable for ShapeProjection<Tetrahedron> {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        let mut points = self.primitive.vertices.map(|p| (self.rotation * p).xy());
        points.sort_by(|a, b| (&a.to_angle()).total_cmp(&b.to_angle()));
        let mut final_positions = vec![];
        for i in 0..points.len() {
            let a = points[(i as i32 - 1).rem_euclid(points.len() as i32) as usize];
            let b = points[(i).rem_euclid(points.len())];
            let c = points[(i + 1).rem_euclid(points.len())];

            let ac = c - a;
            let n = Vec2::new(-ac.y, ac.x).normalize();
            if (n * b).element_sum() - (n * a).element_sum() > 0. {
                continue;
            }
            final_positions.push(b);
        }
        final_positions.push(final_positions[0]);

        vec![PerimeterSegment::LineStrip {
            points: final_positions,
        }]
    }
}
impl Projectable for ShapeProjection<Cone> {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        let r = self.primitive.radius;
        let local_y = (self.rotation * Vec3::Y).xy().normalize_or(Vec2::Y);
        let local_x = local_y.rotate(Vec2::NEG_Y);
        let dir = self.rotation.conjugate() * Vec3::NEG_Z;

        let semi_minor = dir.y.abs() * r;
        let y_offset = self.primitive.height / 2. * dir.xz().length();
        let rotation = local_x.to_angle();

        if semi_minor > 2. * y_offset {
            return vec![PerimeterSegment::EllipticArc {
                center: -y_offset * local_y,
                half_size: Vec2::new(r, semi_minor),
                rotation,
                start_angle: 0.,
                angle: TAU,
            }];
        }

        let intersect_x = r * (1. - (semi_minor / 2. / y_offset).powi(2)).sqrt();
        let intersect_y = semi_minor * (1. - (intersect_x / r).powi(2)).sqrt() - y_offset;
        let intersect_angle =
            Vec2::new(semi_minor * intersect_x / r, intersect_y + y_offset).to_angle();
        let full_angle = PI + 2. * intersect_angle;
        vec![
            PerimeterSegment::LineStrip {
                points: vec![
                    intersect_x * local_x + intersect_y * local_y,
                    y_offset * local_y,
                    -intersect_x * local_x + intersect_y * local_y,
                ],
            },
            PerimeterSegment::EllipticArc {
                center: -y_offset * local_y,
                half_size: Vec2::new(r, semi_minor),
                rotation: rotation + PI,
                start_angle: -intersect_angle,
                angle: full_angle,
            },
        ]
    }
}
impl Projectable for ShapeProjection<Capsule3d> {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        let r = self.primitive.radius;
        let local_y = (self.rotation * Vec3::Y).xy().normalize_or(Vec2::Y);
        let local_x = local_y.rotate(Vec2::NEG_Y);
        let dir = self.rotation.conjugate() * Vec3::NEG_Z;

        let y_offset = self.primitive.half_length * dir.xz().length();
        let rotation = local_x.to_angle();
        vec![
            PerimeterSegment::EllipticArc {
                center: y_offset * local_y,
                half_size: Vec2::splat(r),
                rotation,
                start_angle: 0.,
                angle: PI,
            },
            PerimeterSegment::EllipticArc {
                center: -y_offset * local_y,
                half_size: Vec2::splat(r),
                rotation,
                start_angle: 0.,
                angle: -PI,
            },
            PerimeterSegment::LineStrip {
                points: vec![
                    r * local_x + y_offset * local_y,
                    r * local_x - y_offset * local_y,
                ],
            },
            PerimeterSegment::LineStrip {
                points: vec![
                    -r * local_x + y_offset * local_y,
                    -r * local_x - y_offset * local_y,
                ],
            },
        ]
    }
}
impl Projectable for ShapeProjection<Cylinder> {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        let r = self.primitive.radius;
        let local_y = (self.rotation * Vec3::Y).xy().normalize_or(Vec2::Y);
        let local_x = local_y.rotate(Vec2::NEG_Y);
        let dir = self.rotation.conjugate() * Vec3::NEG_Z;

        let semi_minor = dir.y.abs() * r;
        let y_offset = self.primitive.half_height * dir.xz().length();
        let rotation = local_x.to_angle();
        vec![
            PerimeterSegment::EllipticArc {
                center: y_offset * local_y,
                half_size: Vec2::new(r, semi_minor),
                rotation,
                start_angle: 0.,
                angle: PI,
            },
            PerimeterSegment::EllipticArc {
                center: -y_offset * local_y,
                half_size: Vec2::new(r, semi_minor),
                rotation,
                start_angle: 0.,
                angle: -PI,
            },
            PerimeterSegment::LineStrip {
                points: vec![
                    r * local_x + y_offset * local_y,
                    r * local_x - y_offset * local_y,
                ],
            },
            PerimeterSegment::LineStrip {
                points: vec![
                    -r * local_x + y_offset * local_y,
                    -r * local_x - y_offset * local_y,
                ],
            },
        ]
    }
}
impl Projectable for ShapeProjection<Sphere> {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        vec![PerimeterSegment::EllipticArc {
            center: Vec2::ZERO,
            half_size: Vec2::splat(self.primitive.radius),
            rotation: 0.,
            start_angle: 0.,
            angle: TAU,
        }]
    }
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape(usize);
/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Resource, Default)]
struct Axes(bool);

const X_EXTENT: f32 = 12.0;

fn draw_gizmos(shapes: Query<(&Transform, &Shape)>, mut gizmos: Gizmos, axes: Res<Axes>) {
    let mut first = true;
    for (t, Shape(i)) in shapes.iter() {
        if first {
            let dir = t.rotation.conjugate() * Vec3::NEG_Z;
            gizmos.line_2d(
                Vec2::Y * (3. + 0.0),
                Vec2::X * X_EXTENT / 2. * dir.x + Vec2::Y * (3. + 0.0),
                RED,
            );
            gizmos.line_2d(
                Vec2::Y * (3. + 0.1),
                Vec2::X * X_EXTENT / 2. * dir.y + Vec2::Y * (3. + 0.1),
                GREEN,
            );
            gizmos.line_2d(
                Vec2::Y * (3. + 0.2),
                Vec2::X * X_EXTENT / 2. * dir.z + Vec2::Y * (3. + 0.2),
                BLUE,
            );
            first = false;
        }

        if axes.0 {
            gizmos.axes(t.clone(), 1.);
        }

        let num_shapes = 8;
        let color = Color::hsl(360. * *i as f32 / num_shapes as f32, 0.95, 0.7);
        match *i {
            0 => gizmos.projection(
                ShapeProjection::new(Cylinder::default(), t.rotation),
                t.translation.xy(),
                color,
            ),
            1 => gizmos.projection(
                ShapeProjection::new(Capsule3d::default(), t.rotation),
                t.translation.xy(),
                color,
            ),
            2 => gizmos.projection(
                ShapeProjection::new(Sphere::default(), t.rotation),
                t.translation.xy(),
                color,
            ),
            3 => gizmos.projection(
                ShapeProjection::new(Cone::default(), t.rotation),
                t.translation.xy(),
                color,
            ),
            4 => gizmos.projection(
                ShapeProjection::new(Tetrahedron::default(), t.rotation),
                t.translation.xy(),
                color,
            ),
            5 => gizmos.projection(
                ShapeProjection::new(Cuboid::default(), t.rotation),
                t.translation.xy(),
                color,
            ),
            6 => gizmos.projection(
                ShapeProjection::new(Torus::default(), t.rotation),
                t.translation.xy(),
                color,
            ),
            7 => gizmos.projection(
                ShapeProjection::new(ConicalFrustum::default(), t.rotation),
                t.translation.xy(),
                color,
            ),
            _ => todo!(),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gizmo_config: ResMut<GizmoConfigStore>,
) {
    let (config, _) = gizmo_config.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -1.;

    let shapes = [
        meshes.add(Cylinder::default()),
        meshes.add(Capsule3d::default()),
        meshes.add(Sphere::default().mesh().uv(32, 18)),
        meshes.add(Cone::default().mesh()),
        meshes.add(Tetrahedron::default().mesh()),
        meshes.add(Cuboid::default().mesh()),
        meshes.add(Torus::default().mesh()),
        meshes.add(ConicalFrustum::default().mesh()),
    ];

    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        let x = if num_shapes > 1 {
            -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT
        } else {
            0.
        };
        let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);
        let mat = materials.add(StandardMaterial {
            base_color: color,
            ..default()
        });
        commands.spawn((
            PbrBundle {
                mesh: shape,
                material: mat,
                transform: Transform::from_xyz(x, 0.0, 0.0)
                    .with_rotation(Quat::from_rotation_x(-PI / 4.)),
                ..default()
            },
            Shape(i),
        ));
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        projection: Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::AutoMax {
                max_width: 15.,
                max_height: 200.,
            },
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 0., 12.0),
        ..default()
    });
}

fn input(
    mut query: Query<(&mut Transform, &mut Visibility), With<Shape>>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut axes: ResMut<Axes>,
) {
    let around_x = {
        let mut delta = 0.;
        if keys.pressed(KeyCode::KeyS) {
            delta += 1.;
        }
        if keys.pressed(KeyCode::KeyW) {
            delta -= 1.;
        }
        delta * time.delta_seconds()
    };
    let around_y = {
        let mut delta = 0.;
        if keys.pressed(KeyCode::KeyD) {
            delta += 1.;
        }
        if keys.pressed(KeyCode::KeyA) {
            delta -= 1.;
        }
        delta * time.delta_seconds()
    };
    let around_z = {
        let mut delta = 0.;
        if keys.pressed(KeyCode::KeyQ) {
            delta += 1.;
        }
        if keys.pressed(KeyCode::KeyE) {
            delta -= 1.;
        }
        delta * time.delta_seconds()
    };

    let reset = keys.just_pressed(KeyCode::KeyR);
    let toggle_visibility = keys.just_pressed(KeyCode::KeyV);

    axes.0 ^= keys.just_pressed(KeyCode::KeyC);

    for (mut transform, mut visibility) in &mut query {
        if reset {
            transform.rotation = Quat::IDENTITY;
        } else {
            transform.rotate_x(around_x);
            transform.rotate_y(around_y);
            transform.rotate_z(around_z);
        }

        if toggle_visibility {
            if *visibility == Visibility::Hidden {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

enum PerimeterSegment {
    Curve {
        sampler: Box<dyn Fn(f32) -> Vec2>,
    },
    EllipticArc {
        center: Vec2,
        half_size: Vec2,
        rotation: f32,
        start_angle: f32,
        angle: f32,
    },
    LineStrip {
        points: Vec<Vec2>,
    },
}

trait Projectable {
    fn perimeter(&self) -> Vec<PerimeterSegment>;
}

struct ShapeProjection<P: Primitive3d>
where
    ShapeProjection<P>: Projectable,
{
    primitive: P,
    rotation: Quat,
}
impl<P: Primitive3d> ShapeProjection<P>
where
    ShapeProjection<P>: Projectable,
{
    fn new(primitive: P, rotation: Quat) -> Self {
        Self {
            primitive,
            rotation,
        }
    }
}

trait GizmoProjection<P: Primitive3d>
where
    ShapeProjection<P>: Projectable,
{
    fn projection(&mut self, projection: ShapeProjection<P>, position: Vec2, color: Color);
}
impl<'w, 's, Config, Clear, P: Primitive3d> GizmoProjection<P> for Gizmos<'w, 's, Config, Clear>
where
    ShapeProjection<P>: Projectable,
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    fn projection(&mut self, projection: ShapeProjection<P>, position: Vec2, color: Color) {
        const DEFAULT_SAMPLES: usize = 64;

        for segment in projection.perimeter() {
            match segment {
                PerimeterSegment::Curve { sampler } => {
                    let mut linestrip = vec![];
                    for i in 0..DEFAULT_SAMPLES {
                        let t = i as f32 / (DEFAULT_SAMPLES as f32 - 1.);
                        linestrip.push(sampler(t) + position);
                    }

                    self.linestrip_2d(linestrip, color);
                }
                PerimeterSegment::EllipticArc {
                    center,
                    half_size,
                    rotation,
                    start_angle,
                    angle,
                } => {
                    let rotation = Vec2::from_angle(rotation);
                    let mut linestrip = vec![];

                    for i in 0..DEFAULT_SAMPLES {
                        let t = i as f32 / (DEFAULT_SAMPLES as f32 - 1.);
                        let phi = t * angle + start_angle;

                        let p = Vec2::from_angle(phi) * half_size;
                        let p = p.rotate(rotation);
                        linestrip.push(p + center + position);
                    }

                    self.linestrip_2d(linestrip, color);
                }
                PerimeterSegment::LineStrip { points } => {
                    self.linestrip_2d(points.into_iter().map(|p| p + position), color);
                }
            }
        }
    }
}
