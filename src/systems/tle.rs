use bevy::prelude::*;
use chrono::{NaiveDate, Utc};
use reqwest::Error;
use reqwest::header::USER_AGENT;
use chrono::DateTime;
use sgp4::Prediction;

use crate::EARTH_RADIUS;

pub struct TlePlugin;

impl Plugin for TlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_satellites)
           .add_systems(Update, update_satellite_positions);
    }
}

// unified satellite component with all data
#[derive(Component, Clone)]
pub struct Satellite {
    name: String,
    norad_id: u32,      // SATCAT, 5-digit number
    intl_id: String,    // Intl ID
    launch_year: u16,   // last 2 digits of year
    launch_number: u16, // launch number of year
    epoch_year: u16,    // last 2 digits of year
    epoch_day: f64,     // includes fractional portion of day
    mean_motion: f64,   // ballistic coefficient
    inclination: f64,   // degrees

    // SGP4 pre-parse data
    elements: sgp4::Elements,
    constants: sgp4::Constants,
}

impl Satellite {
    fn parse(name: &str, line1: &str, line2: &str) -> Option<Self> {
        if line1.len() < 69 || line2.len() < 69 {
            return None;
        }

        // note that in TLE format, element positions are fixed in set
        // https://en.wikipedia.org/wiki/Two-line_element_set

        // extract line 1 data
        let norad_id: u32 = line1[2..7].trim().parse().ok()?;
        let intl_id = line1[9..17].trim().to_string();
        let launch_year: u16 = line1[9..11].parse().ok()?;
        let launch_number: u16 = line1[11..14].trim().parse().ok()?;
        let epoch_year: u16 = line1[18..20].parse().ok()?;
        let epoch_day: f64 = line1[20..32].trim().parse().ok()?;

        // extract line 2 data
        let inclination: f64 = line2[8..16].trim().parse().ok()?;
        let mean_motion: f64 = line2[52..63].trim().parse().ok()?;

        // parse with SGP4
        // !!IMPORTANT!!
        let elements = sgp4::Elements::from_tle(
            Some(name.trim().to_string()),
            line1.as_bytes(),
            line2.as_bytes()
        ).ok()?;
        let constants = sgp4::Constants::from_elements(&elements).ok()?;

        Some(Satellite {
            name: name.trim().to_string(),
            norad_id,
            intl_id,
            launch_year: if launch_year < 57 { 2000 + launch_year } else { 1900 + launch_year },
            launch_number,
            epoch_year: if epoch_year < 57 { 2000 + epoch_year } else { 1900 + epoch_year },
            epoch_day,
            mean_motion,
            inclination,
            elements,
            constants,
        })
    }

    // TODO: combine propagation calculations into one function
    // 'Prediction' type returns (x, y, z) position and (vX, vY, vZ) velocities
    fn calculate(&self) -> Option<Prediction> {
        let minutes_since_epoch = self.calculate_minutes_since_epoch()?;
        self.constants.propagate(sgp4::MinutesSinceEpoch(minutes_since_epoch)).ok()
    }

    // may need to export/store local Prediction type in the future to allow data access and rendering in main process, maybe
    // needs structure review

    // calculate how many minutes has passed since this satellite's epoch
    fn calculate_minutes_since_epoch(&self) -> Option<f64> {
        // find position of satellite at CURRENT datetime
        // this feature should be modified in the future
        let now = Utc::now();

        // create epoch datetime
        let epoch_date = NaiveDate::from_ymd_opt(self.epoch_year as i32, 1, 1)?
            .checked_add_signed(chrono::Duration::days((self.epoch_day - 1.0) as i64))?
            .and_time(
                chrono::NaiveTime::from_num_seconds_from_midnight_opt(
                    (self.epoch_day.fract() * 24.0 * 3600.0) as u32, 0
                )?
            );

        let epoch_datetime = DateTime::<Utc>::from_naive_utc_and_offset(epoch_date, Utc);

        // get difference in minutes
        let duration = now.signed_duration_since(epoch_datetime);
        Some(duration.num_minutes() as f64)
    }
}

// convert Cartesian coordinates (x, y, z) to Geodetic coordinates (lat, lon, alt)
// https://en.wikipedia.org/wiki/Geodetic_coordinates
pub fn cartesian_to_geodetic(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let distance = (x*x + y*y + z*z).sqrt();
    let altitude = distance - EARTH_RADIUS as f64;

    let latitude = (z / distance).asin().to_degrees();
    let longitude = y.atan2(x).to_degrees();

    (latitude, longitude, altitude)
}

// convert SGP4 coordinates to Bevy world coordinates
fn sgp4_to_bevy_pos(prediction: &Prediction) -> Vec3 {
    // SGP4 returns coordinates in kilometers
    Vec3::new(
        prediction.position[0] as f32,
        prediction.position[2] as f32,  // swapped Y and Z
        prediction.position[1] as f32,
    )
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
            println!(
                "\n=== Fetched {} Satellites ===\n",
                satellites.len()
            );

            // create material for satellite markers
            let satellite_material = materials.add(StandardMaterial {
                base_color: Srgba::hex("#ff4444").unwrap().into(),
                metallic: 0.0,
                perceptual_roughness: 0.3,
                ..default()
            });

            // spawn each satellite as a small sphere
            for (i, satellite) in satellites.iter().enumerate().take(7) {
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

                        let position = sgp4_to_bevy_pos(&prediction);
                        
                        commands.spawn((
                            Mesh3d(meshes.add(Sphere::new(100.0).mesh().ico(8).unwrap())),
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

            // spawn all remaining satellites without debug output
            for satellite in satellites.iter().skip(7) {
                if let Some(prediction) = satellite.calculate() {
                    let position = sgp4_to_bevy_pos(&prediction);
                    
                    commands.spawn((
                        Mesh3d(meshes.add(Sphere::new(30.0).mesh().ico(8).unwrap())),
                        MeshMaterial3d(satellite_material.clone()),
                        Transform::from_translation(position),
                        satellite.clone(),
                    ));
                }
            }

            if satellites.len() > 7 {
                println!("... and {} more entries not displayed", satellites.len() - 7);
            }
        }
        Ok(Err(e)) => {
            error!("Failed to fetch TLE data: {}", e);
        }
        Err(_) => {
            error!("Thread panicked while fetching TLE data");
        }
    }
}

// update all satellite positions each frame
fn update_satellite_positions(
    mut satellite_query: Query<(&mut Transform, &Satellite)>,
) {
    for (mut transform, satellite) in satellite_query.iter_mut() {
        // calculate current position
        if let Some(prediction) = satellite.calculate() {
            transform.translation = sgp4_to_bevy_pos(&prediction);
        }
    }
}

// async function to actually fetch and parse the satellite data
// QA: should i combine this with "load_satellites()"?
async fn fetch_satellites() -> Result<Vec<Satellite>, Error> {
    // call fileserver here
    let url = "https://celestrak.org/NORAD/elements/gnss.txt";

    let response = reqwest::Client::new()
        .get(url)
        .header(USER_AGENT, "apogeetrak-satellite-tracker")
        .send()
        .await?
        .text()
        .await?;

    // parse the TLE data
    let lines: Vec<&str> = response.lines().collect();
    let mut satellites: Vec<Satellite> = Vec::new();

    for chunk in lines.chunks(3) {
        if chunk.len() == 3 {
            if let Some(satellite) = Satellite::parse(chunk[0], chunk[1], chunk[2]) {
                satellites.push(satellite);
            }
        }
    }

    Ok(satellites)
}