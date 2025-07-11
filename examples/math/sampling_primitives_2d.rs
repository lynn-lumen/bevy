//! Sample a 2D primitive

use std::f32::consts::FRAC_PI_3;

use bevy::{
    color::palettes::css::{RED, WHITE},
    prelude::*,
};
use rand::{prelude::Distribution, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<SampledPoints>()
        .insert_resource(RandomSource(ChaCha8Rng::seed_from_u64(1)))
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_keypress, draw_samples))
        .run();
}

type Primitive = CircularSector;
const PRIMITIVE: Primitive = Primitive {
    arc: Arc2d {
        radius: 300.0,
        half_angle: FRAC_PI_3,
    },
};

#[derive(Debug, Clone, Default, Resource)]
struct SampledPoints(Vec<Vec2>);

/// The source of randomness used by this example.
#[derive(Resource)]
struct RandomSource(ChaCha8Rng);

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Mesh2d(meshes.add(PRIMITIVE)),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(WHITE))),
        Transform::IDENTITY,
    ));
}

fn draw_samples(mut gizmos: Gizmos, samples: Res<SampledPoints>) {
    for s in samples.0.iter() {
        gizmos.cross_2d(Isometry2d::from_translation(*s), 10.0, RED);
    }
}

// Handle user inputs from the keyboard:
fn handle_keypress(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut random_source: ResMut<RandomSource>,
    mut samples: ResMut<SampledPoints>,
) {
    // Space => restart, deleting all samples
    if keyboard.just_pressed(KeyCode::Space) {
        samples.0.clear();
    }

    let mut rng = &mut random_source.0;
    if keyboard.just_pressed(KeyCode::KeyS) {
        let sample = PRIMITIVE.sample_interior(rng);
        samples.0.push(sample);
    }
    if keyboard.just_pressed(KeyCode::KeyD) {
        let sample = PRIMITIVE.sample_boundary(rng);
        samples.0.push(sample);
    }

    if keyboard.just_pressed(KeyCode::KeyK) {
        let dist = PRIMITIVE.interior_dist().sample_iter(&mut rng).take(100);
        samples.0.extend(dist);
    }
    if keyboard.just_pressed(KeyCode::KeyL) {
        let dist = PRIMITIVE.boundary_dist().sample_iter(&mut rng).take(100);
        samples.0.extend(dist);
    }
}
