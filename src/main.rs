//! main.rs
//!
//! Marcel Putra
//! 04-08-2025
//! TLE SGP4 satellite visualizer entry point.
//! NORAD datasets are included in the assets folder

use bevy::prelude::*;
use bevy::pbr::wireframe::{WireframePlugin, WireframeConfig};

pub mod config;

mod systems;
use systems::time::TimePlugin;
use systems::camera::CameraPlugin;
use systems::ui::UIPlugin;

use systems::satellites::SatellitePlugin;
use systems::earth::EarthPlugin;

#[derive(Component)]
pub struct Sun;

fn main() -> bevy::app::AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes_override: Some(true),
            ..default()
        }))
        .add_plugins(WireframePlugin::default())
        .insert_resource(WireframeConfig {
            global: false, // toggle wireframes here
            default_color: Color::BLACK,
        })
        .add_plugins(TimePlugin) // IMPORTANT
        .add_plugins(CameraPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(SatellitePlugin)
        .add_plugins(EarthPlugin)
        .insert_resource(ClearColor(Color::BLACK)) // background color
        .add_systems(Startup, setup)
        .run()
}

fn setup(
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

    // spawn sun light source
    commands.spawn((
        DirectionalLight {
            illuminance: 1_700.,
            ..default()
        },
        Transform::from_xyz(50000.0, 50000.0, 50000.0).looking_at(Vec3::ZERO, Vec3::Y),
        Sun,
    ));
}