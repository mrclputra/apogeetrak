use bevy::prelude::*;
use std::f32::consts::PI;

pub struct LatLon {
    // radians
    pub latitude: f32,
    pub longitude: f32,
}

// convert from cartesian
// https://en.wikipedia.org/wiki/Geodetic_coordinates
impl From<Vec3> for LatLon {
    fn from(value: Vec3) -> Self {
        let normalized = value.normalize();
        let latitude = normalized.y.asin();
        let longitude = normalized.x.atan2(normalized.z);
        LatLon {
            latitude,
            longitude
        }
    }
}

impl LatLon {
    pub fn as_degrees(&self) -> (f32, f32) {
        let latitude = self.latitude * (180.0 / PI);
        let longitude = self.longitude * (180.0 / PI);
        (latitude, longitude)
    }

    pub fn to_uv(&self) -> (f32, f32) {
        let (lat, lon) = self.as_degrees();
        let v = (90.0 - lat) / 180.0;
        let u = (lon + 180.0) / 360.0;
        (u, v)
    }
}