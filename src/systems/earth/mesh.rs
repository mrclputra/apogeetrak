/// This file is used to create the mesh for the Earth object
/// The approach taken was to map a cube to a sphere, this is so that we have better UVs at the poles and more geometry control for displacement operations
/// Should return the finished product

/// displacement should be applied here on start, not during runtime 
 
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use crate::systems::earth::uv::LatLon;
use crate::config::{DISPLACEMENT_SCALE, EARTH_RADIUS};

// raw mesh data for a sphere face
struct FaceMeshData {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u32>,
}

/// Generates a spherical mesh face by projecting a flat grid onto a sphere
/// This is based on Sebastian Lague and Grayson Head's implementation
fn generate_face(
    normal: Vec3,
    resolution: u32,
    x_offset: f32,
    y_offset: f32,
    displacement: Option<&Image>
) -> FaceMeshData {
    let axis_a = Vec3::new(normal.y, normal.z, normal.x);
    let axis_b = axis_a.cross(normal);

    let mut vertices: Vec<Vec3> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    let mut uvs: Vec<Vec2> = Vec::new();

    let mut face_start_longitude = 0.0;
    let mut first_point_set = false;

    // Create a grid of vertices
    for y in 0..resolution {
        for x in 0..resolution {
            let i = x + y * resolution;
            
            let percent = Vec2::new(x as f32, y as f32) / (resolution - 1) as f32;
            let point_on_unit_cube = 
                normal + (percent.x - x_offset) * axis_a + (percent.y - y_offset) * axis_b;
            let point_on_unit_sphere = cube_point_to_sphere_point(point_on_unit_cube);

            // Calculate UV coordinates
            let point_coords = LatLon::from(point_on_unit_sphere.normalize());
            let (lat, lon) = point_coords.as_degrees();
            let (base_u, v) = point_coords.to_uv();

            // Set reference longitude from first vertex
            if !first_point_set {
                face_start_longitude = lon;
                first_point_set = true;
            }

            // Apply enhanced seam handling
            let (u, v) = handle_uv_seam(base_u, v, lon, lat, face_start_longitude);

            // Sample displacement (unchanged)
            let displacement = if let Some(disp_map) = displacement {
                sample_displacement(disp_map, u.clamp(0.0, 1.0), v) * DISPLACEMENT_SCALE
            } else {
                0.0
            };

            // Apply displacement
            let radius = EARTH_RADIUS + displacement;
            let final_point = point_on_unit_sphere.normalize() * radius;

            vertices.push(final_point);
            normals.push(point_on_unit_sphere.normalize());
            uvs.push(Vec2::new(u, v));

            // Build triangles (unchanged)
            if x != resolution - 1 && y != resolution - 1 {
                indices.push(i);
                indices.push(i + resolution);
                indices.push(i + resolution + 1);

                indices.push(i);
                indices.push(i + resolution + 1);
                indices.push(i + 1);
            }
        }
    }

    // Recalculate normals (unchanged)
    recalculate_normals(&mut normals, &vertices, &indices);

    FaceMeshData {
        vertices,
        normals,
        uvs,
        indices,
    }
}

pub fn generate_earth_mesh(
    resolution: u32,
    displacement_image: Option<&Image>,
) -> Mesh {
    // generate face should be called inside this function iteratively
    // displacement generation call done here

    let faces = vec![
        Vec3::X,        // right
        Vec3::NEG_X,    // left
        Vec3::Y,        // top
        Vec3::NEG_Y,    // bottom
        Vec3::Z,        // front
        Vec3::NEG_Z,    // back
    ];

    let offsets = vec![
        (0.0, 0.0), 
        (0.0, 1.0), 
        (1.0, 0.0), 
        (1.0, 1.0)
    ];

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for direction in faces {
        for offset in &offsets {
            let face_data = generate_face(
                direction,
                resolution,
                offset.0,
                offset.1,
                displacement_image 
            );

            let vertex_offset = vertices.len() as u32;

            vertices.extend(face_data.vertices);
            normals.extend(face_data.normals);
            uvs.extend(face_data.uvs);

            // indices need to be offset to point to the right vertices
            for index in face_data.indices {
                indices.push(index + vertex_offset);
            }
        }
    }

    // merge overlapping vertices
    (vertices, normals, uvs, indices) = 
        merge_vertices(vertices, normals, uvs, indices);

    // build mesh
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD
    );

    mesh.insert_indices(Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.generate_tangents().unwrap();

    mesh
}

/// Recalculate normals based on actual mesh geometry
fn recalculate_normals(normals: &mut Vec<Vec3>, vertices: &[Vec3], indices: &[u32]) {
    // reset normals
    normals.fill(Vec3::ZERO);

    for triangle in indices.chunks(3) {
        if triangle.len() == 3 {
            let i0 = triangle[0] as usize;
            let i1 = triangle[1] as usize;
            let i2 = triangle[2] as usize;

            let v0 = vertices[i0];
            let v1 = vertices[i1];
            let v2 = vertices[i2];

            // calculate face normal
            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let mut face_normal = edge1.cross(edge2);

            // check for degenrate triangle
            let face_normal_length = face_normal.length();
            if face_normal_length > 1e-6 {
                face_normal = face_normal / face_normal_length;

                // add face normal to each vertex normal
                normals[i0] += face_normal;
                normals[i1] += face_normal;
                normals[i2] += face_normal;
            }
        }
    }

    // normalize all vertex normals
    for normal in normals.iter_mut() {
        let length = normal.length();
        if length > 1e-6 {
            *normal = *normal / length;
        } else {
            // fallback for isolated vertices
            *normal = Vec3::Y;
        }
    }
}

