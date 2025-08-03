//! earth/mod.rs
//! 
//! Main earth renderer plugin
//! Handles mesh generation and dynamic updates of earth-related entities:
//! - The planet's surface mesh from displacement map
//! - Precomputed normal maps
//! - Atmospheric scattering effects
//! - Cloud layer
//! - Real-time lighting updates (for future seasons implementation)

use bevy::prelude::*;

pub mod materials;
pub mod mesh;
pub mod uv;
pub mod normal;

use mesh::generate_face;
use materials::{EarthMaterial, AtmosphereMaterial, CloudMaterial, SunUniform, AtmosphereUniform};
use normal::{generate_normal_map, save_image_as_png};
use crate::{config::{
    ATMOSPHERE_RADIUS, CLOUD_RADIUS, EARTH_CLOUDS_TEXTURE, EARTH_DIFFUSE_TEXTURE, EARTH_DISPLACEMENT_TEXTURE, EARTH_NIGHT_TEXTURE, EARTH_OCEAN_MASK_TEXTURE, EARTH_ROTATION_SPEED, EARTH_SPECULAR_TEXTURE, MIE_COEFF, RAYLEIGH_COEFF, SUN_INTENSITY, USE_SAVED_NORMAL_MAP, SAVED_NORMAL_MAP_PATH
}, Sun};
use crate::systems::time::TimeState;

pub struct EarthPlugin;

impl Plugin for EarthPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<EarthMaterial>::default())
            .add_plugins(MaterialPlugin::<AtmosphereMaterial>::default())
            .add_plugins(MaterialPlugin::<CloudMaterial>::default())
            .add_systems(Startup, setup)
            .add_systems(Update, (
                generate_earth_faces.run_if(resource_exists::<EarthData>),
                update_shaders, 
                rotate
            ));
    }
}

/// earth tag
#[derive(Component)]
pub struct Earth;

/// atmosphere tag
#[derive(Component)]
pub struct Atmosphere;

/// cloud tag
#[derive(Component)]
pub struct Clouds;

/// holds everything needed for earth generation including normal map
#[derive(Resource)]
struct EarthData {
    displacement_handle: Handle<Image>,
    normal_map_handle: Option<Handle<Image>>, // generated normal map
    earth_entity: Entity,
    earth_material: Option<Handle<EarthMaterial>>, // created after normal map generation
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut atmosphere_materials: ResMut<Assets<AtmosphereMaterial>>,
    mut cloud_materials: ResMut<Assets<CloudMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // sun direction
    let sun_direction = Vec3::new(1.0, 1.0, 1.0).normalize();

    // load textures
    let displacement_handle = asset_server.load(EARTH_DISPLACEMENT_TEXTURE);
    let cloud_texture = asset_server.load(EARTH_CLOUDS_TEXTURE);

