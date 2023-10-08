//
// Copyright (c) 2023 Martin Green. All rights reserved.
//

use bevy::prelude::*;

use crate::GameState;

#[derive(Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, pause);
    }
}


fn pause(
    keys: Res<'_, Input<KeyCode>>,
    state: Res<'_, State<GameState>>,
    mut next_state: ResMut<'_, NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::P) {
        match state.get() {
            GameState::Running => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Running),
            _ => {}
        }
    }
}
