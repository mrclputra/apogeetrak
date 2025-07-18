#import bevy_pbr::forward_io::VertexOutput

// textures
@group(2) @binding(0) var day_texture: texture_2d<f32>;
@group(2) @binding(1) var day_sampler: sampler;
@group(2) @binding(2) var night_texture: texture_2d<f32>;
@group(2) @binding(3) var night_sampler: sampler;
@group(2) @binding(4) var ocean_mask: texture_2d<f32>;
@group(2) @binding(5) var ocean_mask_sampler: sampler;

// sun direction uniform
struct SunUniform {
    sun_direction: vec3<f32>,
    _padding: f32, // 16-byte alignment
};

@group(2) @binding(6) var<uniform> sun_uniform: SunUniform;

// desaturate a color
fn desaturate(color: vec3<f32>, factor: f32) -> vec3<f32> {
    let gray = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    return mix(color, vec3<f32>(gray), factor);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // sample day and night textures
    let day_color = textureSample(day_texture, day_sampler, in.uv);
    let night_color = textureSample(night_texture, night_sampler, in.uv);

    // calculate how much this surface faces the sun
    let world_normal = normalize(in.world_normal);
    let light_dir = normalize(sun_uniform.sun_direction);

    let sun_factor = max(0.0, dot(world_normal, light_dir) + 0.0);
    
    // smooth transition between day and night
    let day_night_blend = smoothstep(0.0, 0.32, sun_factor);
    
    // mix textures based on sun exposure
    var final_color = mix(night_color.rgb, day_color.rgb, day_night_blend);

    // sample ocean mask
    let mask_value = textureSample(ocean_mask, ocean_mask_sampler, in.uv).r;
    if (mask_value > 0.5) {
        final_color = desaturate(final_color, 0.8);
    }

    return vec4<f32>(final_color, 1.0);
}