use bevy::prelude::*;

pub mod mesh;
use mesh::generate_face;

pub struct EarthPlugin;

impl Plugin for EarthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, start);
    }
}

fn start(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // generate earth mesh
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
                    generate_face(direction, 16, offset.0, offset.1),
                )),
                MeshMaterial3d(materials.add(Color::WHITE)),
            ));
        }
    }
}