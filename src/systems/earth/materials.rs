use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::reflect::TypePath;
use bevy::asset::Asset;

// sun direction data
#[derive(ShaderType, Clone, Copy, Debug)]
#[repr(C)]
pub struct SunUniform {
    pub sun_direction: Vec3,
    pub _padding: f32, // ensures proper 16-byte GPU alignment
}

// earth material
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct EarthMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub day_texture: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    pub night_texture: Handle<Image>,
    #[uniform(4)]
    pub sun_uniform: SunUniform,
}

impl Material for EarthMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/earth.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

// cloud material
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CloudMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub cloud_texture: Handle<Image>,
    #[uniform(2)]
    pub sun_uniform: SunUniform,
    #[uniform(3)]
    pub cloud_opacity: f32, // runtime adjustments
}

impl Material for CloudMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/clouds.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend // transparency support
    }
}