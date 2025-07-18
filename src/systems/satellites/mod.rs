use bevy::prelude::*;

pub mod tle;
pub mod labels;

pub use tle::{Satellite, fetch_satellites};
use labels::setup_labels;

// main satellite plugin
// combined TLE and labeling functionality
pub struct SatellitePlugin;

impl Plugin for SatellitePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (setup_satellites, setup_labels))
            .add_systems(Update, labels::update_labels);
    }
}

// create line mesh loop from a series of points (wraps)
// my 'trail renderer
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

    // render orbit, all satellites
    for satellite in satellites {
        // define orbit mesh resolution here (number of vertices)
        let orbit_points = satellite.generate_orbit_path(256);

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

// called on startup
fn setup_satellites(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

            // spawn satellites
            for satellite in &satellites {
                // spawn satellite in 3D
                let position = satellite.current_position();
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

                // // print current ECI position
                // let pos = satellite.current_position();
                // println!(
                //     "  ECI         : {:.2} km, {:.2} km, {:.2} km", pos.x, pos.y, pos.z
                // );

                // // print current geodetic position
                // let (lat, lon, alt) = satellite.current_geodetic_position();
                // println!(
                //     "  Geo         : {:.4}°, {:.4}° at {:.1} km",
                //     lat, lon, alt
                // );

                // // print current velocity
                // let (vx, vy, vz) = satellite.current_velocity();
                // let speed = (vx.powi(2) + vy.powi(2) + vz.powi(2)).sqrt();
                // println!(
                //     "  Velocity    : {:.2} km/s", speed
                // );

                // println!();
            }

            render_orbits(&satellites, &mut commands, &mut meshes, &mut materials);
        }
        Ok(Err(e)) => {
            error!("Failed to fetch TLE data: {}", e);
        }
        Err(_) => {
            error!("Thread panicked while fetching TLE data");
        }
    }
}