/// Sample displacement value from image at UV coordinates
/// white = high, black = low
fn sample_displacement(image: &Image, u: f32, v: f32) -> f32 {
    // clamp UV coordinates
    let u = u.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);

    let width = image.texture_descriptor.size.width as usize;
    let height = image.texture_descriptor.size.height as usize;

    // UV to pixel coordinates
    let x = (u * (width - 1) as f32).round() as usize;
    let y = (v * (height - 1) as f32).round() as usize;

    // get pixel data slice
    if let Some(data) = image.data.as_ref() {
        let pixel_index = (y * width + x) * 4; // 4 bytes per pixel (RGBA)

        if pixel_index + 3 < data.len() {
            // just used red channel
            return data[pixel_index] as f32 / 255.0;
        }
    }

    0.0 // default
}

/// Converts a point on a unit cube to the corresponding point on a unit sphere
/// creates more even distribution of sphere surface
/// https://mathproofs.blogspot.com/2005/07/mapping-cube-to-sphere.html
fn cube_point_to_sphere_point(p: Vec3) -> Vec3 {
    let x2 = p.x * p.x;
    let y2 = p.y * p.y;
    let z2 = p.z * p.z;

    let x = (1.0 - y2 / 2.0 - z2 / 2.0 + (y2 * z2) / 3.0).max(0.0);
    let y = (1.0 - z2 / 2.0 - x2 / 2.0 + (z2 * x2) / 3.0).max(0.0);
    let z = (1.0 - x2 / 2.0 - y2 / 2.0 + (x2 * y2) / 3.0).max(0.0);

    Vec3 {
        x: p.x * x.sqrt(),
        y: p.y * y.sqrt(),
        z: p.z * z.sqrt(),
    }
}

/// Merges vertices that are close together
/// returns cleaned up mesh data
fn merge_vertices(
    vertices: Vec<Vec3>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
    indices: Vec<u32>
) -> (Vec<Vec3>, Vec<Vec3>, Vec<Vec2>, Vec<u32>) {
    let position_tolerance = 0.01;
    let uv_tolerance = 0.1; // UV coordinates must also be close to merge

    let mut m_vertices = Vec::new();
    let mut m_normals = Vec::new();
    let mut m_uvs = Vec::new();
    let mut vertex_mapping = vec![0u32; vertices.len()];

    // Find or create merged vertices - now considering UV coordinates
    for (i, vertex) in vertices.iter().enumerate() {
        let mut found_match = None;

        // Check if a similar vertex already exists
        for (j, existing_vertex) in m_vertices.iter().enumerate() {
            let position_close = vertex.distance(*existing_vertex) < position_tolerance;
            let uv_close = uvs[i].distance(m_uvs[j]) < uv_tolerance;
            
            // Only merge if BOTH position AND UV are close
            if position_close && uv_close {
                found_match = Some(j);
                break;
            }
        }

        if let Some(existing_index) = found_match {
            // Map to existing vertex
            vertex_mapping[i] = existing_index as u32;
        } else {
            // Add new vertex - this preserves UV seams
            vertex_mapping[i] = m_vertices.len() as u32;
            m_vertices.push(*vertex);
            m_normals.push(normals[i]);
            m_uvs.push(uvs[i]);
        }
    }

    // Remap indices
    let m_indices = indices.iter()
        .map(|&index| vertex_mapping[index as usize])
        .collect();

    (m_vertices, m_normals, m_uvs, m_indices)
}

/// Enhanced UV seam handling for longitude wrapping
/// This should be called in generate_face instead of the current seam handling
fn handle_uv_seam(u: f32, v: f32, lon: f32, lat: f32, face_start_lon: f32) -> (f32, f32) {
    let mut fixed_u = u;
    
    // Handle International Date Line crossing (180°/-180°)
    let lon_diff = (lon - face_start_lon).abs();
    
    // If longitude difference is > 180°, we're crossing the date line
    if lon_diff > 180.0 {
        // Determine which side of the seam we want to be on
        if face_start_lon > 90.0 && lon < -90.0 {
            // Face starts in positive longitude, current point is negative
            // Map negative longitude to > 1.0 to prevent wrapping
            fixed_u = 1.0 + (lon + 180.0) / 360.0;
        } else if face_start_lon < -90.0 && lon > 90.0 {
            // Face starts in negative longitude, current point is positive  
            // Map positive longitude to < 0.0
            fixed_u = (lon - 180.0) / 360.0;
        }
    }
    
    // Ensure we're not at exactly 0.0 or 1.0 to avoid edge artifacts
    fixed_u = fixed_u.clamp(0.001, 0.999);
    
    (fixed_u, v)
}