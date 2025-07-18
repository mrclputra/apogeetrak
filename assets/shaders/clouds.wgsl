#import bevy_pbr::forward_io::VertexOutput

struct SunUniform {
   sun_direction: vec3<f32>,
   _padding: f32,
}

@group(2) @binding(0) var cloud_texture: texture_2d<f32>;
@group(2) @binding(1) var cloud_sampler: sampler;
@group(2) @binding(2) var<uniform> sun_uniform: SunUniform;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
   let color = textureSample(cloud_texture, cloud_sampler, in.uv);

   // use brightness as alpha
   let alpha = (color.r + color.g + color.b) / 3.0;
   // let alpha = color.r

   // simple lighting with sun direction
   let normal = normalize(in.world_normal);
   let light_dir = normalize(sun_uniform.sun_direction);
   let lighting = max(0.0, dot(normal, light_dir));

   // mixed cloud color
   let shadow_color = vec3<f32>(0.02, 0.02, 0.04);  // night clouds
   let lit_color = vec3<f32>(1.0, 1.0, 1.0);     // day clouds
   let final_color = mix(shadow_color, lit_color, lighting);

   // darken clouds at night
   let surface_light = max(0.0, sun_uniform.sun_direction.y * 0.5 + 0.5);
   
   return vec4<f32>(final_color * surface_light, alpha);
}