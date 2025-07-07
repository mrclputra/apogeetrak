use bevy::{prelude::*, transform};
use bevy::input::mouse::MouseWheel;

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
    angle: f32,
    v_angle: f32,
    is_dragging: bool,
}

// scene setup here
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // let globe_size = 5.0;

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(5.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::hex("#ffffff").unwrap().into(),
            metallic: 0.0,
            perceptual_roughness: 0.5,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // sun light
    // need to make it based on real sun position
    commands.spawn((
        DirectionalLight {
            illuminance: 1_500.,
            ..default()
        },
        Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // spawn camera 
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            radius: 15.0,           // set initial orbit radius
            speed: 0.5,             // orbit speed
            angle: 0.0,             // starting angle
            v_angle: 0.3,           // starting angle (vertical)
            is_dragging: false,
        },
    ));

}

// update
fn update(
    mut camera_query: Query<(&mut Transform, &mut Camera)>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<CursorMoved>,
    mut scroll_events: EventReader<MouseWheel>,
) {
    for (mut transform, mut camera) in camera_query.iter_mut() {
        // handle mouse drag for camera rotation
        if mouse_buttons.just_pressed(MouseButton::Right) {
            camera.is_dragging = true;
        }
        if mouse_buttons.just_released(MouseButton::Right) {
            camera.is_dragging = false;
        }

        // update camera angles when dragging
        if camera.is_dragging {
            for motion in mouse_motion.read() {
                // convert mouse movement to angle changes
                // negative delta_x rotates left/right (yaw)
                // negative delta_y rotates up/down (pitch)
                if let Some(delta) = motion.delta {
                    camera.angle += delta.x * camera.speed * 0.01;
                    camera.v_angle += delta.y * camera.speed * 0.01;
                }
                
                // keep pitch within reasonable limits so we don't flip upside down
                camera.v_angle = camera.v_angle.clamp(-1.5, 1.5);
            }
        }

        // handle scroll wheel for zoom
        for scroll in scroll_events.read() {
            // scroll up = zoom in (decrease radius)
            // scroll down = zoom out (increase radius)
            camera.radius -= scroll.y * 2.0;
            // keep zoom within sensible bounds
            camera.radius = camera.radius.clamp(3.0, 50.0);
        }

        // calculate camera position using spherical coordinates
        // this gives us proper 3D orbital movement
        let x = camera.radius * camera.v_angle.cos() * camera.angle.cos();
        let y = camera.radius * camera.v_angle.sin();
        let z = camera.radius * camera.v_angle.cos() * camera.angle.sin();
        
        // move camera and keep it looking at the globe
        transform.translation = Vec3::new(x, y, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}