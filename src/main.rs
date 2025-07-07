use bevy::prelude::*;

// import camera
mod systems;
use systems::camera::{OrbitCamPlugin, OrbitCamera};
use systems::ui::GlobeUIPlugin;

fn main() -> bevy::app::AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(OrbitCamPlugin)
        .add_plugins(GlobeUIPlugin)
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_systems(Startup, setup)
        .run()
}

#[derive(Component)]
pub struct LatLong {
    latitude: f32,
    longitude: f32,
}

// UI components (temp)
#[derive(Component)]
struct RandomizeButton;
#[derive(Component)]
struct CoordinateDisplay;

// convert latlon to cartesian
// need to move this somewhere else
fn latlon_to_pos(latitude: f32, longitude: f32, radius: f32) -> Vec3 {
    let lat_rad = latitude.to_radians();
    let lon_rad = longitude.to_radians();

    // spherical to cartesian conversion
    let x = radius * lat_rad.cos() * lon_rad.cos();
    let y = radius * lat_rad.sin();
    let z = radius * lat_rad.cos() * lon_rad.sin();
    
    Vec3::new(x, y, z)
}

// scene setup here
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // let globe_size = 5.0;

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(5.0).mesh().ico(32).unwrap())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::hex("#ffffff").unwrap().into(),
            metallic: 0.0,
            perceptual_roughness: 0.5,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // test marker thing
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.1).mesh().ico(8).unwrap())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::hex("ff0000").unwrap().into(),
            metallic: 0.0,
            perceptual_roughness: 0.3,
            ..default()
        })),
        Transform::from_translation(latlon_to_pos(0.0, 0.0, 5.0)),
        LatLong {
            latitude: 0.0,
            longitude: 0.0,
        },
    ));

    // sun light
    // need to make it based on real sun position
    commands.spawn((
        DirectionalLight {
            illuminance: 1_500.,
            ..default()
        },
        Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // spawn camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera::new(15.0, 0.5)
            .with_target(Vec3::ZERO)
    ));
}