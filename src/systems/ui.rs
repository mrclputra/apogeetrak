use bevy::prelude::*;
use rand::Rng;

pub struct GlobeUIPlugin;

impl Plugin for GlobeUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(Update, (ui_randomize, ui_coordinates));
    }
}

// UI components
#[derive(Component)]
pub struct RandomizeButton;
#[derive(Component)]
pub struct CoordinateDisplay;

use crate::LatLong;

fn setup(mut commands: Commands) {
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

fn ui_randomize(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<RandomizeButton>)>,
    mut marker_query: Query<(&mut Transform, &mut LatLong)>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // generate random lat/lon coordinates
            let mut rng = rand::rng();
            let new_lat = rng.random_range(-90.0..=90.0);
            let new_lon = rng.random_range(-180.0..=180.0);
            
            // update marker position
            for (mut transform, mut marker) in marker_query.iter_mut() {
                marker.latitude = new_lat;
                marker.longitude = new_lon;
                
                transform.translation = crate::latlon_to_pos(new_lat, new_lon, 5.0);
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