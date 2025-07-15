use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::reflect::TypePath;
use bevy::asset::Asset;
use bevy::pbr::MaterialPlugin;

pub mod materials;
use materials::{EarthMaterial, CloudMaterial, SunUniform};

// components (tags)
#[derive(Component)]
pub struct Earth;

#[derive(Component)]
pub struct Clouds;

#[derive(Component)]
pub struct SunLight;

// earth constants
// QA: move to a global constants module later
pub const EARTH_RADIUS: f32 = 6378.0;
pub const CLOUD_RADIUS: f32 = 6428.0;
pub const EARTH_ROTATION_SPEED: f32 = 0.1;

// texture paths
const EARTH_DIFFUSE: &str = "textures/earth_diffuse.jpg";
const EARTH_NIGHT: &str = "textures/earth_night.jpg";
const EARTH_CLOUDS: &str = "textures/earth_clouds.jpg";

pub struct EarthPlugin;

impl Plugin for EarthPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<EarthMaterial>::default())
            .add_plugins(MaterialPlugin::<CloudMaterial>::default())
            .add_systems(Startup, setup)
            .add_systems(Update, (update_sun, rotate_earth));
    }
}

// setup
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut earth_materials: ResMut<Assets<EarthMaterial>>,
    mut cloud_materials: ResMut<Assets<CloudMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // create earth sphere
    let mut earth_sphere = Sphere::new(EARTH_RADIUS).mesh().uv(32, 64);
    earth_sphere.generate_tangents().unwrap(); // needed for proper normal mapping

    // create cloud sphere
    let mut cloud_sphere = Sphere::new(CLOUD_RADIUS).mesh().uv(32, 64);
    cloud_sphere.generate_tangents().unwrap();

    // load textures
    let diffuse_texture = asset_server.load(EARTH_DIFFUSE);
    let night_texture = asset_server.load(EARTH_NIGHT);
    let cloud_texture = asset_server.load(EARTH_CLOUDS);

    // sun direction
    let sun_direction = Vec3::new(1.0, 1.0, 1.0).normalize();

    // spawn the earth sphere
    commands.spawn((
        Mesh3d(meshes.add(earth_sphere)),
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
        Earth, // component marker for systems
    ));

    // spawn the cloud sphere
    commands.spawn((
        Mesh3d(meshes.add(cloud_sphere)),
        MeshMaterial3d(cloud_materials.add(CloudMaterial {
            cloud_texture,
            sun_uniform: SunUniform {
                sun_direction,
                _padding: 0.0,
            },
            cloud_opacity: 0.7, // tweak this for cloud visibility
        })),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_z(std::f32::consts::PI)),
        Name::new("Clouds"),
        Clouds, // component marker
    ));

    // spawn the sun light
    commands.spawn((
        DirectionalLight {
            illuminance: 1_500.,
            ..default()
        },
        Transform::from_xyz(50000.0, 50000.0, 50000.0).looking_at(Vec3::ZERO, Vec3::Y),
        SunLight, // component marker for sun tracking
    ));
}

// update shaders based on sunlight
fn update_sun(
    sun_query: Query<&Transform, (With<SunLight>, Changed<Transform>)>,
    earth_query: Query<&MeshMaterial3d<EarthMaterial>, With<Earth>>,
    cloud_query: Query<&MeshMaterial3d<CloudMaterial>, With<Clouds>>,
    mut earth_materials: ResMut<Assets<EarthMaterial>>,
    mut cloud_materials: ResMut<Assets<CloudMaterial>>,
) {
    // only update when the sun actually moves
    if let Ok(sun_transform) = sun_query.single() {
        let sun_direction = -sun_transform.forward();
        
        // update earth material uniforms
        if let Ok(earth_material_handle) = earth_query.single() {
            if let Some(earth_material) = earth_materials.get_mut(&earth_material_handle.0) {
                earth_material.sun_uniform.sun_direction = sun_direction.into();
            }
        }
        
        // update cloud material uniforms
        if let Ok(cloud_material_handle) = cloud_query.single() {
            if let Some(cloud_material) = cloud_materials.get_mut(&cloud_material_handle.0) {
                cloud_material.sun_uniform.sun_direction = sun_direction.into();
            }
        }
    }
}

// rotate earth and clouds
fn rotate_earth(
    time: Res<Time>,
    mut earth_query: Query<&mut Transform, Or<(With<Earth>, With<Clouds>)>>,
) {
    let rotation_delta = Quat::from_rotation_z(EARTH_ROTATION_SPEED * time.delta_secs());
    
    for mut transform in earth_query.iter_mut() {
        transform.rotation = transform.rotation * rotation_delta;
    }
}

// pub fn rotate_earth_by_angle(
//     angle_radians: f32,
//     earth_query: &mut Query<&mut Transform, With<Earth>>,
// ) {
//     if let Ok(mut earth_transform) = earth_query.single_mut() {
//         let rotation = Quat::from_rotation_y(angle_radians);
//         earth_transform.rotation = earth_transform.rotation * rotation;
//     }
// }