use bevy::{prelude::*, transform};

fn main() -> bevy::app::AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run()
}

// camera component
#[derive(Component, Debug)]
struct Camera {
    radius: f32,
    speed: f32,
    angle: f32
}

// scene setup here
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let globe_size = 5.0;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(globe_size, globe_size, globe_size))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // sun light
    // need to make it based on real sun position
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 2000.0,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // spawn camera 
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            radius: 15.0,  // set initial orbit radius
            speed: 0.5,    // orbit speed
            angle: 0.0,    // starting angle
        },
    ));

}

// update
fn update(
    mut camera_query: Query<(&mut Transform, &mut Camera)>,
    time: Res<Time>,
) {
    for (mut transform, mut camera) in camera_query.iter_mut() {
        // increment the orbit angle based on time
        camera.angle += camera.speed * time.delta_secs();
        
        // calculate new position using circular motion
        let x = camera.radius * camera.angle.cos();
        let z = camera.radius * camera.angle.sin();
        let y = 5.0; // keep camera slightly above for better view
        
        // move camera and keep it looking at the cube
        transform.translation = Vec3::new(x, y, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}