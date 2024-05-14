//! A minimal example that outputs "hello world"

use bevy::{color::palettes::css::RED, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, hello_world_system)
        .run();
}

#[derive(Debug, Clone, Copy, Component)]
struct Shape;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 0., 8.),
        ..Default::default()
    });

    let extr = Extrusion {
        base_shape: Rectangle::default(),
        depth: 2.,
    };
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(extr),
            material: materials.add(Color::from(RED)),
            ..default()
        },
        Shape,
    ));

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
}

fn hello_world_system(
    mut gizmos: Gizmos,
    mut shapes: Query<&mut Transform, With<Shape>>,
    time: Res<Time>,
) {
    let extr = Extrusion {
        base_shape: Triangle2d::default(),
        depth: 3.,
    };

    gizmos.primitive_3d(
        extr,
        Vec3::ZERO,
        Quat::from_rotation_y(time.elapsed_seconds()),
        RED,
    );

    for mut t in shapes.iter_mut() {
        t.rotation = Quat::from_rotation_y(time.elapsed_seconds());
    }
}
