use bevy::prelude::*;

pub struct GlobeUIPlugin;

impl Plugin for GlobeUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
           .add_systems(Update, update_satellite_count);
    }
}

// UI component to display satellite count
#[derive(Component)]
pub struct SatelliteCounter;

use crate::systems::tle::Satellite;

fn setup(mut commands: Commands) {
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