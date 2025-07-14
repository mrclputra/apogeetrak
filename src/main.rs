use bevy::prelude::*;

// import camera and systems
mod systems;
use systems::camera::{OrbitCamPlugin, OrbitCamera};
use systems::ui::GlobeUIPlugin;
use systems::satellites::render::TlePlugin;
use systems::satellites::labels::LabelsPlugin;

// WGS84
const EARTH_RADIUS: f32 = 6378.0;

// textures
const EARTH_DIFFUSE: &str = "textures/earth_diffuse.jpg";       // base color/albedo
const EARTH_NORMAL: &str = "textures/earth_normal.jpg";         // surface detail normals
const EARTH_SPECULAR: &str = "textures/earth_specular.jpg";     // roughness map (water=smooth, land=rough)
// const EARTH_EMISSIVE: &str = "textures/earth_emissive.jpg";     // night lights
// const EARTH_OCCLUSION: &str = "textures/earth_occlusion.jpg";   // ambient occlusion


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
    asset_server: Res<AssetServer>,
) {
    
    // create Earth sphere with UV
    let mut sphere_mesh = Sphere::new(EARTH_RADIUS).mesh().uv(32, 64);
    // generate normal map tangents
    sphere_mesh.generate_tangents().unwrap();

    // earth textures
    let diffuse_texture = asset_server.load(EARTH_DIFFUSE);
    // let normal_texture = asset_server.load(EARTH_NORMAL);
    // let specular_texture = asset_server.load(EARTH_SPECULAR);

    // create the Earth sphere
    commands.spawn((
        Mesh3d(meshes.add(sphere_mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(diffuse_texture),
            base_color: Color::WHITE, // keep this way, no tint
            
            // normal map
            // normal_map_texture: Some(normal_texture),

            // // specular texture
            // metallic_roughness_texture: Some(specular_texture),
            metallic: 0.0,
            perceptual_roughness: 1.0,

            // other material settings
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
            unlit: false, // keep pbr lighting

            ..default()  
        })),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_z(std::f32::consts::PI)),
        Name::new("Earth")
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