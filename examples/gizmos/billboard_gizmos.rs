//! This example demonstrates Bevy's immediate mode drawing API intended for visual debugging.

use bevy::{color::palettes::css::*, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_camera)
        .add_systems(Update, (draw_example_collection, update_config))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 1., -8.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // text
    commands.spawn(TextBundle::from_section(
        "Hold 'Left' or 'Right' to change the width of gizm os\n\
        Hold 'Up' or 'Down' to change the height of gizmos\n\
        Press 'Space' to toggle the visibility of gizmos\n\
		Press 'P' to toggle perspective for the gizmos",
        TextStyle {
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            font_size: 24.,
            color: Color::WHITE,
        },
    ));
}

fn draw_example_collection(mut gizmos: Gizmos) {
    gizmos.billboard(Vec3::new(-2., -2., 0.), AssetId::<Image>::invalid());
    gizmos.billboard_tinted(Vec3::new(2., -2., 0.), AssetId::<Image>::invalid(), BLUE);
    gizmos.billboard_tinted(Vec3::new(-2., 2., 0.), AssetId::<Image>::invalid(), RED);
    gizmos.billboard_tinted(Vec3::new(2., 2., 0.), AssetId::<Image>::invalid(), YELLOW);
}

fn rotate_camera(mut query: Query<&mut Transform, With<Camera>>, time: Res<Time>) {
    let mut transform = query.single_mut();

    transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(time.delta_seconds() / 2.));
}

fn update_config(
    mut config_store: ResMut<GizmoConfigStore>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    if keyboard.pressed(KeyCode::ArrowRight) {
        config.billboard_size.x += 20. * time.delta_seconds();
        config.billboard_size.x = config.billboard_size.x.clamp(0., 200.);
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        config.billboard_size.x -= 20. * time.delta_seconds();
        config.billboard_size.x = config.billboard_size.x.clamp(0., 200.);
    }

    if keyboard.pressed(KeyCode::ArrowUp) {
        config.billboard_size.y += 20. * time.delta_seconds();
        config.billboard_size.y = config.billboard_size.y.clamp(0., 200.);
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        config.billboard_size.y -= 20. * time.delta_seconds();
        config.billboard_size.y = config.billboard_size.y.clamp(0., 200.);
    }

    if keyboard.just_pressed(KeyCode::Space) {
        config.enabled ^= true;
    }
    if keyboard.just_pressed(KeyCode::KeyP) {
        config.billboard_perspective ^= true;
    }
}
