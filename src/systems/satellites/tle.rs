//! tle.rs
//! 
//! Orbital mechanics utilities here
//! uses SGP4 model to propagate satellite orbits from TLE datasets, and convert
//! orbital predictions into Bevy world coordinates

use bevy::prelude::*;

use bevy::asset::uuid::Error;
use chrono::{DateTime, Duration, Utc};
use sgp4::Prediction;

use std::fs;
use std::path::Path;

use crate::config::EARTH_RADIUS;

// point in orbital path
#[derive(Clone, Debug)]
pub struct OrbitPoint {
    pub time: DateTime<Utc>,
    pub position: Vec3,
}

// satellite component
#[derive(Component, Clone)]
pub struct Satellite {
    // sgp4 datatypes
    pub elements: sgp4::Elements,
    pub constants: sgp4::Constants,

    pub orbit_path: Vec<OrbitPoint>,
    pub orbit_duration_m: f64, // how long the orbit path covers, minutes
}

impl Satellite {
    // note: TLE is always 3 lines
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
            constants,
            orbit_path: Vec::new(), // will be populated later
            orbit_duration_m: 0.0,
        })
    }

    // getters
    pub fn name(&self) -> &str {
        self.elements.object_name.as_deref().unwrap_or("Unknown")
    }
    // pub fn norad_id(&self) -> u64 {
    //     self.elements.norad_id
    // }
    // pub fn intl_id(&self) -> &str {
    //     self.elements.international_designator.as_deref().unwrap_or("Unknown")
    // }
    // pub fn inclination(&self) -> f64 {
    //     self.elements.inclination
    // }
    // pub fn mean_motion(&self) -> f64 {
    //     self.elements.mean_motion
    // }
    // pub fn epoch_datetime(&self) -> &chrono::NaiveDateTime {
    //     &self.elements.datetime
    // }

    // generate orbital path and store it in self.orbit_path
    pub fn generate_orbit_path(&mut self, resolution: usize, base_time: DateTime<Utc>) {
        self.orbit_duration_m = if self.elements.mean_motion > 0.0 {
            1440.0 / self.elements.mean_motion // revolutions per day
        } else {
            90.0 // fallback for weird cases
        };

        self.orbit_path.clear();
        self.orbit_path.reserve(resolution);

        // generate points along orbital path
        for i in 0..resolution {
            // figure out what time this point represents
            let time_fraction = i as f64 / resolution as f64;
            let time_offset_seconds = time_fraction * self.orbit_duration_m * 60.0;

            let point_time = base_time + Duration::milliseconds((time_offset_seconds * 1000.0) as i64);

            if let Some(minutes_since_epoch) = self.minutes_since_epoch(point_time) {
                let prediction = self
                    .constants
                    .propagate(sgp4::MinutesSinceEpoch(minutes_since_epoch))
                    .unwrap_or(Prediction {
                        position: [0.0, 0.0, 0.0],
                        velocity: [0.0, 0.0, 0.0],
                    });
                
                let position = sgp4_to_cartesian(&prediction);
                
                self.orbit_path.push(OrbitPoint {
                    time: point_time,
                    position,
                });
            }
        }
    }

    /// Get position of satellite given a time value
    /// interpolates across the generated orbit path
    pub fn get_position(&self, target_time: DateTime<Utc>) -> Vec3 {
        if self.orbit_path.is_empty() {
            return Vec3::ZERO;
        }

        if self.orbit_path.len() == 1 {
            return self.orbit_path[0].position;
        }

        let elapsed_minutes = (target_time - self.orbit_path[0].time).num_seconds() as f64 / 60.0;
        let cycle_time = elapsed_minutes.rem_euclid(self.orbit_duration_m); // correct modulo for negative times
        let time_per_segment = self.orbit_duration_m / (self.orbit_path.len() - 1) as f64;
        let segment_index = ((cycle_time / time_per_segment).floor() as usize)
            .min(self.orbit_path.len() - 2);

        let t = ((cycle_time % time_per_segment) / time_per_segment).clamp(0.0, 1.0);

        self.orbit_path[segment_index]
            .position
            .lerp(self.orbit_path[segment_index + 1].position, t as f32)
    }

    /// get geodetic position at specific time (lat, lon, alt)
    pub fn geodetic_position(&self, time: DateTime<Utc>) -> (f64, f64, f64) {
        let position = self.get_position(time);
        cartesian_to_geodetic(
            position.x as f64,
            position.y as f64, 
            position.z as f64
        )
    }

    // HELPERS

    /// time difference since TLE epoch
    fn minutes_since_epoch(&self, target_time: DateTime<Utc>) -> Option<f64> {
        let target_naive = target_time.naive_utc();

        match self.elements.datetime_to_minutes_since_epoch(&target_naive) {
            Ok(minutes_since_epoch) => Some(minutes_since_epoch.0),
            Err(_) => None,
        }
    }
}

// UTILS

/// convert Cartesian coordinates (x, y, z) to Geodetic coordinates (lat, lon, alt)
/// https://en.wikipedia.org/wiki/Geodetic_coordinates
pub fn cartesian_to_geodetic(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let distance = (x*x + y*y + z*z).sqrt();
    let altitude = distance - EARTH_RADIUS as f64;

    let latitude = (z / distance).asin().to_degrees();
    let longitude = y.atan2(x).to_degrees();

    (latitude, longitude, altitude)
}

/// convert SGP4 coordinates to Bevy world coordinates
pub fn sgp4_to_cartesian(prediction: &Prediction) -> Vec3 {
    // SGP4 returns coordinates in kilometers
    Vec3::new(
        prediction.position[0] as f32,
        prediction.position[2] as f32,  // swapped Y and Z
        prediction.position[1] as f32,
    )
}

/// fetch satellite data, asynchronous
pub async fn fetch_satellites() -> Result<Vec<Satellite>, Error> {
    let path = Path::new("assets/data/weather.txt");
    let tle_data = match fs::read_to_string(path) {
        Ok(contents) => {
            info!("Loaded TLE data from local file: {:?}", path);
            contents
        }
        Err(err) => {
            eprintln!("Failed to read TLE file: {err}");
            return Ok(vec![]);
        }
    };
    
    // parse TLE data into satellites
    let lines: Vec<&str> = tle_data.lines().collect();
    let mut satellites: Vec<Satellite> = Vec::new();

    for chunk in lines.chunks(3) {
        if chunk.len() == 3
            && let Some(satellite) = Satellite::parse(chunk[0], chunk[1], chunk[2])
        {
            satellites.push(satellite);
        }
    }

    info!("Parsed {} satellites", satellites.len());
    Ok(satellites)
}