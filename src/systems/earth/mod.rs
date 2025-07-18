use bevy::prelude::*;

pub mod materials;
pub mod mesh;
pub mod uv;

use mesh::generate_face;
use materials::{EarthMaterial, CloudMaterial, SunUniform};
use crate::{config::{
    EARTH_CLOUDS_TEXTURE, EARTH_DIFFUSE_TEXTURE, EARTH_NIGHT_TEXTURE, EARTH_OCEAN_MASK_TEXTURE,
    CLOUD_RADIUS,  EARTH_ROTATION_SPEED
}, Sun};

pub struct EarthPlugin;

impl Plugin for EarthPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<EarthMaterial>::default())
            .add_plugins(MaterialPlugin::<CloudMaterial>::default())
            .add_systems(Startup, start)
            .add_systems(Update, (update_shaders, rotate));
    }
}

// grounded tag
#[derive(Component)]
pub struct Grounded;

fn start(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut earth_materials: ResMut<Assets<EarthMaterial>>,
    mut cloud_materials: ResMut<Assets<CloudMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let _earth = commands
        .spawn((
            Grounded,
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    // sun direction
    let sun_direction = Vec3::new(1.0, 1.0, 1.0).normalize();

    // load textures
    let diffuse_texture = asset_server.load(EARTH_DIFFUSE_TEXTURE);
    let night_texture = asset_server.load(EARTH_NIGHT_TEXTURE);
    let cloud_texture = asset_server.load(EARTH_CLOUDS_TEXTURE);
    let ocean_mask_texture = asset_server.load(EARTH_OCEAN_MASK_TEXTURE);

    // generate earth mesh
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
                    generate_face(direction, 22, offset.0, offset.1),
                )),
                MeshMaterial3d(earth_materials.add(EarthMaterial {
                    day_texture: diffuse_texture.clone(),
                    night_texture: night_texture.clone(),
                    sun_uniform: SunUniform {
                        direction: sun_direction,
                        _padding: 0.0,
                    },
                    ocean_mask: ocean_mask_texture.clone(),
                })),
                // Transform::from_scale(Vec3::splat(EARTH_RADIUS)),
                // GlobalTransform::default(),
                // Grounded,
            ))
            .insert(ChildOf(_earth));
        }
    }

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
            cloud_opacity: 0.5, // tweak this for cloud visibility
        })),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        // Grounded,
    ))
    .insert(ChildOf(_earth));
}

// update shaders
fn update_shaders(
    sun_query: Query<&Transform, (With<Sun>, Changed<Transform>)>,
    earth_query: Query<&MeshMaterial3d<EarthMaterial>, With<Grounded>>,
    cloud_query: Query<&MeshMaterial3d<CloudMaterial>, With<Grounded>>,
    mut earth_materials: ResMut<Assets<EarthMaterial>>,
    mut cloud_materials: ResMut<Assets<CloudMaterial>>,
) {
    if let Ok(sun_transform) = sun_query.single() {
        let sun_direction = -sun_transform.forward();
        
        // update earth material uniforms
        if let Ok(earth_material_handle) = earth_query.single() {
            if let Some(earth_material) = earth_materials.get_mut(&earth_material_handle.0) {
                earth_material.sun_uniform.direction = sun_direction.into();
            }
        }
        
        // update cloud material uniforms
        if let Ok(cloud_material_handle) = cloud_query.single() {
            if let Some(cloud_material) = cloud_materials.get_mut(&cloud_material_handle.0) {
                cloud_material.sun_uniform.direction = sun_direction.into();
            }
        }
    }
}

// rotate earth
fn rotate(
    time: Res<Time>,
    mut earth_query: Query<&mut Transform, With<Grounded>>
) {
    let delta_rotation = Quat::from_rotation_y(EARTH_ROTATION_SPEED * time.delta_secs());

    if let Ok(mut transform) = earth_query.single_mut() {
        transform.rotation = transform.rotation * delta_rotation;
    }
}