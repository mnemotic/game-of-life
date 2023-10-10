//
// Copyright (c) 2023 Martin Green. All rights reserved.
//

use bevy::prelude::*;

use crate::{AdvanceSimTriggeredEvent, GameState, PauseSimTriggeredEvent, RewindSimTriggeredEvent};


#[derive(Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (
                    toggle_pause_simulation_on_key,
                    advance_simulation_on_key,
                    rewind_simulation_on_key,
                ),
                toggle_simulation_paused,
            )
                .chain(),
        );
    }
}


/// Pause / unpause the simulation on key press.
fn toggle_pause_simulation_on_key(
    keys: Res<'_, Input<KeyCode>>,
    state: Res<'_, State<GameState>>,
    mut ev_pause: EventWriter<'_, PauseSimTriggeredEvent>,
) {
    const PAUSE_KEYS: [KeyCode; 2] = [KeyCode::Space, KeyCode::P];

    for key in PAUSE_KEYS {
        if keys.just_pressed(key) {
            // Pause when running and unpause when paused.
            match state.get() {
                GameState::Running => ev_pause.send(PauseSimTriggeredEvent { pause: true }),
                GameState::Paused => ev_pause.send(PauseSimTriggeredEvent { pause: false }),
                _ => {}
            }
            break;
        }
    }
}


/// Advance the simulation by a single tick (generation) on key press.
fn advance_simulation_on_key(
    keys: Res<'_, Input<KeyCode>>,
    mut ev_advance: EventWriter<'_, AdvanceSimTriggeredEvent>,
    mut ev_pause: EventWriter<'_, PauseSimTriggeredEvent>,
) {
    const ADV_SIM_BINDINGS: [KeyCode; 1] = [KeyCode::BracketRight];

    for binding in ADV_SIM_BINDINGS {
        if keys.just_pressed(binding) {
            ev_advance.send(AdvanceSimTriggeredEvent);
            ev_pause.send(PauseSimTriggeredEvent { pause: true });
            break;
        }
    }
}


/// Rewind the simulation by a single tick (generation) on key press.
fn rewind_simulation_on_key(
    keys: Res<'_, Input<KeyCode>>,
    mut ev_rewind: EventWriter<'_, RewindSimTriggeredEvent>,
    mut ev_pause: EventWriter<'_, PauseSimTriggeredEvent>,
) {
    const RWD_SIM_BINDINGS: [KeyCode; 1] = [KeyCode::BracketLeft];

    for bindings in RWD_SIM_BINDINGS {
        if keys.just_pressed(bindings) {
            ev_rewind.send(RewindSimTriggeredEvent);
            ev_pause.send(PauseSimTriggeredEvent { pause: true });
            break;
        }
    }
}


/// Pause / unpause the simulation.
fn toggle_simulation_paused(
    state: Res<'_, State<GameState>>,
    mut next_state: ResMut<'_, NextState<GameState>>,
    mut ev_pause: EventReader<'_, '_, PauseSimTriggeredEvent>,
) {
    for ev in &mut ev_pause {
        match state.get() {
            GameState::Running if ev.pause => next_state.set(GameState::Paused),
            GameState::Paused if !ev.pause => next_state.set(GameState::Running),
            _ => {}
        }
    }
}
