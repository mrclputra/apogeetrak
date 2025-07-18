#import bevy_pbr::forward_io::VertexOutput

// textures
@group(2) @binding(0) var day_texture: texture_2d<f32>;
@group(2) @binding(1) var day_sampler: sampler;
@group(2) @binding(2) var night_texture: texture_2d<f32>;
@group(2) @binding(3) var night_sampler: sampler;
@group(2) @binding(4) var ocean_mask: texture_2d<f32>;
@group(2) @binding(5) var ocean_mask_sampler: sampler;
@group(2) @binding(6) var specular_map: texture_2d<f32>;
@group(2) @binding(7) var specular_map_sampler: sampler;

// sun direction uniform
struct SunUniform {
    sun_direction: vec3<f32>,
    _padding: f32, // 16-byte alignment
};

@group(2) @binding(8) var<uniform> sun_uniform: SunUniform;

// desaturate a color - keeps your original function
fn desaturate(color: vec3<f32>, factor: f32) -> vec3<f32> {
    let gray = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    return mix(color, vec3<f32>(gray), factor);
}

// calculate specular reflection using Blinn-Phong model
fn calculate_specular(
    world_normal: vec3<f32>,
    light_dir: vec3<f32>,
    view_dir: vec3<f32>,
    roughness: f32,
    specular_strength: f32
) -> f32 {
    // use halfway vector for more accurate specular highlights
    let halfway = normalize(light_dir + view_dir);
    
    // calculate shininess from roughness (rough = low shininess, smooth = high shininess)
    let shininess = mix(512.0, 8.0, roughness);
    
    // calculate specular component
    let spec = pow(max(dot(world_normal, halfway), 0.0), shininess);
    
    return spec * specular_strength;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // sample all our textures
    let day_color = textureSample(day_texture, day_sampler, in.uv);
    let night_color = textureSample(night_texture, night_sampler, in.uv);
    let mask_value = textureSample(ocean_mask, ocean_mask_sampler, in.uv).r;
    let specular_value = textureSample(specular_map, specular_map_sampler, in.uv).r;

    // calculate lighting vectors
    let world_normal = normalize(in.world_normal);
    let light_dir = normalize(sun_uniform.sun_direction);
    
    // view direction approximation
    // fragment towards camera
    let view_dir = normalize(-in.world_position.xyz);
    
    // diffuse lighting calculation
    let sun_factor = max(0.0, dot(world_normal, light_dir) + 0.0);
    
    // smooth transition between day and night
    let day_night_blend = smoothstep(0.0, 0.32, sun_factor);
    
    // mix textures based on sun exposure
    var final_color = mix(night_color.rgb, day_color.rgb, day_night_blend);

    // apply ocean mask
    if (mask_value > 0.5) {
        final_color = desaturate(final_color, 0.8);
    }

    // calculate specular contribution
    // black = rough, white = smooth
    let roughness = 1.0 - specular_value; // invert 
    let specular_strength = specular_value * 100.0; // adjust overall specular intensity
    
    let specular_contribution = calculate_specular(
        world_normal,
        light_dir,
        view_dir,
        roughness,
        specular_strength
    );
    
    // add specular highlights (only on day side, with slight warm tint)
    let specular_color = vec3<f32>(1.0, 1.0, 0.95); // slightly warm white
    final_color += specular_contribution * day_night_blend * specular_color;

    return vec4<f32>(final_color, 1.0);
}