use bevy::prelude::*;

pub mod tle;
pub mod labels;

pub use tle::{Satellite, fetch_satellites};
use labels::setup_labels;
use crate::systems::ui::TimeState;

/// Main satellite plugin
pub struct SatellitePlugin;

impl Plugin for SatellitePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (
                setup_labels,
                start.after(crate::systems::ui::start),
            ))
            .add_systems(Update, (
                update_positions,
                labels::update_labels,
            ));
    }
}

// create line mesh from a series of points
// wraps around
fn create_trail_mesh(points: &[Vec3]) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // add points
    for point in points {
        positions.push([point.x, point.y, point.z]);
    }

    // create line segments
    for i in 0..points.len() {
        let next_i = (i + 1) % points.len(); // wrap
        indices.push(i as u32);
        indices.push(next_i as u32);
    }

    // build the mesh
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::LineList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    mesh
}

fn render_orbits(
    satellites: &[Satellite],
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // create orbit material, reusable
    let orbit_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.05),
        alpha_mode: AlphaMode::Blend,
        unlit: true, // glowing effect
        ..default()
    });

    // render orbit for all satellites
    for satellite in satellites {
        if !satellite.orbit_path.is_empty() {
            // extract just the positions from the orbit points
            let orbit_positions: Vec<Vec3> = satellite.orbit_path
                .iter()
                .map(|point| point.position)
                .collect();

            let orbit_mesh = create_trail_mesh(&orbit_positions);

            commands.spawn((
                Mesh3d(meshes.add(orbit_mesh)),
                MeshMaterial3d(orbit_material.clone()),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        }
    }
}

// update satellite positions
fn update_positions(
    time_state: Res<TimeState>,
    mut satellite_query: Query<(&Satellite, &mut Transform)>,
) {
    // println!("{}", time_state.sim_time);

    for (satellite, mut transform) in satellite_query.iter_mut() {
        let new_position = satellite.get_position(time_state.sim_time);
        transform.translation = new_position;
    }
}

// called on startup
// setup satellites, meshes, and stuff
fn start(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time_state: Res<TimeState>,
) {
    // fetch TLE data
    let task = std::thread::spawn(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            fetch_satellites().await
        })
    });

    // block process briefly to get data
    // need to implement proper async handling in the future
    match task.join() {
        Ok(Ok(mut satellites)) => {
            // create satellite material
            let satellite_material = materials.add(StandardMaterial {
                base_color: Srgba::hex("#ffffff").unwrap().into(),
                metallic: 0.0,
                perceptual_roughness: 1.0,
                ..default()
            });

            // generate orbit paths
            for satellite in &mut satellites {
                satellite.generate_orbit_path(256, time_state.sim_time);
                
                // debug: print orbit info
                println!("Generated orbit for {}: {:.1} minutes, {} points", 
                    satellite.name(), 
                    satellite.orbit_duration_m, 
                    satellite.orbit_path.len());
            }

            // spawn satellites at initial positions
            for satellite in &satellites {
                let position = satellite.get_position(time_state.sim_time);
                
                commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(50.0).mesh().ico(8).unwrap())),
                    MeshMaterial3d(satellite_material.clone()),
                    Transform::from_translation(position),
                    satellite.clone(),
                ));
            }

            render_orbits(&satellites, &mut commands, &mut meshes, &mut materials);
        }
        Ok(Err(e)) => {
            error!("Failed to fetch TLE data: {:?}", e);
        }
        Err(_) => {
            error!("Thread panicked while fetching TLE data");
        }
    }
}