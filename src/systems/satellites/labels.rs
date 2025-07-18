use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::window::Window;

use crate::systems::satellites::Satellite;
use crate::config::EARTH_RADIUS;

// full ui screen container component
#[derive(Component)]
pub struct LabelContainer;

// individual satellite labels
#[derive(Component)]
pub struct SatelliteLabel {
    pub satellite_entity: Entity,
    // add more attributes here (size, offset, etc)
}

// setup UI overlay
pub fn setup_labels(mut commands: Commands) {
    // create UI container covering entire screen
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
        LabelContainer,
    ));
}

pub fn update_labels(
    mut commands: Commands,
    satellites: Query<(Entity, &Transform, &Satellite)>,
    camera: Query<(&Camera, &Transform)>,
    mut labels: Query<(Entity, &mut Node, &mut Visibility, &SatelliteLabel)>,
    container: Query<Entity, With<LabelContainer>>,
    window: Query<&Window>,
) {
    let (Ok(window), Ok((camera, cam_transform)), Ok(container)) = 
        (window.single(), camera.single(), container.single()) else { return; };

    // map existing labels by satellite entity
    let existing_labels: HashMap<Entity, Entity> = labels.iter()
        .map(|(label_entity, _, _, sat_label)| (sat_label.satellite_entity, label_entity))
        .collect();

    // process each satellite
    for (sat_entity, sat_transform, satellite) in satellites.iter() {
        let sat_pos = sat_transform.translation;

        // check visibility, get screen position
        let visible = is_visible(sat_pos, cam_transform.translation, Vec3::ZERO, EARTH_RADIUS);
        let screen_pos = world_to_screen(sat_pos, camera, cam_transform, window.width(), window.height());

        let should_show = visible && screen_pos.is_some();

        if let Some(&label_entity) = existing_labels.get(&sat_entity) {
            // update existing label
            if let Ok((_, mut node, mut visibility, _)) = labels.get_mut(label_entity) {
                if should_show {
                    let pos = screen_pos.unwrap(); // known Some
                    *visibility = Visibility::Inherited;
                    node.left = Val::Px(pos.x);
                    node.top = Val::Px(pos.y);
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        } else if should_show {
            // create new label
            let pos = screen_pos.unwrap(); // known Some
            let label_text = format!("{}\nAlt: {:.0}km", 
                satellite.name(), 
                satellite.current_geodetic_position().2, // altitude
                // sat_pos.length() - EARTH_RADIUS
            );

            commands.entity(container).with_children(|parent| {
                parent.spawn((
                    Text::new(label_text),
                    TextFont { font_size: 8.0, ..default() },
                    TextColor(Color::WHITE),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(pos.x),
                        top: Val::Px(pos.y),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)), // textbox background
                    SatelliteLabel { satellite_entity: sat_entity },
                ));
            });
        }
    }
}

// UTILS

// convert world coordinates to screen coordinates
fn world_to_screen(
    world_pos: Vec3,
    camera: &Camera,
    camera_transform: &Transform,
    screen_width: f32,
    screen_height: f32,
) -> Option<Vec2> {
    let view_matrix = camera_transform.compute_matrix().inverse();
    let view_projection = camera.clip_from_view() * view_matrix;

    // transform to clip space
    let clip_pos = view_projection * Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);
    
    if clip_pos.w <= 0.0 { return None; } // behind camera

    // convert to NDC and check bounds
    let ndc = clip_pos.xyz() / clip_pos.w;
    if ndc.x.abs() > 1.0 || ndc.y.abs() > 1.0 { return None; } // offscreen

    // NDC to screen coordinates
    Some(Vec2::new(
        (ndc.x + 1.0) * 0.5 * screen_width,
        (1.0 - ndc.y) * 0.5 * screen_height, // Y is flipped
    ))
}

// check if satellite is visible from camera (unblocked by earth)
// simple ray-sphere intersection test, tbf
fn is_visible(sat_pos: Vec3, cam_pos: Vec3, earth_center: Vec3, earth_radius: f32) -> bool {
    let cam_to_sat = sat_pos - cam_pos;
    let cam_to_earth = earth_center - cam_pos;
    
    let projection = cam_to_earth.dot(cam_to_sat.normalize());
    if projection < 0.0 || projection > cam_to_sat.length() { return true; }

    let closest_point = cam_pos + cam_to_sat.normalize() * projection;
    (closest_point - earth_center).length() > earth_radius
}