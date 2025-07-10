use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;

pub struct OrbitCamPlugin;

impl Plugin for OrbitCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
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

            // min_radius: EARTH_RADIUS + 1000.0,
            // max_radius: EARTH_RADIUS + 20000.0
        }
    }
}

impl OrbitCamera {
    pub fn new(radius: f32, speed: f32) -> Self {
        Self {
            radius,
            speed,
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

fn update(
    mut camera_query: Query<(&mut Transform, &mut OrbitCamera)>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<CursorMoved>,
    mut scroll_events: EventReader<MouseWheel>,
) {
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
                    camera.angle += delta.x * camera.speed * 0.01;
                    camera.v_angle += delta.y * camera.speed * 0.01;
                }
                // clamp pitch
                camera.v_angle = camera.v_angle.clamp(-1.5, 1.5);
            }
        }

        // handle mouse scroll
        for scroll in scroll_events.read() {
            camera.radius -= scroll.y * 170.0;
            camera.radius = camera.radius.clamp(camera.min_radius, camera.max_radius);
        }

        // update camera position/orientation
        transform.translation = camera.calculate_position();
        transform.look_at(camera.target, Vec3::Y);
    }
}