//! time.rs
//! 
//! Global time state resource across the simulation defined here
//! pretty accurate

use bevy::prelude::*;
use chrono::Utc;

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeState::default())
           .add_systems(Update, update);
    }
}

/// Central time control state for entire simulation
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
            sim_time: chrono::DateTime::parse_from_rfc3339("2025-01-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        }
    }
}

impl TimeState {
    // pub fn reset(&mut self) {
    //     *self = TimeState::default();
    // }

    // pub fn set_speed(&mut self, speed: f64) {
    //     self.speed_mult = speed.clamp(-4096.0, 4096.0);
    // }

    // pub fn toggle_pause(&mut self) {
    //     self.is_paused = !self.is_paused;
    // }

    // decrease speed, or go negative
    pub fn step_backward(&mut self) {
        self.is_paused = false;

        self.speed_mult = if self.speed_mult > 1.0 {
            self.speed_mult / 2.0
        } else if self.speed_mult == 1.0 {
            -1.0
        } else {
            (self.speed_mult * 2.0).clamp(-4096.0, -1.0)
        };
    }

    // increase speed, or go positive
    pub fn step_forward(&mut self) {
        self.is_paused = false;

        self.speed_mult = if self.speed_mult < -1.0 {
            self.speed_mult / 2.0
        } else if self.speed_mult == -1.0 {
            1.0
        } else {
            (self.speed_mult * 2.0).clamp(1.0, 4096.0)
        };
    }

    pub fn reset_to_normal(&mut self) {
        self.speed_mult = 1.0;
        self.is_paused = false;
    }
}

fn update(
    mut time_state: ResMut<TimeState>,
    time: Res<Time>
) {
    if !time_state.is_paused {
        let real_delta_seconds = time.delta_secs_f64();
        let sim_delta_seconds = real_delta_seconds * time_state.speed_mult;
        
        // apply
        if let Some(new_time) = time_state.sim_time.checked_add_signed(
            chrono::Duration::milliseconds((sim_delta_seconds * 1000.0) as i64)
        ) {
            time_state.sim_time = new_time;
        }
    }
}