    // create earth entity
    let earth_entity = commands
        .spawn((
            Earth,
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    // create atmosphere
    let mut atmosphere_sphere = Sphere::new(ATMOSPHERE_RADIUS * 1.2).mesh().uv(16, 16);
    atmosphere_sphere.generate_tangents().unwrap();

    commands.spawn((
        Mesh3d(meshes.add(atmosphere_sphere)),
        MeshMaterial3d(atmosphere_materials.add(AtmosphereMaterial {
            atmosphere_uniform: AtmosphereUniform {
                sun_direction, 
                camera_position: Vec3::ZERO, // will be updated on runtime
                rayleigh_coeff: Vec3::from(RAYLEIGH_COEFF),
                mie_coeff: MIE_COEFF,
                sun_intensity: SUN_INTENSITY,
                atmosphere_radius: ATMOSPHERE_RADIUS,
                _padding: 0.0,
            },
        })),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Atmosphere,
    ));

    // create cloud sphere
    let mut cloud_sphere = Sphere::new(CLOUD_RADIUS).mesh().uv(32, 64);
    cloud_sphere.generate_tangents().unwrap();

    // spawn the cloud sphere
    commands.spawn((
        Mesh3d(meshes.add(cloud_sphere)),
        MeshMaterial3d(cloud_materials.add(CloudMaterial {
            cloud_texture,
            sun_uniform: SunUniform {
                direction: sun_direction,
                _padding: 0.0,
            },
            cloud_opacity: 0.7,
        })),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Clouds,
    ))
    .insert(ChildOf(earth_entity));

    // store data for mesh and material generation once displacement loads
    commands.insert_resource(EarthData {
        displacement_handle,
        normal_map_handle: None,
        earth_entity,
        earth_material: None,
    });
}

/// generate earth faces
/// only runs after displacement map is loaded
fn generate_earth_faces(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut earth_materials: ResMut<Assets<EarthMaterial>>,
    mut earth_data: ResMut<EarthData>,
    asset_server: Res<AssetServer>,
) {
    info!("Generating earth mesh...");

    // check if displacement map is loaded
    let displacement_image = {
        let maybe_image = images.get(&earth_data.displacement_handle);
        if maybe_image.is_none() {
            return; // wait for image to load
        }
        maybe_image.unwrap().clone()
    };

    // handle normal map
    if earth_data.normal_map_handle.is_none() {
        let normal_map_handle = if USE_SAVED_NORMAL_MAP {
            // try to load the saved normal map first
            match load_saved_normal_map(&asset_server) {
                Some(handle) => {
                    info!("Using normal maps...");
                    handle
                }
                None => {
                    // fallback
                    info!("No normal maps found, creating new ones...");
                    let normal_map_image = generate_normal_map(&displacement_image);
                    save_image_as_png(&normal_map_image, &format!("assets/{}", SAVED_NORMAL_MAP_PATH));
                    info!("Normal maps generated, saved to: assets/{}", SAVED_NORMAL_MAP_PATH);
                    images.add(normal_map_image)
                }
            }
        } else {
            // always generate fresh
            info!("Creating normal maps...");
            let normal_map_image = generate_normal_map(&displacement_image);
            
            save_image_as_png(&normal_map_image, &format!("assets/{}", SAVED_NORMAL_MAP_PATH));
            info!("Normal map generated");
            images.add(normal_map_image)
        };
        
        earth_data.normal_map_handle = Some(normal_map_handle);
    }

    // create earth material
    if earth_data.earth_material.is_none() {
        let sun_direction = Vec3::new(1.0, 1.0, 1.0).normalize();
        
        // load remaining textures
        let diffuse_texture = asset_server.load(EARTH_DIFFUSE_TEXTURE);
        let night_texture = asset_server.load(EARTH_NIGHT_TEXTURE);
        let ocean_mask_texture = asset_server.load(EARTH_OCEAN_MASK_TEXTURE);
        let specular_texture = asset_server.load(EARTH_SPECULAR_TEXTURE);

        let earth_material = earth_materials.add(EarthMaterial {
            day_texture: diffuse_texture,
            night_texture,
            ocean_mask: ocean_mask_texture,
            specular_map: specular_texture,
            normal_map: earth_data.normal_map_handle.clone().unwrap(),
            sun_uniform: SunUniform {
                direction: sun_direction,
                _padding: 0.0,
            },
        });

        earth_data.earth_material = Some(earth_material);
    }

    // generate earth mesh faces
    if let Some(ref earth_material) = earth_data.earth_material {
        let faces = vec![
            Vec3::X,        // right
            Vec3::NEG_X,    // left
            Vec3::Y,        // top
            Vec3::NEG_Y,    // bottom
            Vec3::Z,        // front
            Vec3::NEG_Z,    // back
        ];

        let offsets = vec![(0.0, 0.0), (0.0, 1.0), (1.0, 0.0), (1.0, 1.0)];

        for direction in faces {
            for offset in &offsets {
                commands.spawn((
                    Mesh3d(meshes.add(
                        // you can modify displacement resolution here
                        generate_face(
                            direction, 
                            84,
                            offset.0, 
                            offset.1,
                            Some(&displacement_image),
                        ),
                    )),
                    MeshMaterial3d(earth_material.clone()),
                ))
                .insert(ChildOf(earth_data.earth_entity));
            }
        }

        // cleanup
        commands.remove_resource::<EarthData>();

        info!("Earth mesh generation complete...");
    }
}

/// helper function to try loading a saved normal map
fn load_saved_normal_map(asset_server: &AssetServer) -> Option<Handle<Image>> {
    // check if the file exists before trying to load it
    if std::path::Path::new(&format!("assets/{}", SAVED_NORMAL_MAP_PATH)).exists() {
        Some(asset_server.load(SAVED_NORMAL_MAP_PATH))
    } else {
        None
    }
}

/// update shaders
fn update_shaders(
    sun_query: Query<&Transform, With<Sun>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Sun>)>,
    earth_query: Query<&MeshMaterial3d<EarthMaterial>, With<Earth>>,
    atmosphere_query: Query<&MeshMaterial3d<AtmosphereMaterial>, With<Atmosphere>>,
    cloud_query: Query<&MeshMaterial3d<CloudMaterial>, With<Earth>>,
    mut earth_materials: ResMut<Assets<EarthMaterial>>,
    mut atmosphere_materials: ResMut<Assets<AtmosphereMaterial>>,
    mut cloud_materials: ResMut<Assets<CloudMaterial>>,
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

    // update cloud material uniforms
    if let Ok(cloud_material_handle) = cloud_query.single() {
        if let Some(cloud_material) = cloud_materials.get_mut(&cloud_material_handle.0) {
            cloud_material.sun_uniform.direction = sun_direction.into();
        }
    }
}

/// rotate earth
fn rotate(
    time_state: Res<TimeState>,
    mut earth_query: Query<&mut Transform, With<Earth>>,
) {
    if !time_state.is_paused {
        let rotation_speed = EARTH_ROTATION_SPEED * time_state.speed_mult as f32;

        // keep rotation smooth
        let delta_rotation = Quat::from_rotation_y(rotation_speed * (1.0 / 60.0)); // 60fps equiv

        // rotate earth
        if let Ok(mut transform) = earth_query.single_mut() {
            transform.rotation = transform.rotation * delta_rotation;
        }
    }
}