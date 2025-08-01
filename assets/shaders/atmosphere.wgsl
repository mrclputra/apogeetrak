#import bevy_pbr::forward_io::VertexOutput

// atmosphere paramaters
struct AtmosphereUniform {
    sun_direction: vec3<f32>,
    camera_position: vec3<f32>,
    rayleigh_coeff: vec3<f32>,
    mie_coeff: f32,
    sun_intensity: f32,
    atmosphere_radius: f32,
    _padding: f32,
}

@group(2) @binding(0) var<uniform> atmosphere: AtmosphereUniform;

// ray-marching config
const SAMPLE_COUNT: i32 = 64;
const EARTH_RADIUS: f32 = 6378.0;

// phase functions for scattering
fn rayleigh_phase(cos_theta: f32) -> f32 {
    return 0.75 * (1.0 + cos_theta * cos_theta);
}

fn mie_phase(cos_theta: f32, g: f32) -> f32 {
    // Henvey-Greenstein phase function
    // https://www.oceanopticsbook.info/view/scattering/level-2/the-henyey-greenstein-phase-function
    let g2 = g * g;
    let denom = 1.0 + g2 - 2.0 * g2 * cos_theta;
    return (1.0 - g2) / (4.0 * 3.14159 * pow(denom, 1.5));
}

// camera ray sphere intesersection
// return distance or -1 if no hit
fn compute_ray_sphere_hit(ray_origin: vec3<f32>, ray_dir: vec3<f32>, center: vec3<f32>, radius: f32) -> f32 {
    let oc = ray_origin - center;
    let a = dot(ray_dir, ray_dir);
    let b = 2.0 * dot(oc, ray_dir);
    let c = dot(oc, oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return -1.0; // no intersect
    }

    let t1 = (-b - sqrt(discriminant)) / (2.0 * a);
    let t2 = (-b + sqrt(discriminant)) / (2.0 * a);

    // return nearest positive intersection
    if t1 > 0.0 {
        return t1;
    } else if t2 > 0.0 {
        return t2;
    }

    return -1.0;
}

// simple density function
fn atmosphere_density(position: vec3<f32>) -> f32 {
    // let altitude = length(position) - EARTH_RADIUS;
    let altitude = max(0.0, length(position) - EARTH_RADIUS);
    let max_altitude = atmosphere.atmosphere_radius - EARTH_RADIUS + 100.0;

    if altitude < -1.0 || altitude > max_altitude {
        return 0.0;
    }

    let scale_height = 8000.0; // km, controls exponential dropoff
    let exp_falloff = exp(-altitude / scale_height);
    let fade_to_zero = clamp(1.0 - (altitude / max_altitude), 0.0, 1.0);

    return exp_falloff * fade_to_zero;
}

// calculate how much light get;s abosrbed along a path
// optical depth
fn compute_optical_depth(start_pos: vec3<f32>, end_pos: vec3<f32>, steps: i32) -> vec2<f32> {
    let ray_dir = end_pos - start_pos;
    let step_size = length(ray_dir) / f32(steps);
    let step_dir = normalize(ray_dir);

    var rayleigh_depth = 0.0;
    var mie_depth = 0.0;

    // TODO: depth and sun should determine color
    for (var i = 0; i < steps; i++) {
        let t = (f32(i) + 0.5) * step_size;
        let sample_pos = start_pos * step_dir * t;
        let density = atmosphere_density(sample_pos);

        rayleigh_depth += density * step_size;
        mie_depth += density * step_size;
    }

    return vec2<f32>(rayleigh_depth, mie_depth);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let world_pos = in.world_position.xyz;
    let camera_pos = atmosphere.camera_position;

    // ray from camera through this pixel
    let ray_dir = normalize(world_pos - camera_pos);
    let ray_origin = camera_pos;

    // check if ray hits earth
    // for termination since I don't want to deal with layers
    let earth_distance = compute_ray_sphere_hit(ray_origin, ray_dir, vec3<f32>(0.0), EARTH_RADIUS);

    // determine how far to march
    // either hit the earth or until atmosphere becomes negligible
    let max_march_distance = select(atmosphere.atmosphere_radius * 10.0, earth_distance, earth_distance > 0.0);

    // ray marching
    let step_size = max_march_distance / f32(SAMPLE_COUNT);
    let sun_dir = normalize(atmosphere.sun_direction);

    var rayleigh_sum = vec3<f32>(0.0);
    var mie_sum = vec3<f32>(0.0);
    var total_density_encountered = 0.0;

    // march along ray
    for (var i = 0; i < SAMPLE_COUNT; i++) {
        let t = f32(i) * step_size + step_size * 0.5; // sample middle of each step
        let sample_pos = ray_origin + ray_dir * t;

        // get atmospheric density at this point
        let density = atmosphere_density(sample_pos);
        if density < 0.00001 { continue; } // skip thin atmosphere

        total_density_encountered += density;

        // calculate scattering contributions
        let cos_theta = dot(ray_dir, sun_dir); // angle between view and sun
        let rayleigh_scatter = atmosphere.rayleigh_coeff * density * rayleigh_phase(cos_theta);
        
        // mie
        let g = 0.76;
        let mie_scatter = vec3<f32>(atmosphere.mie_coeff) * density * mie_phase(cos_theta, g);

        // calculate how much sunlight reaches this point
        let sun_attenuation = clamp(dot(normalize(sample_pos), sun_dir), 0.005, 1.0);

        rayleigh_sum += rayleigh_scatter * sun_attenuation * step_size;
        mie_sum += mie_scatter * sun_attenuation * step_size;
    }

    // combine scattering effects
    let final_color = (rayleigh_sum + mie_sum) * atmosphere.sun_intensity;
    
    // calculate alpha baed on total scattering
    // let alpha = min(1.0, (rayleigh_sum.r + rayleigh_sum.g + rayleigh_sum.b + mie_sum.r) * 0.5);
    let alpha = min(1.0, total_density_encountered * 0.01);

    return vec4<f32>(final_color, alpha);
}