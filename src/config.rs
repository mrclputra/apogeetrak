// Earth measurements (in kilometers)
pub const EARTH_RADIUS: f32 = 6378.0;
pub const CLOUD_RADIUS: f32 = 6428.0;

// Rotation speeds
pub const EARTH_ROTATION_SPEED: f32 = 0.01;

// Asset paths
pub const EARTH_DIFFUSE_TEXTURE: &str = "textures/diffuse.tif";
pub const EARTH_NIGHT_TEXTURE: &str = "textures/night.tif";
pub const EARTH_CLOUDS_TEXTURE: &str = "textures/clouds.tif";
pub const EARTH_OCEAN_MASK_TEXTURE: &str = "textures/ocean_mask.png";
pub const EARTH_SPECULAR_TEXTURE: &str = "textures/specular.tif";

pub const EARTH_DISPLACEMENT_TEXTURE: &str = "textures/topography.png";
pub const DISPLACEMENT_SCALE: f32 = 10.0; // maximum displacement, in km