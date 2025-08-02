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
@group(2) @binding(8) var normal_map: texture_2d<f32>;
@group(2) @binding(9) var normal_map_sampler: sampler;

// sun direction uniform
struct SunUniform {
    sun_direction: vec3<f32>,
    _padding: f32, // 16-byte alignment
};

@group(2) @binding(10) var<uniform> sun_uniform: SunUniform;

const PI: f32 = 3.14159265;

// desaturate a color
fn desaturate(color: vec3<f32>, factor: f32) -> vec3<f32> {
    let gray = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    return mix(color, vec3<f32>(gray), factor);
}

// calculate tangent space vectors for spherical surfaces
fn calculate_sphere_tangent_space(world_pos: vec3<f32>, uv: vec2<f32>) -> mat3x3<f32> {
    // normalize position to get point on unit sphere
    let point_on_sphere = normalize(world_pos);

    let longitude = uv.x * 2.0 * PI;
    let latitude = (0.5 - uv.y) * PI;

    // calculate tangent
    // direction of increasing longitude
    let tangent = vec3<f32>(
        -sin(longitude),
        0.0,
        cos(longitude)
    );

    // calculate bitangent
    // direction of increasing latitude
    let bitangent = vec3<f32>(
        -cos(longitude) * sin(latitude),
        cos(latitude),
        -sin(longitude) * sin(latitude)
    );

    let normal = point_on_sphere;

    return mat3x3<f32>(
        normalize(tangent),
        normalize(bitangent),
        normal
    );
}

// sample and decode normal
fn sample_normal_map_sphere(uv: vec2<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    // get the mesh normal
    let mesh_normal = normalize(world_pos);

    // sample normal map
    let normal_sample = textureSample(normal_map, normal_map_sampler, uv).rgb;
    
    let detail_normal = normal_sample * 2.0 - 1.0;
    let tbn_matrix = calculate_sphere_tangent_space(world_pos, uv);
    
    // transform detail normal to world space
    let world_detail_normal = tbn_matrix * detail_normal;

    // u can control normal strength here
    let blended_normal = normalize(mesh_normal + world_detail_normal * 0.3);
    
    return blended_normal;
}

// calculate specular value at a given position
fn calculate_specular(
    world_normal: vec3<f32>,
    light_dir: vec3<f32>,
    view_dir: vec3<f32>,
    roughness: f32,
    specular_strength: f32
) -> f32 {
    let halfway = normalize(light_dir + view_dir);
    let shininess = mix(512.0, 1.0, roughness);

    // calculate specular component
    let spec = pow(max(dot(world_normal, halfway), 0.0), shininess);
    return spec * specular_strength;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // sample textures
    let day_color = textureSample(day_texture, day_sampler, in.uv);
    let night_color = textureSample(night_texture, night_sampler, in.uv).rgb * vec3<f32>(0.94, 0.78, 0.67);
    let mask_value = textureSample(ocean_mask, ocean_mask_sampler, in.uv).r;
    let specular_value = textureSample(specular_map, specular_map_sampler, in.uv).r;

    let world_normal = sample_normal_map_sphere(in.uv, in.world_position.xyz);

    // calculate lighing vectors
    let light_dir = normalize(sun_uniform.sun_direction);
    let view_dir = normalize(-in.world_position.xyz);

    // point on unit sphere for consistent lighting reference
    let point_on_sphere = normalize(in.world_position.xyz);
    
    // diffuse lighting calculation using normal-mapped surface
    let sun_factor = max(0.0, dot(world_normal, light_dir) + 0.0);
    
    // add some fake lighting
    // prevent flat when sun is overhead
    let fake_lighting = pow(dot(world_normal, point_on_sphere), 4.0);
    
    // transition between day and night
    let base_blend = 0.003;
    let day_night_blend = base_blend + (1.0 - base_blend) * smoothstep(0.0, 0.42, sun_factor);
    
    // mix textures based on sun exposure
    var final_color = mix(night_color.rgb, day_color.rgb, day_night_blend);

    // apply ocean mask
    // oceans are colourless, the sky makes them blue
    if (mask_value > 0.5) {
        final_color = desaturate(final_color, 0.85);
        
        // calculate ocean specular
        let ocean_roughness = 0.1; // oceans are smoother
        let ocean_specular_strength = specular_value * 200.0; // should give stronger specular
        
        let ocean_specular = calculate_specular(
            world_normal,
            light_dir,
            view_dir,
            ocean_roughness,
            ocean_specular_strength
        ) * clamp(sun_factor, 0.0, 1.0);
        
        // add ocean specular highlights
        let ocean_specular_color = vec3<f32>(1.0, 1.0, 0.95);
        final_color += ocean_specular * day_night_blend * ocean_specular_color;
    } else {
        // land specular 
        let land_roughness = 1.0 - specular_value * 0.5; // land is rougher
        let land_specular_strength = specular_value * 20.0; // should be weaker than ocean
        
        let land_specular = calculate_specular(
            world_normal,
            light_dir,
            view_dir,
            land_roughness,
            land_specular_strength
        ) * clamp(sun_factor, 0.0, 1.0);
        
        // add land specular
        let land_specular_color = vec3<f32>(1.0, 1.0, 0.95);
        final_color += land_specular * day_night_blend * land_specular_color;
    }
    
    // fake lighting to add dimension
    final_color *= mix(fake_lighting, 1.0, 0.5);

    return vec4<f32>(final_color, 1.0);
}