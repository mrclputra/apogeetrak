// Earth measurements (in kilometers)
pub const EARTH_RADIUS: f32 = 6378.0;
pub const ATMOSPHERE_RADIUS: f32 = 7000.0;
pub const DISPLACEMENT_SCALE: f32 = 100.0; // maximum displacement, in km

// Atmospheric scattering parameters
// https://physics.stackexchange.com/questions/241190
pub const RAYLEIGH_COEFF: [f32; 3] = [0.0000055, 0.000013, 0.0000224]; // RGB wavelengths
pub const MIE_COEFF: f32 = 0.00012;
pub const SUN_INTENSITY: f32 = 22.0;

// Atmosphere quality settings
pub const ATMOSPHERE_SAMPLE_COUNT: i32 = 24; // ray marching steps for atmosphere
pub const ATMOSPHERE_SOFT_EDGE_START: f32 = 0.8; // where soft edge transition begins (as fraction of atmosphere radius)

// Rotation speeds
pub const EARTH_ROTATION_SPEED: f32 = 0.07;

// Asset paths
pub const EARTH_DIFFUSE_TEXTURE: &str = "textures/diffuse.tif";
pub const EARTH_NIGHT_TEXTURE: &str = "textures/night.tif";
pub const EARTH_CLOUDS_TEXTURE: &str = "textures/clouds.tif";
pub const EARTH_OCEAN_MASK_TEXTURE: &str = "textures/ocean_mask.png";
pub const EARTH_SPECULAR_TEXTURE: &str = "textures/specular.tif";

pub const EARTH_DISPLACEMENT_TEXTURE: &str = "textures/topography.png";