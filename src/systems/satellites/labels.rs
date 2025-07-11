use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::window::Window;

use crate::systems::satellites::tle::Satellite;
use crate::EARTH_RADIUS;

pub struct LabelsPlugin;

impl Plugin for LabelsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(Update, update);
    }
}

// all screen ui labels container component
#[derive(Component)]
pub struct LabelContainer;

// individual satellite labels component
#[derive(Component)]
pub struct SatelliteLabel {
    pub satellite_entity: Entity,
    pub offset: Vec2, // screen-space offset
}

// setup UI overlay
fn setup(mut commands: Commands) {
    // create UI container covering entire screen
    // holds all label components as children
    commands
        .spawn((
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

fn update(
    mut commands: Commands,
    satellite_query: Query<(Entity, &Transform, &Satellite)>,                       // query all satellites in the world
    camera_query: Query<(&Camera, &Transform)>,                                     // query main camera
    mut label_query: Query<(Entity, &mut Node, &mut Visibility, &SatelliteLabel)>,  // query existing labels
    label_container_query: Query<Entity, With<LabelContainer>>,                     // query label screen
    window_query: Query<&Window>,                                                   // get app window
) {
    let Ok(window) = window_query.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    let Ok(label_container) = label_container_query.single() else {
        return;
    };

    // create a map of existing labels by satellite entity
    let mut existing_labels: std::collections::HashMap<Entity, Entity> = std::collections::HashMap::new();
    for (label_entity, _, _, satellite_label) in label_query.iter() {
        existing_labels.insert(satellite_label.satellite_entity, label_entity);
    }

    // process each satellite
    for (satellite_entity, satellite_transform, satellite) in satellite_query.iter() {
        let satellite_pos = satellite_transform.translation;

        // check if visible (not blocked by Earth)
        let is_blocked = !is_visible(
            satellite_pos,
            camera_transform.translation,
            Vec3::ZERO, // earth center
            EARTH_RADIUS
        );

        // convert 3d position to screen coordinates
        let screen_pos = world_to_screen(
            satellite_pos,
            camera,
            camera_transform,
            window.width(),
            window.height()
        );

        // check if satellite is offscreen (outside of view frustrum)
        let is_ofscreen = screen_pos.is_none();

        if is_blocked || is_ofscreen {
            if let Some(&label_entity) = existing_labels.get(&satellite_entity) {
                if let Ok((_, _, mut visibility, _)) = label_query.get_mut(label_entity) {
                    *visibility = Visibility::Hidden;
                }
            }
            continue;
        }

        // known Some
        let screen_pos = screen_pos.unwrap();

        // check if there is already a label
        if let Some(&label_entity) = existing_labels.get(&satellite_entity) {
            // update existing label
            if let Ok((_, mut node, mut visibility, _)) = label_query.get_mut(label_entity) {
                *visibility = Visibility::Inherited;
                node.left = Val::Px(screen_pos.x);
                node.top = Val::Px(screen_pos.y);
            }
        } else {
            // create a new label
            let label_text = format!("{}\nAlt: {:.0}km", 
                satellite.name, 
                satellite_pos.length() - EARTH_RADIUS
            );

            commands.entity(label_container).with_children(|parent| {
                parent.spawn((
                    Text::new(label_text),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(screen_pos.x),
                        top: Val::Px(screen_pos.y),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)), // textbox background
                    SatelliteLabel {
                        satellite_entity,
                        offset: Vec2::new(10.0, -10.0), // label offset 
                    },
                ));
            });
        }
    }

    // clean up labels for satellites that no longer exist
    let satellite_entities: std::collections::HashSet<Entity> = satellite_query.iter()
        .map(|(entity, _, _)| entity)
        .collect();

    for (label_entity, _, _, satellite_label) in label_query.iter() {
        if !satellite_entities.contains(&satellite_label.satellite_entity) {
            commands.entity(label_entity).despawn();
        }
    }
}

// UTILS

// convert world coordinates to screen coordinates
// https://en.wikipedia.org/wiki/3D_projection
fn world_to_screen(
    world_pos: Vec3,
    camera: &Camera,
    camera_transform: &Transform,
    screen_width: f32,
    screen_height: f32,
) -> Option<Vec2> {
    // get camera view project matrix
    let view_matrix = camera_transform.compute_matrix().inverse();
    let projection_matrix = camera.clip_from_view();
    let view_projection = projection_matrix * view_matrix;

    // transform world position to clip space
    let world_pos_homogeneous = Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);
    let clip_space_pos = view_projection * world_pos_homogeneous;

    // check if in-front of camera
    if clip_space_pos.w <= 0.0 {
        return None;
    }

    // convert to normalized device coordinates (NDC)
    // https://learnopengl.com/Getting-started/Coordinate-Systems
    let ndc = clip_space_pos.xyz() / clip_space_pos.w;

    // check if point is within screen bounds (view frustrum)
    // retun None for offscreen
    if ndc.x < -1.0 || ndc.x > 1.0 || ndc.y < -1.0 || ndc.y > 1.0 {
        return None;
    }

    // NDC back to screen-space coordinates
    let screen_x = (ndc.x + 1.0) * 0.5 * screen_width;
    let screen_y = (1.0 - ndc.y) * 0.5 * screen_height; // Y is flipped

    Some(Vec2::new(screen_x, screen_y))
}

// check if a satellite is visible from camera (unblocked by earth)
// simple ray-sphere intersection test implementation
fn is_visible(
    satellite_pos: Vec3,
    camera_pos: Vec3,
    earth_center: Vec3,
    earth_radius: f32,
) -> bool {
    // get vector from camera to satellite
    let camera_to_satellite = satellite_pos - camera_pos;
    let distance_to_satellite = camera_to_satellite.length();

    // get vector from camera to earth center
    let camera_to_earth = earth_center - camera_pos;

    let projection_length = camera_to_earth.dot(camera_to_satellite.normalize());
    if projection_length < 0.0 || projection_length > distance_to_satellite {
        return true;
    }

    // find closest point on the ray to earth's center
    let closest_point = camera_pos + camera_to_satellite.normalize() * projection_length;
    let distance_to_earth_center = (closest_point - earth_center).length();

    // radius check
    distance_to_earth_center > earth_radius
}