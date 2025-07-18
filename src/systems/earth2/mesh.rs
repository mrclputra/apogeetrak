use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use crate::constants::EARTH_RADIUS;

/// Generates a spherical mesh face by projecting a flat grid onto a sphere
/// Based on Sebastian Lague's implementation
fn generate_face(
    normal: Vec3,
    resolution: u32,
    x_offset: f32,
    y_offset: f32,
) -> Mesh {
    // this creates two perpendicular axes on the cube face
    let axis_a = Vec3::new(normal.y, normal.z, normal.x);
    let axis_b = axis_a.cross(normal);

    // TODO: optimize memory creation (use pre-defined capacity)
    let mut vertices: Vec<Vec3> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();

    // create a grid of vertices
    for y in 0..resolution {
        for x in 0..resolution {
            // traverse
            let i = x + y * resolution;

            let percent = Vec2::new(x as f32, y as f32) / (resolution - 1) as f32;
            let point_on_unit_cube = 
                normal + (percent.x - x_offset) * axis_a + (percent.y - y_offset) * axis_b;
            let point_on_unit_sphere = cube_point_to_sphere_point(point_on_unit_cube);

            // scale to size
            let final_point = point_on_unit_sphere.normalize() * EARTH_RADIUS; // 'normalize' makes it spherical
            vertices.push(final_point);
            normals.push(point_on_unit_sphere.normalize());

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

    // build bevy mesh
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList, 
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages:: MAIN_WORLD
    );
    mesh.insert_indices(Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    // mesh.generate_tangents().unwrap(); // need this for normal mapping (TODO)

    mesh
}

pub fn generate_sphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // asset_server: Res<AssetServer>,
) {
    let faces = vec![
        Vec3::X,        // right
        Vec3::NEG_X,    // left
        Vec3::Y,        // top
        Vec3::NEG_Y,    // bottom
        Vec3::Z,        // front
        Vec3::NEG_Z,    // back
    ];

    let offsets = vec![(0.0, 0.0), (0.0, 1.0), (1.0, 0.0), (1.0, 1.0)];

    for direction in faces {
        for offset in &offsets {
            commands.spawn((
                Mesh3d(meshes.add(
                    generate_face(direction, 32, offset.0, offset.1),
                )),
                MeshMaterial3d(materials.add(Color::WHITE)),
            ));
        }
    }
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