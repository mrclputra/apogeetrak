use bevy::prelude::*;

use crate::systems::satellites::tle::{Satellite, cartesian_to_geodetic, sgp4_to_cartesian, fetch_satellites};

pub struct TlePlugin;

impl Plugin for TlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_satellites);
    }
}

// create a line mesh from a series of points
// 'trail renderer'
fn create_orbit_line_mesh(points: &[Vec3]) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // add all points
    for point in points {
        positions.push([point.x, point.y, point.z]);
    }

    // create line segments connecting said points
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

// render orbital paths given satellites
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

    // render orbit for each satellite
    for satellite in satellites {
        // define orbit mesh resolution here
        // number of vertices
        let orbit_points = satellite.generate_orbit_path(128);

        if !orbit_points.is_empty() {
            let orbit_mesh = create_orbit_line_mesh(&orbit_points);

            commands.spawn((
                Mesh3d(meshes.add(orbit_mesh)),
                MeshMaterial3d(orbit_material.clone()),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        }
    }
}

// load satellite data and spawn them in the world
fn load_satellites(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // function to actually fetch TLE data from fileserver, called on plugin startup
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

            println!(
                "\n=== Fetched {} Satellites ===\n",
                satellites.len()
            );

            // create satellite material
            let satellite_material = materials.add(StandardMaterial {
                base_color: Srgba::hex("#ffffff").unwrap().into(),
                metallic: 0.0,
                perceptual_roughness: 1.0,
                ..default()
            });

            // spawn each satellite as a small sphere, with debug information
            for (i, satellite) in satellites.iter().enumerate() {
                println!("Satellite #{}", i + 1);
                println!("  Name           : {}", satellite.name);
                println!("  NORAD ID       : {}", satellite.norad_id);
                println!("  Intl. ID       : {}", satellite.intl_id);
                println!("  Launch         : {}-{:03} (Year-Launch Number)", satellite.launch_year, satellite.launch_number);
                println!("  Inclination    : {:.2}°", satellite.inclination);
                println!("  Mean Motion    : {:.2} rev/day", satellite.mean_motion);
                println!("  Epoch          : Year {} Day {:.2}", satellite.epoch_year, satellite.epoch_day);
                println!();

                match satellite.calculate() {
                    Some(prediction) => {
                        let (lat, lon, alt) = cartesian_to_geodetic(
                            prediction.position[0],
                            prediction.position[1],
                            prediction.position[2]
                        );
                        println!(
                            "  Current Position: {:.4}°, {:.4}° at {:.1} km altitude",
                            lat, lon, alt
                        );

                        // show velocity magnitude
                        let velocity = (prediction.velocity[0].powi(2) + 
                                       prediction.velocity[1].powi(2) + 
                                       prediction.velocity[2].powi(2)).sqrt();
                        println!("  Velocity: {:.2} km/s", velocity);

                        let position = sgp4_to_cartesian(&prediction);
                        
                        // spawn the satellite
                        commands.spawn((
                            Mesh3d(meshes.add(Sphere::new(50.0).mesh().ico(8).unwrap())),
                            MeshMaterial3d(satellite_material.clone()),
                            Transform::from_translation(position),
                            satellite.clone(),
                        ));
                    }
                    None => {
                        println!("  Current Position: Unable to calculate");
                    }
                }

                println!();
            }

            // render orbits for ALL satellites
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