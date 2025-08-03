use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use crate::systems::earth::uv::LatLon;
use crate::config::{DISPLACEMENT_SCALE, EARTH_RADIUS};


/// Generates a spherical mesh face by projecting a flat grid onto a sphere
/// Based on Sebastian Lague and Grayson Head's implementation
pub fn generate_face(
    normal: Vec3,
    resolution: u32,
    x_offset: f32,
    y_offset: f32,
    displacement: Option<&Image>, // optional displacement map
) -> Mesh {
    // this creates two perpendicular axes on the cube face
    let axis_a = Vec3::new(normal.y, normal.z, normal.x);
    let axis_b = axis_a.cross(normal);

    // TODO: optimize memory creation (use pre-defined capacity)
    let mut vertices: Vec<Vec3> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    let mut uvs: Vec<Vec2> = Vec::new();

    let mut first_longitude = 0.0;

    // create a grid of vertices
    for y in 0..resolution {
        for x in 0..resolution {
            // traverse
            let i = x + y * resolution;

            let percent = Vec2::new(x as f32, y as f32) / (resolution - 1) as f32;
            let point_on_unit_cube = 
                normal + (percent.x - x_offset) * axis_a + (percent.y - y_offset) * axis_b;
            let point_on_unit_sphere = cube_point_to_sphere_point(point_on_unit_cube);

            // uv
            let point_coords = LatLon::from(point_on_unit_sphere.normalize());
            let (lat, lon) = point_coords.as_degrees();

            // scale to size
            // let final_point = point_on_unit_sphere.normalize() * EARTH_RADIUS; // 'normalize' makes it spherical
            // vertices.push(final_point);
            // normals.push(point_on_unit_sphere.normalize());

            let (mut u, v) = point_coords.to_uv();

            // handle UV seam cases to prevent texture distortion
            if y == 0 && x == 0 {
                first_longitude = lon;
            }

            // implement wrapping, as polygons may have to cross texture edge
            // if starting on negative -long crossing into +long, set u to 0.0
            if first_longitude < 0.0 && lon > 0.0 && lat < 89.0 && lat > -89.0 {
                u = 0.0;
            }

            // if below -40 degrees lat and tile starts at 180, set u to 0.0
            if x == 0 && lon == 180.0 && lat < -40.0 {
                u = 0.0;
            }

            // sample displacement
            let displacement = if let Some(disp_map) = displacement {
                sample_displacement(disp_map, u, v) * DISPLACEMENT_SCALE
            } else {
                0.0
            };

            // apply displacement
            let radius = EARTH_RADIUS + displacement;
            let final_point = point_on_unit_sphere.normalize() * radius;

            vertices.push(final_point);
            normals.push(point_on_unit_sphere.normalize());
            uvs.push(Vec2::new(u, v));

            // build triangles
            if x != resolution - 1 && y != resolution - 1 {
                // triangle 1
                indices.push(i);
                indices.push(i + resolution);
                indices.push(i + resolution + 1);

                // triangle 2
                indices.push(i);
                indices.push(i + resolution + 1);
                indices.push(i + 1);
            }
        }
    }

    // after generating vertices, recalculate normals
    // this is to make sure the normals account for the displacement
    // recalculate_normals(&mut normals, &vertices, &indices);

    // the problem with recalculating normals is that it messes up the day/night shader
    // i.e. cities on mountain sides becomes bright earlier/later than they should

    // build bevy mesh
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList, 
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages:: MAIN_WORLD
    );
    mesh.insert_indices(Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.generate_tangents().unwrap(); // need this for normal mapping

    mesh
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

/// Recalculate normals based on actual mesh geometry
#[allow(dead_code)]
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

    0.0 // default if not available, or out of bounds
}