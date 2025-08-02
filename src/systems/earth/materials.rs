use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::reflect::TypePath;
use bevy::asset::Asset;

// sun direction data (needs to be in a struct)
// this is how to pass sun vector3 data to the earth shader (local only)
// https://www.w3.org/TR/WGSL/#address-space-layout-constraints
#[derive(ShaderType, Clone, Copy, Debug)]
#[repr(C)]
pub struct SunUniform {
    pub direction: Vec3,
    pub _padding: f32, // ensures proper 16-byte GPU alignment
}

// atmosphere uniform data
#[derive(ShaderType, Clone, Copy, Debug)]
#[repr(C)]
pub struct AtmosphereUniform {
    pub sun_direction: Vec3,
    pub camera_position: Vec3,

    // https://en.wikipedia.org/wiki/Rayleigh_scattering
    pub rayleigh_coeff: Vec3,
    pub mie_coeff: f32,
    pub sun_intensity: f32,
    pub atmosphere_radius: f32,
    pub _padding: f32,
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
    #[texture(4)]
    #[sampler(5)]
    pub ocean_mask: Handle<Image>,
    #[texture(6)]
    #[sampler(7)]
    pub specular_map: Handle<Image>,
    #[texture(8)]
    #[sampler(9)]
    pub normal_map: Handle<Image>,
    #[uniform(10)]
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

// atmosphere material
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct AtmosphereMaterial {
    #[uniform(0)]
    pub atmosphere_uniform: AtmosphereUniform,
}

impl Material for AtmosphereMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/atmosphere.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
    
    fn specialize(
            _pipeline: &bevy::pbr::MaterialPipeline<Self>,
            descriptor: &mut RenderPipelineDescriptor,
            _layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
            _key: bevy::pbr::MaterialPipelineKey<Self>,
        ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
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