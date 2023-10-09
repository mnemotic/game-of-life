//
// Copyright (c) 2023 Martin Green. All rights reserved.
//

use bevy::prelude::*;

use crate::{AdvanceSimTriggeredEvent, GameState, RewindSimTriggeredEvent};


#[derive(Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                toggle_simulation_paused,
                trigger_simulation_advance.run_if(in_state(GameState::Paused)),
                trigger_simulation_rewind.run_if(in_state(GameState::Paused)),
            ),
        );
    }
}


/// Pause / unpause the simulation.
fn toggle_simulation_paused(
    keys: Res<'_, Input<KeyCode>>,
    state: Res<'_, State<GameState>>,
    mut next_state: ResMut<'_, NextState<GameState>>,
) {
    const PAUSE_KEYS: [KeyCode; 2] = [KeyCode::Space, KeyCode::P];

    for key in PAUSE_KEYS {
        if keys.just_pressed(key) {
            match state.get() {
                GameState::Running => next_state.set(GameState::Paused),
                GameState::Paused => next_state.set(GameState::Running),
                _ => {}
            }
            break;
        }
    }
}


/// Trigger advance of the simulation by a single tick (generation).
fn trigger_simulation_advance(
    keys: Res<'_, Input<KeyCode>>,
    mut ev_trigger: EventWriter<'_, AdvanceSimTriggeredEvent>,
) {
    const ADV_SIM_BINDINGS: [KeyCode; 1] = [KeyCode::BracketRight];

    for binding in ADV_SIM_BINDINGS {
        if keys.just_pressed(binding) {
            ev_trigger.send(AdvanceSimTriggeredEvent);
            break;
        }
    }
}


/// Trigger rewind of the simulation by a single tick (generation).
fn trigger_simulation_rewind(
    keys: Res<'_, Input<KeyCode>>,
    mut ev_trigger: EventWriter<'_, RewindSimTriggeredEvent>,
) {
    const RWD_SIM_BINDINGS: [KeyCode; 1] = [KeyCode::BracketLeft];

    for bindings in RWD_SIM_BINDINGS {
        if keys.just_pressed(bindings) {
            ev_trigger.send(RewindSimTriggeredEvent);
            break;
        }
    }
}
