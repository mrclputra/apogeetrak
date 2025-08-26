//! ui.rs
//! 
//! Simplistic UI implementation
//! just has satellite count, datetime, and buttons for time control

use bevy::prelude::*;

use crate::systems::satellites::Satellite;
use crate::systems::time::TimeState;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
           .add_systems(Update, (
                update_satellite_count, 
                update_datetime, 
                handle_time_control,
                handle_exit,
            ));
    }
}

// UI component to display satellite count
#[derive(Component)]
pub struct SatelliteCounter;

// UI component to display current datetime
#[derive(Component)]
pub struct DateTimeDisplay;

// time control button components
#[derive(Component)]
pub struct ResetButton;

#[derive(Component)]
pub struct BackwardButton;

#[derive(Component)]
pub struct ForwardButton;

// system exit
fn handle_exit(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
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

/// Handle time control button interactions
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
    if let Ok(interaction) = backward_query.single()
        && *interaction == Interaction::Pressed
    {
        time_state.step_backward();
    }

    // handle reset button
    if let Ok(interaction) = reset_query.single()
        && *interaction == Interaction::Pressed
    {
        time_state.reset_to_normal();
    }

    // handle forward button
    if let Ok(interaction) = forward_query.single()
        && *interaction == Interaction::Pressed
    {
        time_state.step_forward();
    }
}

/// update the satellite count display
fn update_satellite_count(
    satellite_query: Query<&Satellite>,
    mut text_query: Query<&mut Text, With<SatelliteCounter>>,
) {
    let count = satellite_query.iter().count();
    
    if let Ok(mut text) = text_query.single_mut() {
        text.0 = format!("Satellites: {}", count);
    }
}

/// update the datetime display with current simulation time
fn update_datetime(
    mut text_query: Query<&mut Text, With<DateTimeDisplay>>,
    time_state: Res<TimeState>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        text.0 = format!(
            "Time: {} UTC x{:.1}",
            time_state.sim_time.format("%Y-%m-%d %H:%M:%S"),
            time_state.speed_mult,
        );
    }
}