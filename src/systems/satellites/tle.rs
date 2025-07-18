use bevy::prelude::*;

use chrono::{Utc, DateTime, Datelike};
use reqwest::Error;
use reqwest::header::USER_AGENT;
use sgp4::Prediction;

// TEMP
use std::fs;
use std::path::Path;

use crate::config::EARTH_RADIUS;

// unified satellite component type
#[derive(Component, Clone)]
pub struct Satellite {
    // SGP4 datatypes, extracted from TLE lines
    pub elements: sgp4::Elements,
    pub constants: sgp4::Constants,
}

impl Satellite {
    // note: TLS is always 3 lines
    pub fn parse(name: &str, line1: &str, line2: &str) -> Option<Self> {
        if line1.len() < 69 || line2.len() < 69 {
            return None;
        }

        // I just let the SGP4 library do the heavy lifting
        let elements = sgp4::Elements::from_tle(
            Some(name.trim().to_string()),
            line1.as_bytes(),
            line2.as_bytes()
        ).ok()?;
        
        let constants = sgp4::Constants::from_elements(&elements).ok()?;

        Some(Satellite {
            elements,
            constants
        })
    }

    // getters
    pub fn name(&self) -> &str {
        self.elements.object_name.as_deref().unwrap_or("Unknown")
    }
    pub fn norad_id(&self) -> u64 {
        self.elements.norad_id
    }
    pub fn intl_id(&self) -> &str {
        self.elements.international_designator.as_deref().unwrap_or("Unknown")
    }
    pub fn inclination(&self) -> f64 {
        self.elements.inclination
    }
    pub fn mean_motion(&self) -> f64 {
        self.elements.mean_motion
    }
    pub fn epoch_datetime(&self) -> &chrono::NaiveDateTime {
        &self.elements.datetime
    }

    // calculates position of satellite given a time
    // 'Prediction' type returns (x, y, z) position and (vX, vY, vZ) velocities
    pub fn calculate(&self, propagation_time: DateTime<Utc>) -> Prediction {
        let tsince = match self.minutes_since_epoch(propagation_time) {
            Some(t) => t,
            None => {
                return Prediction {
                    position: [0.0, 0.0, 0.0],
                    velocity: [0.0, 0.0, 0.0],
                };
            }
        };

        self.constants.propagate(sgp4::MinutesSinceEpoch(tsince))
            .unwrap_or_else(|_| Prediction {
                position: [0.0, 0.0, 0.0],
                velocity: [0.0, 0.0, 0.0],
            })
        // self.constants.propagate(sgp4::MinutesSinceEpoch(tsince)).ok()
    }

    fn minutes_since_epoch(&self, target_time: DateTime<Utc>) -> Option<f64> {
        let target_naive = target_time.naive_utc();

        match self.elements.datetime_to_minutes_since_epoch(&target_naive) {
            Ok(minutes_since_epoch) => Some(minutes_since_epoch.0),
            Err(_) => None,
        }
    }

    // calculate orbital trajectory
    // returns a traversable array of points on oorbital path
    pub fn generate_orbit_path(&self, resolution: usize) -> Vec<Vec3> {
        let mut path_points = Vec::with_capacity(resolution); // optimization

        let orbital_period = if self.elements.mean_motion > 0.0 {
            1440.0 / self.elements.mean_motion // minutes in one orbit
        } else {
            90.0 // fallback
        };

        let start_time = self.elements.datetime.and_utc();

        // generate points along the orbit
        for i in 0..resolution {
            let time_offset_minutes = (i as f64 / resolution as f64) * orbital_period;
            let offset_duration = chrono::Duration::seconds((time_offset_minutes * 60.0) as i64);
            let target_time = start_time + offset_duration;

            let prediction = self.calculate(target_time);
            let pos = sgp4_to_cartesian(&prediction);
            path_points.push(pos);
        }

        path_points
    }

    // HELPERS

    // get current position (x, y ,z)
    pub fn current_position(&self) -> Vec3 {
        let prediction = self.calculate(Utc::now());
        sgp4_to_cartesian(&prediction)
    }

    // get current geodetic position (lat, lon, alt)
    pub fn current_geodetic_position(&self) -> (f64, f64, f64) {
        let prediction = self.calculate(Utc::now());
        cartesian_to_geodetic(
            prediction.position[0],
            prediction.position[1], 
            prediction.position[2]
        )
    }

    // get current velocity (vX, vY, vZ)
    #[allow(dead_code)]
    pub fn current_velocity(&self) -> (f64, f64, f64) {
        let prediction = self.calculate(Utc::now());
        (
            prediction.velocity[0],
            prediction.velocity[1],
            prediction.velocity[2]
        )
    }

    // just prints contents
    #[allow(dead_code)]
    pub fn print(&self) {
        println!("  Name        : {}", self.name());
        println!("  NORAD ID    : {}", self.norad_id());
        println!("  Intl ID     : {}", self.intl_id());
        println!("  Inclination : {:.2}", self.inclination());
        println!("  Mean Motion : {:.2}", self.mean_motion());
        println!("  Epoch       : Year {} Day {:.2}", self.epoch_datetime().year(), self.epoch_datetime().day());
        println!();

        // print current ECI position
        let pos = self.current_position();
        println!(
            "  ECI         : {:.2} km, {:.2} km, {:.2} km", pos.x, pos.y, pos.z
        );

        // print current geodetic position
        let (lat, lon, alt) = self.current_geodetic_position();
        println!(
            "  Geo         : {:.4}°, {:.4}° at {:.1} km",
            lat, lon, alt
        );

        // print current velocity
        let (vx, vy, vz) = self.current_velocity();
        let speed = (vx.powi(2) + vy.powi(2) + vz.powi(2)).sqrt();
        println!(
            "  Velocity    : {:.2} km/s", speed
        );

        println!();
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

// async function to actually fetch and parse the satellite data
pub async fn fetch_satellites() -> Result<Vec<Satellite>, Error> {
    // DEV: toggle to select source
    let load_from_file = true;

    let tle_data = if load_from_file {
        // load from local file
        let path = Path::new("assets/data/gnss.txt");
        match fs::read_to_string(path) {
            Ok(contents) => {
                println!("Loaded TLE data from local file: {:?}", path);
                contents
            }
            Err(err) => {
                eprintln!("Failed to read TLE file: {err}");
                return Ok(vec![]);
            }
        }
    } else {
        // load from remote URL (keep this intact)
        let url = "https://celestrak.org/NORAD/elements/gp.php?GROUP=gnss&FORMAT=tle";
        let response = reqwest::Client::new()
            .get(url)
            .header(USER_AGENT, "apogeetrak-satellite-tracker")
            .send()
            .await?;

        println!("Fetched TLE data from remote URL: {url}");
        response.text().await?
    };

    // parse the TLE data
    let lines: Vec<&str> = tle_data.lines().collect();
    let mut satellites: Vec<Satellite> = Vec::new();

    for chunk in lines.chunks(3) {
        if chunk.len() == 3 {
            if let Some(satellite) = Satellite::parse(chunk[0], chunk[1], chunk[2]) {
                satellites.push(satellite);
            }
        }
    }

    println!("Successfully parsed {} satellites.", satellites.len());
    Ok(satellites)
}