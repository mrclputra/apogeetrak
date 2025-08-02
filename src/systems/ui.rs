use bevy::prelude::*;
use chrono::Utc;

use crate::systems::satellites::Satellite;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, start)
           .add_systems(Update, (
                update_satellite_count, 
                update_datetime, 
                handle_time_control,
            ));
    }
}

// time control state resource
#[derive(Resource)]
pub struct TimeState {
    pub is_paused: bool,
    pub speed_mult: f64,
    pub sim_time: chrono::DateTime<Utc>,
}

impl Default for TimeState {
    fn default() -> Self {
        Self {
            is_paused: false,
            speed_mult: 1.0,
            sim_time: chrono::DateTime::parse_from_rfc3339("2000-01-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        }
    }
}

// UI component to display satellite count
#[derive(Component)]
pub struct SatelliteCounter;

// UI component to display current datetimme
#[derive(Component)]
pub struct DateTimeDisplay;

// time control button components
#[derive(Component)]
pub struct ResetButton;

#[derive(Component)]
pub struct BackwardButton;

#[derive(Component)]
pub struct ForwardButton;

pub fn start(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    // initialize time state
    commands.insert_resource(TimeState::default());

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

            // time control buttons container
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_children(|buttons_parent| {
                // backward button
                    buttons_parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(32.0),
                                height: Val::Px(32.0),
                                margin: UiRect::right(Val::Px(5.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                            BorderRadius::all(Val::Px(4.0)),
                            BackwardButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                ImageNode::new(asset_server.load("textures/icons/backward.png")),
                                Node {
                                    width: Val::Px(24.0),
                                    height: Val::Px(24.0),
                                    ..default()
                                },
                            ));
                        });

                    // reset button
                    buttons_parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(32.0),
                                height: Val::Px(32.0),
                                margin: UiRect::right(Val::Px(5.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                            BorderRadius::all(Val::Px(4.0)),
                            ResetButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                ImageNode::new(asset_server.load("textures/icons/reset.png")),
                                Node {
                                    width: Val::Px(24.0),
                                    height: Val::Px(24.0),
                                    ..default()
                                },
                            ));
                        });

                    // forward button
                    buttons_parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(32.0),
                                height: Val::Px(32.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                            BorderRadius::all(Val::Px(4.0)),
                            ForwardButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                ImageNode::new(asset_server.load("textures/icons/forward.png")),
                                Node {
                                    width: Val::Px(24.0),
                                    height: Val::Px(24.0),
                                    ..default()
                                },
                            ));
                        });
            });
        });
}

fn handle_time_control(
    mut time_state: ResMut<TimeState>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    backward_query: Query<&Interaction, (With<BackwardButton>, Changed<Interaction>)>,
    reset_query: Query<&Interaction, (With<ResetButton>, Changed<Interaction>)>,
    forward_query: Query<&Interaction, (With<ForwardButton>, Changed<Interaction>)>,
) {
    // visual feedback
    for (interaction, mut color) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Pressed => Color::srgba(0.4, 0.4, 0.4, 0.8),
            Interaction::Hovered => Color::srgba(0.3, 0.3, 0.3, 0.8),
            Interaction::None => Color::srgba(0.2, 0.2, 0.2, 0.8),
        }
        .into();
    }

    // handle backward button
    if let Ok(interaction) = backward_query.single() {
        if *interaction == Interaction::Pressed {
            time_state.is_paused = false;

            time_state.speed_mult = if time_state.speed_mult > 1.0 {
                time_state.speed_mult / 2.0
            } else if time_state.speed_mult == 1.0 {
                -1.0
            } else {
                (time_state.speed_mult * 2.0).clamp(-4096.0, -1.0)
            };
        }
    }

    // handle reset button
    if let Ok(interaction) = reset_query.single() {
        if *interaction == Interaction::Pressed {
            time_state.speed_mult = 1.0;
            time_state.is_paused = false;
        }
    }

    // handle forward button
    if let Ok(interaction) = forward_query.single() {
        if *interaction == Interaction::Pressed {
            time_state.is_paused = false;

            time_state.speed_mult = if time_state.speed_mult < -1.0 {
                (time_state.speed_mult / 2.0).clamp(1.0, -1.0)
            } else if time_state.speed_mult == -1.0 {
                1.0
            } else {
                (time_state.speed_mult * 2.0).clamp(1.0, 4096.0) // change max time speed here
            };
        }
    }
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
    time_state: ResMut<TimeState>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        // Create a fixed datetime (e.g., 2025-01-01 12:00:00 UTC)
        let fixed_datetime = chrono::DateTime::parse_from_rfc3339("2000-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        
        // format
        // text.0 = format!("Time: {} UTC", fixed_datetime.format("%Y-%m-%d %H:%M:%S"));
        text.0 = format!(
            "Time: {} UTC x{:.1}",
            fixed_datetime.format("%Y-%m-%d %H:%M:%S"),
            time_state.speed_mult,
        );
    }
}