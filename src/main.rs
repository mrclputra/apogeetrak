use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::reflect::TypePath;
use bevy::asset::Asset;
use bevy::pbr::MaterialPlugin;

// import camera and systems
mod systems;
use systems::camera::{OrbitCamPlugin, OrbitCamera};
use systems::ui::GlobeUIPlugin;
// use systems::satellites::render::TlePlugin;
use systems::satellites::labels::LabelsPlugin;

// WGS84
const EARTH_RADIUS: f32 = 6378.0;
const EARTH_ROTATION_SPEED: f32 = 0.1; // radians per second

// textures
const EARTH_DIFFUSE: &str = "textures/earth_diffuse.jpg";       // base color/albedo
const EARTH_NIGHT: &str = "textures/earth_night.jpg";           // night albedo


// sun direction data for shader
#[derive(ShaderType, Clone, Copy, Debug)]
#[repr(C)]
pub struct SunUniform {
    pub sun_direction: Vec3,
    pub _padding: f32, // ensure 16-byte alignment
}

// earth material with dynamic sun lighting
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct EarthMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub day_texture: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    pub night_texture: Handle<Image>,
    #[uniform(4)]
    pub sun_uniform: SunUniform,
}

impl Material for EarthMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/earth.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

// component markers for our entities
#[derive(Component)]
pub struct Earth;

#[derive(Component)]
pub struct SunLight;

fn main() -> bevy::app::AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MaterialPlugin::<EarthMaterial>::default()) // register material
        .add_plugins(OrbitCamPlugin)
        .add_plugins(GlobeUIPlugin)
        // .add_plugins(TlePlugin)
        .add_plugins(LabelsPlugin)
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_systems(Startup, setup)
        .add_systems(Update, (update_sun_direction, rotate_earth))
        .run()
}

// set up the main scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut earth_materials: ResMut<Assets<EarthMaterial>>,
    asset_server: Res<AssetServer>,
) {
    
    // create Earth sphere with UV
    let mut sphere_mesh = Sphere::new(EARTH_RADIUS).mesh().uv(32, 64);
    // generate normal map tangents !!NEEDED!!
    sphere_mesh.generate_tangents().unwrap();

    // earth textures
    let diffuse_texture = asset_server.load(EARTH_DIFFUSE);
    let night_texture = asset_server.load(EARTH_NIGHT);

    // initial sun direction
    let sun_direction = Vec3::new(1.0, 1.0, 1.0).normalize();

    // create the Earth sphere
    commands.spawn((
        Mesh3d(meshes.add(sphere_mesh)),
        MeshMaterial3d(earth_materials.add(EarthMaterial {
            day_texture: diffuse_texture,
            night_texture: night_texture,
            sun_uniform: SunUniform {
                sun_direction,
                _padding: 0.0,
            },
        })),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_z(std::f32::consts::PI)),
        Name::new("Earth"),
        Earth, // mark for updates
    ));

    // sun light
    commands.spawn((
        DirectionalLight {
            illuminance: 1_500.,
            ..default()
        },
        Transform::from_xyz(50000.0, 50000.0, 50000.0).looking_at(Vec3::ZERO, Vec3::Y),
        SunLight, // mark as sun
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

// keep earth shader in sync with sun position
fn update_sun_direction(
    sun_query: Query<&Transform, (With<SunLight>, Changed<Transform>)>,
    earth_query: Query<&MeshMaterial3d<EarthMaterial>, With<Earth>>,
    mut earth_materials: ResMut<Assets<EarthMaterial>>,
) {
    // only update when sun actually moves
    if let Ok(sun_transform) = sun_query.single() {
        if let Ok(earth_material_handle) = earth_query.single() {
            if let Some(earth_material) = earth_materials.get_mut(&earth_material_handle.0) {
                // get direction from sun to earth (negative forward)
                let sun_direction = -sun_transform.forward();
                earth_material.sun_uniform.sun_direction = sun_direction.into();
            }
        }
    }
}

fn rotate_earth(
    time: Res<Time>,
    mut earth_query: Query<&mut Transform, With<Earth>>,
) {
    if let Ok(mut earth_transform) = earth_query.single_mut() {
        // rotate around Y axis (north-south pole axis), need to change
        let rotation_delta = Quat::from_rotation_z(EARTH_ROTATION_SPEED * time.delta_secs());
        earth_transform.rotation = earth_transform.rotation * rotation_delta;
    }
}

// fn rotate_earth_by_angle(
//     angle_radians: f32,
//     earth_query: &mut Query<&mut Transform, With<Earth>>,
// ) {
//     if let Ok(mut earth_transform) = earth_query.single_mut() {
//         // apply rotation around Y axis
//         let rotation = Quat::from_rotation_y(angle_radians);
//         earth_transform.rotation = earth_transform.rotation * rotation;
//     }
// }