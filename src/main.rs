use bevy::prelude::*;

// import camera and systems
mod systems;
use systems::camera::OrbitCamPlugin;
use systems::ui::GlobeUIPlugin;
use systems::satellites::TlePlugin;
use systems::satellites::labels::LabelsPlugin;
use systems::earth::EarthPlugin;

fn main() -> bevy::app::AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(OrbitCamPlugin)
        .add_plugins(GlobeUIPlugin)
        .add_plugins(TlePlugin)
        .add_plugins(LabelsPlugin)
        .add_plugins(EarthPlugin)
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0))) // background color
        .add_systems(Startup, setup_scene)
        .run()
}

// set up the main scene
fn setup_scene(mut commands: Commands) {
    // spawn camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-8000.0, 8000.0, 12000.0).looking_at(Vec3::ZERO, Vec3::Y),
        systems::camera::OrbitCamera::new(15000.0, 0.3)
            .with_target(Vec3::ZERO)
            .with_zoom_limits(7000.0, 100000.0)
    ));
}