//! normal.rs
//! 
//! Normal map generator for height textures on a sphere

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::{ImageBuffer, Rgba};

use crate::config::EARTH_RADIUS;

/// Generates a normal map from a height map
/// each pixel's normal is calculated by sampling neighboring heights
/// and computing tanget vectors in world space
pub fn generate_normal_map(height_map: &Image) -> Image {
    let width = height_map.texture_descriptor.size.width as usize;
    let height = height_map.texture_descriptor.size.height as usize;

    // create output buffer
    let mut normal_data = vec![0u8; width * height * 4]; // RGBA

    // parse height data
    let height_data = match height_map.data.as_ref() {
        Some(data) => data.as_slice(),
        None => {
            // if no data, return blank
            return Image::new(
                Extent3d {
                    width: width as u32,
                    height: height as u32,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                normal_data,
                bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
            );
        }
    };

    for y in 0..height {
        for x in 0..width {
            // sample neighboring heights
            let h_north = sample_height(height_data, width, height, x, y + 1);
            let h_south = sample_height(height_data, width, height, x, y.wrapping_sub(1));
            let h_east = sample_height(height_data, width, height, x + 1, y);
            let h_west = sample_height(height_data, width, height, x.wrapping_sub(1), y);
            
            let u = x as f32 / (width - 1) as f32;
            let v = y as f32 / (height - 1) as f32;

            // calculate world positions
            let pos_north = height_to_world_position(u, wrap_v(v + 1.0 / height as f32), h_north);
            let pos_south = height_to_world_position(u, wrap_v(v - 1.0 / height as f32), h_south);
            let pos_east = height_to_world_position(wrap_u(u + 1.0 / width as f32), v, h_east);
            let pos_west = height_to_world_position(wrap_u(u - 1.0 / width as f32), v, h_west);
            
            // calculate tangent vectors
            let tangent_ns = (pos_north - pos_south).normalize();
            let tangent_ew = (pos_east - pos_west).normalize();

            let normal = tangent_ns.cross(tangent_ew).normalize();

            // to 255 range
            let r = ((normal.x + 1.0) * 0.5 * 255.0) as u8;
            let g = ((normal.y + 1.0) * 0.5 * 255.0) as u8; 
            let b = ((normal.z + 1.0) * 0.5 * 255.0) as u8;

            let pixel_index = (y * width + x) * 4;
            normal_data[pixel_index] = r;     // X component
            normal_data[pixel_index + 1] = g; // Y component  
            normal_data[pixel_index + 2] = b; // Z component
            normal_data[pixel_index + 3] = 255; // Alpha
        }
    }

    // create new image
    Image::new(
        Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        normal_data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
}


/// Sample height from a texture
/// wrapping implemented
fn sample_height(
    height_data: &[u8], 
    width: usize, 
    height: usize, 
    x: usize, 
    y: usize
) -> f32 {
    // wrap X coordinate
    let wrapped_x = x % width;
    let wrapped_y = y.min(height - 1);

    let pixel_index = (wrapped_y * width + wrapped_x) * 4;

    if pixel_index < height_data.len() {
        height_data[pixel_index] as f32 / 255.0
    } else {
        0.0
    }
}

/// Convert UV coordinates and height to world position on sphere
fn height_to_world_position(u: f32, v: f32, height: f32) -> Vec3 {
    // UV to longitude/latitude
    let longitude = (u * 2.0 - 1.0) * std::f32::consts::PI; // -π to π
    let latitude = (0.5 - v) * std::f32::consts::PI; // -π/2 to π/2
    
    // apply height displacement  
    let radius = EARTH_RADIUS + height * crate::config::DISPLACEMENT_SCALE;
    
    let x = radius * latitude.cos() * longitude.cos();
    let y = radius * latitude.sin();
    let z = radius * latitude.cos() * longitude.sin();
    
    Vec3::new(x, y, z)
}

/// Wrap U coordinate (longitude)
fn wrap_u(u: f32) -> f32 {
    if u < 0.0 {
        u + 1.0
    } else if u > 1.0 {
        u - 1.0
    } else {
        u
    }
}

/// Clamp V coordinate (latitude)
fn wrap_v(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

// HELPERS

/// save normal map to an image file 
pub fn save_image_as_png(image: &Image, path: &str) {
    let width = image.texture_descriptor.size.width;
    let height = image.texture_descriptor.size.height;

    // check if image format compatible
    assert!(
        image.texture_descriptor.format == TextureFormat::Rgba8Unorm
            || image.texture_descriptor.format == TextureFormat::Rgba8UnormSrgb,
        "Unsupported texture format for saving"
    );

    // extract raw RGBA8 data
    let data = image.data.as_ref().expect("Image data is missing");

    // convert to ImageBuffer
    let buffer: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(width, height, data.to_vec())
        .expect("Failed to create ImageBuffer");

    // save to disk
    buffer.save(path).expect("Failed to save image");
}