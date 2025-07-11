use bevy::prelude::*;

use chrono::{NaiveDate, Utc};
use chrono::DateTime;
use reqwest::Error;
use reqwest::header::USER_AGENT;
use sgp4::Prediction;

use crate::EARTH_RADIUS;

// unified satellite component type
#[derive(Component, Clone)]
pub struct Satellite {
    pub name: String,
    pub norad_id: u32,      // SATCAT, 5-digit number
    pub intl_id: String,    // Intl ID
    pub launch_year: u16,   // last 2 digits of year
    pub launch_number: u16, // launch number of year
    pub epoch_year: u16,    // last 2 digits of year
    pub epoch_day: f64,     // includes fractional portion of day
    pub mean_motion: f64,   // ballistic coefficient
    pub inclination: f64,   // degrees

    // SGP4 pre-parse data
    // elements: sgp4::Elements,
    constants: sgp4::Constants,
}

impl Satellite {
    // note: TLE is always 3 lines
    pub fn parse(name: &str, line1: &str, line2: &str) -> Option<Self> {
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
            // elements,
            constants,
        })
    }

    // TODO: combine propagation calculations into one function
    // 'Prediction' type returns (x, y, z) position and (vX, vY, vZ) velocities
    pub fn calculate(&self) -> Option<Prediction> {
        let minutes_since_epoch = self.calculate_minutes_since_epoch()?;
        self.constants.propagate(sgp4::MinutesSinceEpoch(minutes_since_epoch)).ok()
    }

    // calculate position at a specific time offset (in minutes from now, can be negative)
    // used to get positions of satellite at different periods of the orbit (for drawing)
    // TODO: maybe merge this function with 'calculate'
    pub fn calculate_at_offset(&self, offset_minutes: f64) -> Option<Prediction> {
        let minutes_since_epoch = self.calculate_minutes_since_epoch()? + offset_minutes;
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

    // calculate orbital trajectory
    // returns a traceable array of points on orbital path
    pub fn generate_orbit_path(&self, num_points: usize) -> Vec<Vec3> {
        let mut path_points = Vec::new();
        
        // calculate one full orbital period
        let orbital_period = if self.mean_motion > 0.0 {
            1440.0 / self.mean_motion // minutes in one orbit
        } else {
            90.0 // fallback to 90 minutes
        };

        // generate points along the orbit
        for i in 0..num_points {
            let time_offset = (i as f64 / num_points as f64) * orbital_period;
            
            if let Some(prediction) = self.calculate_at_offset(time_offset) {
                let pos = sgp4_to_cartesian(&prediction);
                path_points.push(pos);
            }
        }
        
        path_points
    }
}

// UTILS

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
pub fn sgp4_to_cartesian(prediction: &Prediction) -> Vec3 {
    // SGP4 returns coordinates in kilometers
    Vec3::new(
        prediction.position[0] as f32,
        prediction.position[2] as f32,  // swapped Y and Z
        prediction.position[1] as f32,
    )
}

// // update all satellite positions
// pub fn update_satellite_positions(
//     mut satellite_query: Query<(&mut Transform, &Satellite)>,
// ) {
//     for (mut transform, satellite) in satellite_query.iter_mut() {
//         // calculate current position
//         if let Some(prediction) = satellite.calculate() {
//             transform.translation = sgp4_to_cartesian(&prediction);
//         }
//     }
// }

// async function to actually fetch and parse the satellite data
// QA: should i combine this with "load_satellites()"?
pub async fn fetch_satellites() -> Result<Vec<Satellite>, Error> {
    // call fileserver here
    let url = "https://celestrak.org/NORAD/elements/gp.php?GROUP=gnss&FORMAT=tle";

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