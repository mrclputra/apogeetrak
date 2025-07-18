use bevy::prelude::*;
use bevy::pbr::wireframe::{WireframePlugin, WireframeConfig};

pub mod constants;

// import camera and systems
mod systems;
use systems::camera::CameraPlugin;
use systems::ui::UIPlugin;

use crate::systems::earth2::mesh::generate_sphere;
// use systems::satellites::SatellitePlugin;
// use systems::earth::EarthPlugin;

#[derive(Component)]
pub struct Sun;

fn main() -> bevy::app::AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WireframePlugin::default())
        .insert_resource(WireframeConfig {
            global: true,
            default_color: Color::BLACK,
        })
        .add_plugins(CameraPlugin)
        .add_plugins(UIPlugin)
        // .add_plugins(SatellitePlugin)
        // .add_plugins(EarthPlugin)
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0))) // background color
        .add_systems(Startup, setup_scene)
        .run()
}

// set up the main scene
fn setup_scene(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    // spawn camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-8000.0, 8000.0, 12000.0).looking_at(Vec3::ZERO, Vec3::Y),
        systems::camera::OrbitCamera::new(15000.0, 0.3)
            .with_target(Vec3::ZERO)
            .with_zoom_limits(7000.0, 100000.0)
    ));

    // spawn the sun light
    commands.spawn((
        DirectionalLight {
            illuminance: 1_500.,
            ..default()
        },
        Transform::from_xyz(50000.0, 50000.0, 50000.0).looking_at(Vec3::ZERO, Vec3::Y),
        Sun, // component marker for sun tracking
    ));

    // spawn the earth (temp, need to move to own plugin)
    generate_sphere(commands, meshes, materials);
}