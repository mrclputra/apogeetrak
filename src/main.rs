use bevy::prelude::*;

// import camera and systems
mod systems;
use systems::camera::{OrbitCamPlugin, OrbitCamera};
use systems::ui::GlobeUIPlugin;
use systems::satellites::render::TlePlugin;
use systems::satellites::labels::LabelsPlugin;

// WGS84
const EARTH_RADIUS: f32 = 6378.0;

fn main() -> bevy::app::AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(OrbitCamPlugin)
        .add_plugins(GlobeUIPlugin)
        .add_plugins(TlePlugin)
        .add_plugins(LabelsPlugin)
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_systems(Startup, setup)
        .run()
}

// set up the main scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // create the Earth sphere
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(EARTH_RADIUS).mesh().ico(32).unwrap())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::hex("#0070a0").unwrap().into(),
            metallic: 0.0,
            perceptual_roughness: 0.5,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // sun light
    // TODO: need to make it based on real sun position
    commands.spawn((
        DirectionalLight {
            illuminance: 1_500.,
            ..default()
        },
        Transform::from_xyz(50000.0, 50000.0, 50000.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // spawn camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-8000.0, 8000.0, 12000.0).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera::new(15000.0, 0.3)
            .with_target(Vec3::ZERO)
            .with_zoom_limits(7000.0, 100000.0)
    ));
}