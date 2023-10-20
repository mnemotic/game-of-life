//
// Copyright (c) 2023 Martin Green. All rights reserved.
//

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::camera::MainCamera;
use crate::config::cells::{SPRITE_SIZE, SPRITE_WORLD_OFFSET};
use crate::{GameState, SimulationConfig, SimulationUpdateTimer, WindowFocused};


#[derive(Default, Resource, Deref, DerefMut)]
struct CursorWorldPosition(Vec2);


#[derive(Event)]
pub enum InputAction {
    ToggleCell(IVec2),
    PauseSimulation,
    UnpauseSimulation,
    AdvanceSimulation,
    RewindSimulation,
}


#[derive(Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPosition>()
            .add_event::<InputAction>()
            .add_systems(
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
            )
            .add_systems(
                Update,
                (get_cursor_world_position, toggle_cell_on_lmb).chain(),
            )
            .add_systems(Update, change_simulation_rate);
    }
}


/// Pause / unpause the simulation on key press.
fn toggle_pause_simulation_on_key(
    keys: Res<'_, Input<KeyCode>>,
    state: Res<'_, State<GameState>>,
    mut actions: EventWriter<'_, InputAction>,
) {
    const PAUSE_KEYS: [KeyCode; 2] = [KeyCode::Space, KeyCode::P];

    for key in PAUSE_KEYS {
        if keys.just_pressed(key) {
            // Pause when running and unpause when paused.
            match state.get() {
                GameState::Running => actions.send(InputAction::PauseSimulation),
                GameState::Paused => actions.send(InputAction::UnpauseSimulation),
                _ => {}
            }
            break;
        }
    }
}


/// Advance the simulation by a single tick (generation) on key press.
fn advance_simulation_on_key(
    keys: Res<'_, Input<KeyCode>>,
    mut actions: EventWriter<'_, InputAction>,
) {
    const ADV_SIM_BINDINGS: [KeyCode; 1] = [KeyCode::BracketRight];

    for binding in ADV_SIM_BINDINGS {
        if keys.just_pressed(binding) {
            actions.send(InputAction::PauseSimulation);
            actions.send(InputAction::AdvanceSimulation);
            break;
        }
    }
}


/// Rewind the simulation by a single tick (generation) on key press.
fn rewind_simulation_on_key(
    keys: Res<'_, Input<KeyCode>>,
    mut actions: EventWriter<'_, InputAction>,
) {
    const RWD_SIM_BINDINGS: [KeyCode; 1] = [KeyCode::BracketLeft];

    for bindings in RWD_SIM_BINDINGS {
        if keys.just_pressed(bindings) {
            actions.send(InputAction::PauseSimulation);
            actions.send(InputAction::RewindSimulation);
            break;
        }
    }
}


/// Pause / unpause the simulation.
fn toggle_simulation_paused(
    state: Res<'_, State<GameState>>,
    mut next_state: ResMut<'_, NextState<GameState>>,
    mut actions: EventReader<'_, '_, InputAction>,
) {
    for action in &mut actions {
        match state.get() {
            GameState::Running => {
                if let InputAction::PauseSimulation = action {
                    next_state.set(GameState::Paused);
                }
            }
            GameState::Paused => {
                if let InputAction::UnpauseSimulation = action {
                    next_state.set(GameState::Running);
                }
            }
            _ => {}
        }
    }
}


// @CREDIT: <https://bevy-cheatbook.github.io/cookbook/cursor2world.html>
fn get_cursor_world_position(
    q_window: Query<'_, '_, &Window, With<PrimaryWindow>>,
    q_camera: Query<'_, '_, (&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_position: ResMut<'_, CursorWorldPosition>,
) {
    let (camera, transform) = q_camera.single();
    let window = q_window.single();

    // @NOTE: Sprites are offset by `OFFSET`. We need to offset the cursor position by
    // `OFFSET.neg()`.
    if let Some(position) = window.cursor_position().and_then(|cursor| {
        camera
            .viewport_to_world_2d(transform, cursor)
            .map(|v| v + -SPRITE_WORLD_OFFSET)
    }) {
        *mouse_position = CursorWorldPosition(position);
    }
}

fn toggle_cell_on_lmb(
    buttons: Res<'_, Input<MouseButton>>,
    mouse_position: Res<'_, CursorWorldPosition>,
    mut ev_focused: EventReader<'_, '_, WindowFocused>,
    mut actions: EventWriter<'_, InputAction>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        // Ignore input that caused the window to receive focus.
        for event in &mut ev_focused {
            if event.focused {
                info!("Ignoring input due to receiving focus");
                return;
            }
        }

        #[allow(clippy::cast_possible_truncation)]
        let xy = IVec2::new(
            (mouse_position.x / SPRITE_SIZE.x).round() as i32,
            (mouse_position.y / SPRITE_SIZE.y).round() as i32,
        );

        debug!("Clicked {xy:?}");
        actions.send(InputAction::ToggleCell(xy));
    }
}


fn change_simulation_rate(
    keys: Res<'_, Input<KeyCode>>,
    mut config: ResMut<'_, SimulationConfig>,
    mut timer: ResMut<'_, SimulationUpdateTimer>,
) {
    let mut tps = config.ticks_per_second;
    if keys.just_pressed(KeyCode::Minus) {
        tps -= 1;
    }
    if keys.just_pressed(KeyCode::Equals) {
        tps += 1;
    }
    tps = tps.clamp(1, 64);

    if tps != config.ticks_per_second {
        debug!("TPS changed: {} -> {}", config.ticks_per_second, tps);

        // @REVIEW: This resets the timer.
        config.ticks_per_second = tps;
        *timer = SimulationUpdateTimer(Timer::from_seconds(1.0 / tps as f32, TimerMode::Repeating));
    }
}
