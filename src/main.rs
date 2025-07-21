use bevy::prelude::*;
use bevy::pbr::wireframe::{WireframePlugin, WireframeConfig};

pub mod config;

// import camera and systems
mod systems;
use systems::camera::CameraPlugin;
use systems::ui::UIPlugin;

use systems::satellites::SatellitePlugin;
use systems::earth::EarthPlugin;

#[derive(Component)]
pub struct Sun;

fn main() -> bevy::app::AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugins(WireframePlugin::default())
        // .insert_resource(WireframeConfig {
        //     global: true,
        //     default_color: Color::BLACK,
        // })
        .add_plugins(CameraPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(SatellitePlugin)
        .add_plugins(EarthPlugin)
        .insert_resource(ClearColor(Color::BLACK)) // background color
        .add_systems(Startup, start)
        .run()
}

// set up the main scene
fn start(
    mut commands: Commands,
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
            illuminance: 1_700.,
            ..default()
        },
        Transform::from_xyz(50000.0, 50000.0, 50000.0).looking_at(Vec3::ZERO, Vec3::Y),
        Sun, // component marker for sun tracking
    ));
}