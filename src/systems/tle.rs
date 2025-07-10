use bevy::prelude::*;
use reqwest::Error;
use reqwest::header::USER_AGENT;
use chrono::{DateTime, Utc, NaiveDate};

pub struct TlePlugin;

impl Plugin for TlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, fetch_data);
    }
}

struct Satellite {
    name: String,
    norad_id: u32,
    intl_id: String,
    launch_year: u16,
    launch_number: u16,
    epoch_year: u16,
    epoch_day: f64,
    mean_motion: f64,
    inclination: f64,

    // SPG4 data
    elements: sgp4::Elements,
    constants: sgp4::Constants,
}

impl Satellite {
    fn parse(name: &str, line1: &str, line2: &str) -> Option<Self> {
        if line1.len() < 69 || line2.len() < 69 {
            return None;
        }

        // note that in TLE format, positions are fixed
        // https://en.wikipedia.org/wiki/Two-line_element_set

        // extract data from line 1
        let norad_id: u32 = line1[2..7].trim().parse().ok()?;
        let intl_id = line1[9..17].trim().to_string();
        let launch_year: u16 = line1[9..11].parse().ok()?;
        let launch_number: u16 = line1[11..14].trim().parse().ok()?;
        let epoch_year: u16 = line1[18..20].parse().ok()?;
        let epoch_day: f64 = line1[20..32].trim().parse().ok()?;

        // extract data from line 2
        let inclination: f64 = line2[8..16].trim().parse().ok()?;
        let mean_motion: f64 = line2[52..63].trim().parse().ok()?;

        // parse with SGP4
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

    // calculate geodesic position using SGP4
    fn calculate_position(&self) -> Option<(f64, f64, f64)> {
        let minutes_since_epoch = self.calculate_minutes_since_epoch()?;
        let prediction = self.constants.propagate(sgp4::MinutesSinceEpoch(minutes_since_epoch)).ok()?;

        // convert cartesian position (km) to lat/lon/alt
        let (lat, lon, alt) = cartesian_to_geodetic(
            prediction.position[0],
            prediction.position[1],
            prediction.position[2]
        );

        Some((lat, lon, alt))
    }

    // calculate how many minutes has passed since this satellite's epoch
    fn calculate_minutes_since_epoch(&self) -> Option<f64> {
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

// convert cartesian coordinates (x, y, z) to geodetic coordinates (lat, lon, alt)
// https://en.wikipedia.org/wiki/Spherical_coordinate_system#Cartesian_coordinates
fn cartesian_to_geodetic(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let earth_radius = 6371.0;

    let distance = (x*x + y*y + z*z).sqrt();
    let altitude = distance - earth_radius;

    let latitude = (z / distance).asin().to_degrees();
    let longitude = y.atan2(x).to_degrees();

    (latitude, longitude, altitude)
}

// system to fetch TLE data from API
fn fetch_data() {
    let task = std::thread::spawn(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            fetch_satellites().await
        })
    });

    // block briefly to get data
    // need to implement proper async handling in the future
    match task.join() {
        Ok(Ok(satellites)) => {
            println!(
                "\n=== Fetched {} Navigation Satellites ===\n",
                satellites.len()
            );

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

                match satellite.calculate_position() {
                    Some((lat, lon, alt)) => {
                        println!(
                            "  Current Position: {:.4}°, {:.4}° at {:.1} km altitude",
                            lat, lon, alt
                        );
                    }
                    None => {
                        println!("  Current Position: Unable to calculate");
                    }
                }

                println!();
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

// async function to actually fetch and parse the satellite data
async fn fetch_satellites() -> Result<Vec<Satellite>, Error> {
    // call API here
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