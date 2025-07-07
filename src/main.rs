use bevy::prelude::*;
use rand::Rng;

// import camera
mod camera;
use camera::{OrbitCamPlugin, OrbitCamera};

fn main() -> bevy::app::AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(OrbitCamPlugin)
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_systems(Startup, setup)
        .add_systems(Update, (ui_button, ui_coordinates))
        .run()
}

#[derive(Component)]
struct LatLong {
    latitude: f32,
    longitude: f32,
}

// UI components (temp)
#[derive(Component)]
struct RandomizeButton;
#[derive(Component)]
struct CoordinateDisplay;

// convert latlon to cartesian
// need to move this somewhere else
fn latlon_to_pos(latitude: f32, longitude: f32, radius: f32) -> Vec3 {
    let lat_rad = latitude.to_radians();
    let lon_rad = longitude.to_radians();

    // spherical to cartesian conversion
    let x = radius * lat_rad.cos() * lon_rad.cos();
    let y = radius * lat_rad.sin();
    let z = radius * lat_rad.cos() * lon_rad.sin();
    
    Vec3::new(x, y, z)
}

// scene setup here
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // let globe_size = 5.0;

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(5.0).mesh().ico(32).unwrap())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::hex("#ffffff").unwrap().into(),
            metallic: 0.0,
            perceptual_roughness: 0.5,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // test marker thing
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.1).mesh().ico(8).unwrap())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::hex("ff0000").unwrap().into(),
            metallic: 0.0,
            perceptual_roughness: 0.3,
            ..default()
        })),
        Transform::from_translation(latlon_to_pos(0.0, 0.0, 5.0)),
        LatLong {
            latitude: 0.0,
            longitude: 0.0,
        },
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
        OrbitCamera::new(15.0, 0.5)
            .with_target(Vec3::ZERO)
    ));

    // create UI for controls and coordinate display
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            // button to randomize marker position
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.4, 0.8)),
                    RandomizeButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Random Location"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // text display for current coordinates
            parent.spawn((
                Text::new("Lat: 0.0°, Lon: 0.0°"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                CoordinateDisplay,
            ));
        });
}

fn ui_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<RandomizeButton>)>,
    mut marker_query: Query<(&mut Transform, &mut LatLong)>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // generate random lat/lon coordinates
            let mut rng = rand::rng();
            let new_lat = rng.random_range(-90.0..=90.0);
            let new_lon = rng.random_range(-180.0..=180.0);
            
            // update the marker position
            for (mut transform, mut marker) in marker_query.iter_mut() {
                marker.latitude = new_lat;
                marker.longitude = new_lon;
                
                // position marker slightly above globe surface to avoid z-fighting
                transform.translation = latlon_to_pos(new_lat, new_lon, 5.0);
            }
        }
    }
}

fn ui_coordinates(
    marker_query: Query<&LatLong>,
    mut text_query: Query<&mut Text, With<CoordinateDisplay>>,
) {
    if let Ok(marker) = marker_query.single() {
        if let Ok(mut text) = text_query.single_mut() {
            text.0 = format!(
                "Lat: {:.1}, Lon: {:.1}", 
                marker.latitude, 
                marker.longitude
            );
        }
    }
}