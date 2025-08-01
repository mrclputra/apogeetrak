use bevy::prelude::*;
use chrono::Utc;

use crate::systems::satellites::Satellite;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
           .add_systems(Update, (update_satellite_count, update_datetime));
    }
}

// UI component to display satellite count
#[derive(Component)]
pub struct SatelliteCounter;

// UI component to display current datetimme
#[derive(Component)]
pub struct DateTimeDisplay;

fn setup_ui(mut commands: Commands) {
    // create UI container
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
            // display satellite count
            parent.spawn((
                Text::new("Satellites: Loading..."),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                SatelliteCounter,
            ));

            // display datetime
            parent.spawn((
                Text::new("Time: Loading..."),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                DateTimeDisplay,
                Node {
                    margin: UiRect::top(Val::Px(5.0)), // spacing
                    ..default()
                },
            ));
        });
}

// update the satellite count display
fn update_satellite_count(
    satellite_query: Query<&Satellite>,
    mut text_query: Query<&mut Text, With<SatelliteCounter>>,
) {
    let count = satellite_query.iter().count();
    
    if let Ok(mut text) = text_query.single_mut() {
        text.0 = format!("Satellites: {}", count);
    }
}

// update the datetime display with current UTC time
fn update_datetime(
    mut text_query: Query<&mut Text, With<DateTimeDisplay>>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        // Create a fixed datetime (e.g., 2025-01-01 12:00:00 UTC)
        let fixed_datetime = chrono::DateTime::parse_from_rfc3339("2000-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        
        // format
        text.0 = format!("Time: {} UTC", fixed_datetime.format("%Y-%m-%d %H:%M:%S"));
    }
}