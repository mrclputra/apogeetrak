use bevy::prelude::*;

pub mod tle;
pub mod labels;

pub use tle::{Satellite, fetch_satellites};
use labels::setup_labels;
use crate::systems::ui::TimeState; // add this import

// main satellite plugin
// combined TLE and labeling functionality
pub struct SatellitePlugin;

impl Plugin for SatellitePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (
                setup_labels,
                setup_satellites.after(crate::systems::ui::start),
            ));
            // .add_systems(Update, (
            //     update_positions,
            //     labels::update_labels
            // ));
    }
}

// create line mesh loop from a series of points (wraps)
// my 'trail renderer'
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
    sim_time: chrono::DateTime<chrono::Utc>,
) {
    // create orbit material, reusable
    let orbit_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.05),
        alpha_mode: AlphaMode::Blend,
        unlit: true, // glowing effect
        ..default()
    });

    // render orbit, all satellites
    for satellite in satellites {
        // define orbit mesh resolution here (number of vertices)
        let orbit_points = satellite.generate_orbit_path(256, sim_time);

        if !orbit_points.is_empty() {
            let orbit_mesh = create_trail_mesh(&orbit_points);

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
    for (satellite, mut transform) in satellite_query.iter_mut() {
        let new_position = satellite.position_at_time(time_state.sim_time);
        transform.translation = new_position;
    }
}

// called on startup, to setup satellite objects initially
fn setup_satellites(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time_state: Res<TimeState>,
) {
    // fetch TLE data from Celestrak fileserver
    // QA: should this be adapted for APIs?
    let task = std::thread::spawn(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            fetch_satellites().await
        })
    });

    // block process briefly to get data
    // need to implement proper async handling in the future
    match task.join() {
        Ok(Ok(satellites)) => {
            // satellites.truncate(1);

            // create satellite material
            let satellite_material = materials.add(StandardMaterial {
                base_color: Srgba::hex("#ffffff").unwrap().into(),
                metallic: 0.0,
                perceptual_roughness: 1.0,
                ..default()
            });

            // spawn satellites at initial positions
            for satellite in &satellites {
                // spawn satellites at initial time
                let position = satellite.position_at_time(time_state.sim_time);
                commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(50.0).mesh().ico(8).unwrap())),
                    MeshMaterial3d(satellite_material.clone()),
                    Transform::from_translation(position),
                    satellite.clone(),
                ));

                // println!("  Name        : {}", satellite.name());
                // println!("  NORAD ID    : {}", satellite.norad_id());
                // println!("  Intl ID     : {}", satellite.intl_id());
                // println!("  Inclination : {:.2}", satellite.inclination());
                // println!("  Mean Motion : {:.2}", satellite.mean_motion());
                // println!("  Epoch       : Year {} Day {:.2}", satellite.epoch_datetime().year(), satellite.epoch_datetime().day());
                // println!();

                // // print ECI position at sim time
                // let pos = satellite.position_at_time(time_state.sim_time);
                // println!(
                //     "  ECI         : {:.2} km, {:.2} km, {:.2} km", pos.x, pos.y, pos.z
                // );

                // // print geodetic position at sim time
                // let (lat, lon, alt) = satellite.geodetic_position_at_time(time_state.sim_time);
                // println!(
                //     "  Geo         : {:.4}°, {:.4}° at {:.1} km",
                //     lat, lon, alt
                // );

                // // print velocity at sim time
                // let (vx, vy, vz) = satellite.velocity_at_time(time_state.sim_time);
                // let speed = (vx.powi(2) + vy.powi(2) + vz.powi(2)).sqrt();
                // println!(
                //     "  Velocity    : {:.2} km/s", speed
                // );

                // println!();
            }

            render_orbits(&satellites, &mut commands, &mut meshes, &mut materials, time_state.sim_time);
        }
        Ok(Err(e)) => {
            error!("Failed to fetch TLE data: {}", e);
        }
        Err(_) => {
            error!("Thread panicked while fetching TLE data");
        }
    }
}