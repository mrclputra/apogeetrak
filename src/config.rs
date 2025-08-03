//! config.rs
//!
//! Configure some simulation constants here
//! Not all are included, some are still magic numbers in code somewhere :P

// Earth measurements (in km)
pub const EARTH_RADIUS: f32 = 6378.0;
pub const ATMOSPHERE_RADIUS: f32 = 7000.0;
pub const CLOUD_RADIUS: f32 = 6478.0;
pub const DISPLACEMENT_SCALE: f32 = 80.0; // maximum terrain height

// Atmospheric scattering parameters
// based on values from https://www.scratchapixel.com/lessons/procedural-generation-virtual-worlds/simulating-sky/simulating-colors-of-the-sky.html
pub const RAYLEIGH_COEFF: [f32; 3] = [5.8e-6, 13.5e-6, 33.1e-6]; // RGB wavelengths
pub const MIE_COEFF: f32 = 210.0e-5;
pub const SUN_INTENSITY: f32 = 10.0;

// Rotation speeds
pub const EARTH_ROTATION_SPEED: f32 = 0.00005; // radians, dont touch

// Normal map generation config
pub const USE_SAVED_NORMAL_MAP: bool = true; // change this if you want the program to generate a new normal map every time it compiles
pub const SAVED_NORMAL_MAP_PATH: &str = "textures/normal.png";

// Asset paths
pub const EARTH_DIFFUSE_TEXTURE: &str = "textures/diffuse.tif";
pub const EARTH_NIGHT_TEXTURE: &str = "textures/night.tif";
pub const EARTH_CLOUDS_TEXTURE: &str = "textures/clouds.tif";
pub const EARTH_OCEAN_MASK_TEXTURE: &str = "textures/ocean_mask.png";
pub const EARTH_SPECULAR_TEXTURE: &str = "textures/specular.tif";

pub const EARTH_DISPLACEMENT_TEXTURE: &str = "textures/topography.png";