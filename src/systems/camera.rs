use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_orbit_camera);
    }
}

// camera component
#[derive(Component, Debug)]
pub struct OrbitCamera {
    pub radius: f32,
    pub speed: f32, 
    pub angle: f32,
    pub v_angle: f32,
    pub is_dragging: bool,
    pub target: Vec3,

    pub min_radius: f32,
    pub max_radius: f32,

    // smoothing values
    target_radius: f32,
    target_angle: f32,
    target_v_angle: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            radius: 15.0,
            speed: 0.1,
            angle: 0.0,
            v_angle: 0.3,
            is_dragging: false,
            target: Vec3::ZERO,

            min_radius: 0.0,
            max_radius: 1000.0,

            target_radius: 15.0,
            target_angle: 0.0,
            target_v_angle: 0.3,
        }
    }
}

impl OrbitCamera {
    pub fn new(radius: f32, speed: f32) -> Self {
        Self {
            radius,
            speed,
            target_radius: radius,
            ..default()
        }
    }

    // set target point that for the camera to orbit
    // to be used/implemented itf
    pub fn with_target(mut self, target: Vec3) -> Self {
        self.target = target;
        self
    }

    // allow custom zoom limits
    // to implement when switching targets
    pub fn with_zoom_limits(mut self, min_radius: f32, max_radius: f32) -> Self {
        self.min_radius = min_radius;
        self.max_radius = max_radius;
        self
    }

    // calculate world position from spherical coordinates
    // https://en.wikipedia.org/wiki/Spherical_coordinate_system#Cartesian_coordinates
    pub fn calculate_position(&self) -> Vec3 {
        let x = self.radius * self.v_angle.cos() * self.angle.cos();
        let y = self.radius * self.v_angle.sin();
        let z = self.radius * self.v_angle.cos() * self.angle.sin();
        
        self.target + Vec3::new(x, y, z)
    }
}

fn update_orbit_camera(
    mut camera_query: Query<(&mut Transform, &mut OrbitCamera)>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<CursorMoved>,
    mut scroll_events: EventReader<MouseWheel>,
    time: Res<Time>,
) {
    const ZOOM_SPEED: f32 = 1400.0;
    const SMOOTH_SPEED: f32 = 10.0;

    for (mut transform, mut camera) in camera_query.iter_mut() {
        // handle mouse drag
        if mouse_buttons.just_pressed(MouseButton::Right) {
            camera.is_dragging = true;
        }
        if mouse_buttons.just_released(MouseButton::Right) {
            camera.is_dragging = false;
        }

        // update camera angles
        if camera.is_dragging {
            for motion in mouse_motion.read() {
                if let Some(delta) = motion.delta {
                    camera.target_angle += delta.x * camera.speed * 0.01;
                    camera.target_v_angle += delta.y * camera.speed * 0.01;
                }
                // clamp pitch on the target value
                camera.target_v_angle = camera.target_v_angle.clamp(-1.5, 1.5);
            }
        }

        // handle mouse scroll
        for scroll in scroll_events.read() {
            // zoom speed
            // TODO: expose functionality
            camera.target_radius -= scroll.y * ZOOM_SPEED;
            camera.target_radius = camera.target_radius.clamp(camera.min_radius, camera.max_radius);
        }

        // interpolate actual values towards targets
        let dt = time.delta_secs();
        camera.angle += (camera.target_angle - camera.angle) * dt * SMOOTH_SPEED;
        camera.v_angle += (camera.target_v_angle - camera.v_angle) * dt * SMOOTH_SPEED;
        camera.radius += (camera.target_radius - camera.radius) * dt * SMOOTH_SPEED;

        // update camera position/orientation
        transform.translation = camera.calculate_position();
        transform.look_at(camera.target, Vec3::Y);
    }
}