use bevy::prelude::*;

pub mod materials;
pub mod mesh;
pub mod uv;

use mesh::generate_earth_mesh;
use materials::{EarthMaterial, AtmosphereMaterial, SunUniform, AtmosphereUniform};
use crate::{config::{
    ATMOSPHERE_RADIUS, EARTH_DIFFUSE_TEXTURE, EARTH_DISPLACEMENT_TEXTURE, EARTH_NIGHT_TEXTURE, EARTH_OCEAN_MASK_TEXTURE, EARTH_RADIUS, EARTH_ROTATION_SPEED, EARTH_SPECULAR_TEXTURE, MIE_COEFF, RAYLEIGH_COEFF, SUN_INTENSITY
}, Sun};

pub struct EarthPlugin;

impl Plugin for EarthPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(MaterialPlugin::<EarthMaterial>::default())
            .add_plugins(MaterialPlugin::<AtmosphereMaterial>::default())
            .add_systems(Startup, start)
            .add_systems(Update, (
                update_shaders,
                update,
                rotate
            ));
    }
}

// atmosphere tag
#[derive(Component)]
pub struct Atmosphere;

// earth tag
#[derive(Component)]
pub struct Earth;

// displacement placeholder replacement
#[derive(Component)]
struct Placeholder(Handle<Image>);

fn start(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut earth_material: ResMut<Assets<EarthMaterial>>,
    mut atmosphere_material: ResMut<Assets<AtmosphereMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // sun direction
    let _sun_direction = Vec3::new(1.0, 1.0, 1.0).normalize();

    // load all textures
    // load all textures
    let _diffuse_texture = asset_server.load(EARTH_DIFFUSE_TEXTURE);
    let _night_texture = asset_server.load(EARTH_NIGHT_TEXTURE);
    let _ocean_mask_texture = asset_server.load(EARTH_OCEAN_MASK_TEXTURE);
    let _specular_texture = asset_server.load(EARTH_SPECULAR_TEXTURE);
    let _displacement_handle = asset_server.load(EARTH_DISPLACEMENT_TEXTURE);

    // create earth material
    let earth_material = earth_material.add(EarthMaterial {
        day_texture: _diffuse_texture,
        night_texture: _night_texture,
        ocean_mask: _ocean_mask_texture,
        specular_map: _specular_texture,
        sun_uniform: SunUniform {
            direction: _sun_direction,
            _padding: 0.0,
        }
    });

    // create atmosphere
    let mut atmosphere_mesh = Sphere::new(ATMOSPHERE_RADIUS * 1.4).mesh().uv(32, 64);
    atmosphere_mesh.generate_tangents().unwrap();

    commands.spawn((
        Mesh3d(meshes.add(atmosphere_mesh)),
        MeshMaterial3d(atmosphere_material.add(AtmosphereMaterial {
            atmosphere_uniform: AtmosphereUniform {
                sun_direction: _sun_direction,
                camera_position: Vec3::ZERO, // will be updated
                rayleigh_coeff: Vec3::from(RAYLEIGH_COEFF),
                mie_coeff: MIE_COEFF,
                sun_intensity: SUN_INTENSITY,
                atmosphere_radius: ATMOSPHERE_RADIUS,
                _padding: 0.0,
            },
        })),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Atmosphere
    ));

    // create earth
    let mut placeholder_mesh = Sphere::new(EARTH_RADIUS).mesh().uv(64, 32);
    placeholder_mesh.generate_tangents().unwrap();

    commands.spawn((
        Mesh3d(meshes.add(placeholder_mesh)),
        MeshMaterial3d(earth_material),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
        Earth,
        Placeholder(_displacement_handle),
    ));
}

// update everything else
fn update(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    images: Res<Assets<Image>>,
    mut earth_query: Query<(Entity, &mut Mesh3d, &Placeholder), With<Earth>>
) {
    for (entity, mut mesh_handle, placeholder) in earth_query.iter_mut() {
        // check if displacement texture is loaded
        if let Some(displacement_image) = images.get(&placeholder.0) {
            let earth_mesh = generate_earth_mesh(16, Some(displacement_image));
            
            // replace the mesh
            mesh_handle.0 = meshes.add(earth_mesh);
            
            // remove marker component
            commands.entity(entity).remove::<Placeholder>();
        }
    }
}

// update shaders
fn update_shaders(
    sun_query: Query<&Transform, (With<Sun>, Changed<Transform>)>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Sun>)>,
    earth_query: Query<&MeshMaterial3d<EarthMaterial>, With<Earth>>,
    atmosphere_query: Query<&MeshMaterial3d<AtmosphereMaterial>, With<Atmosphere>>,
    mut earth_materials: ResMut<Assets<EarthMaterial>>,
    mut atmosphere_materials: ResMut<Assets<AtmosphereMaterial>>,
) {
    let sun_direction = if let Ok(sun_transform) = sun_query.single() {
        -sun_transform.forward()
    } else {
        Dir3::new(Vec3::new(1.0, 1.0, 1.0).normalize()).unwrap() // fallback
    };

    let camera_position = if let Ok(camera_transform) = camera_query.single() {
        camera_transform.translation
    } else {
        Vec3::new(0.0, 0.0, 15000.0) // fallback
    };
    
    // update earth material uniforms
    if let Ok(earth_material_handle) = earth_query.single() {
        if let Some(earth_material) = earth_materials.get_mut(&earth_material_handle.0) {
            earth_material.sun_uniform.direction = sun_direction.into();
        }
    }
    
    // update atmosphere material uniforms
    if let Ok(atmosphere_material_handle) = atmosphere_query.single() {
        if let Some(atmosphere_material) = atmosphere_materials.get_mut(&atmosphere_material_handle.0) {
            atmosphere_material.atmosphere_uniform.sun_direction = sun_direction.into();
            atmosphere_material.atmosphere_uniform.camera_position = camera_position;
        }
    }
}

// rotate earth
fn rotate(
    time: Res<Time>,
    mut earth_query: Query<&mut Transform, With<Earth>>
) {
    let delta_rotation = Quat::from_rotation_y(EARTH_ROTATION_SPEED * time.delta_secs());

    if let Ok(mut transform) = earth_query.single_mut() {
        transform.rotation = transform.rotation * delta_rotation;
    }
}