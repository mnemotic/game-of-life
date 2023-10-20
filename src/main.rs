//
// Copyright (c) 2023 Martin Green. All rights reserved.
//

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]
#![deny(clippy::disallowed_types, clippy::missing_enforced_import_renames)]
#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]

use std::collections::VecDeque;

use ahash::AHashMap as HashMap;
use bevy::math::IRect;
use bevy::prelude::*;

use crate::assets::GlyphAtlas;
use crate::input::InputAction;


mod assets;
mod camera;
mod color_gradient;
mod config;
mod input;
mod ui;


const NEIGHBOR_OFFSETS: [IVec2; 8] = [
    IVec2 { x: 0, y: 1 },
    IVec2 { x: 1, y: 1 },
    IVec2 { x: 1, y: 0 },
    IVec2 { x: 1, y: -1 },
    IVec2 { x: 0, y: -1 },
    IVec2 { x: -1, y: -1 },
    IVec2 { x: -1, y: 0 },
    IVec2 { x: -1, y: 1 },
];


#[derive(Resource)]
pub struct SimulationConfig {
    ticks_per_second: i32,
}


#[derive(Default, Resource)]
struct WindowFocus {
    focused: bool,
}

#[derive(Event)]
struct WindowFocused {
    focused: bool,
}


fn main() {
    use bevy::window::close_on_esc;

    // @REVIEW: See <https://github.com/bevy-cheatbook/bevy-cheatbook/issues/196>.
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let width = config::window::WIDTH;
    let height = config::window::HEIGHT;

    let tps = config::sim::DEFAULT_TICKS_PER_SECOND;

    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Life::new(width / 20, height / 20))
        .insert_resource(SimulationConfig {
            ticks_per_second: tps,
        })
        .insert_resource(SimulationUpdateTimer(Timer::from_seconds(
            1.0 / tps as f32,
            TimerMode::Repeating,
        )))
        .add_state::<GameState>()
        .add_event::<WindowFocused>()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (width as f32, height as f32).into(),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        title: "Conway's Game of Life".to_owned(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(input::InputPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(assets::AssetPlugin)
        .add_systems(
            Startup,
            |mut next_state: ResMut<'_, NextState<GameState>>| next_state.set(GameState::Startup),
        )
        .add_systems(PreUpdate, track_window_focus)
        .add_systems(
            OnEnter(GameState::Running),
            (init_simulation, init_presentation)
                .chain()
                .run_if(run_once()),
        )
        .add_systems(Update, close_on_esc)
        .add_systems(
            Update,
            tick_simulation_update_timer.run_if(in_state(GameState::Running)),
        )
        .add_systems(
            Update,
            (advance_simulation, update_presentation)
                .chain()
                .run_if(on_event::<InputAction>()),
        )
        .add_systems(
            Update,
            (rewind_simulation, update_presentation)
                .chain()
                .run_if(on_event::<InputAction>()),
        )
        .add_systems(OnEnter(GameState::Paused), reset_simulation_update_timer)
        .add_systems(
            Update,
            (toggle_cell, update_presentation)
                .chain()
                .run_if(on_event::<InputAction>()),
        )
        .run();
}


#[derive(States, Clone, Debug, Default, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    None,
    Startup,
    Running,
    Paused,
}


#[derive(Resource, Deref, DerefMut)]
struct SimulationUpdateTimer(Timer);


#[derive(Component, Deref)]
struct Position(pub IVec2);


#[derive(Copy, Clone)]
pub struct Cell {
    alive: bool,
    age: usize,
}

impl Cell {
    fn new(alive: bool, age: usize) -> Self {
        Self { alive, age }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new(true, 0)
    }
}


#[derive(Resource)]
pub struct Life {
    pub bounds: IRect,
    pub history: VecDeque<HashMap<IVec2, Cell>>,
    pub cells: HashMap<IVec2, Cell>,
    pub generation: usize,
}

impl Life {
    pub const MAX_HISTORY_SIZE: usize = 32;

    pub fn new(width: u32, height: u32) -> Self {
        let half_width = (width / 2) as i32;
        let half_height = (height / 2) as i32;

        let max = IVec2 {
            x: half_width,
            y: half_height,
        };
        let min = -max;

        Self {
            bounds: IRect::from_corners(min, max),
            cells: HashMap::new(),
            history: VecDeque::with_capacity(Self::MAX_HISTORY_SIZE),
            generation: 0,
        }
    }
}


fn init_simulation(mut world: ResMut<'_, Life>) {
    // "Butterfly" pattern.
    world.cells.insert(IVec2::new(0, 3), Cell::default());
    world.cells.insert(IVec2::new(0, 2), Cell::default());
    world.cells.insert(IVec2::new(0, 1), Cell::default());
    world.cells.insert(IVec2::new(0, 0), Cell::default());
    world.cells.insert(IVec2::new(0, -1), Cell::default());
    world.cells.insert(IVec2::new(0, -2), Cell::default());
    world.cells.insert(IVec2::new(0, -3), Cell::default());

    world.cells.insert(IVec2::new(1, 0), Cell::default());
    world.cells.insert(IVec2::new(-1, 0), Cell::default());
}


fn init_presentation(
    mut commands: Commands<'_, '_>,
    world: Res<'_, Life>,
    glyphs: Res<'_, GlyphAtlas>,
) {
    use config::cells::{get_age_color, DEAD_COLOR, SPRITE_SIZE, SPRITE_WORLD_OFFSET};

    for y in world.bounds.min.y..world.bounds.max.y {
        for x in world.bounds.min.x..world.bounds.max.x {
            let sprite = if world.cells.contains_key(&IVec2::new(x, y)) {
                TextureAtlasSprite {
                    index: 254,
                    color: get_age_color(0),
                    custom_size: Some(SPRITE_SIZE),
                    ..default()
                }
            } else {
                TextureAtlasSprite {
                    index: 255,
                    color: DEAD_COLOR,
                    custom_size: Some(SPRITE_SIZE),
                    ..default()
                }
            };

            let transform = Transform::from_translation(
                (Vec2::new(x as f32, y as f32) * SPRITE_SIZE + SPRITE_WORLD_OFFSET).extend(0.0),
            );
            commands.spawn((
                SpriteSheetBundle {
                    sprite,
                    texture_atlas: glyphs.clone(),
                    transform,
                    ..default()
                },
                Position(IVec2::new(x, y)),
            ));
        }
    }
}


fn tick_simulation_update_timer(
    mut timer: ResMut<'_, SimulationUpdateTimer>,
    time: Res<'_, Time>,
    mut actions: EventWriter<'_, InputAction>,
) {
    if timer.tick(time.delta()).finished() {
        // @REVIEW: **Technically** not an *input* action.
        actions.send(InputAction::AdvanceSimulation);
    }
}

/// Reset simulation update timer.
///
/// Executed on entering the `GameState::Paused` state.
fn reset_simulation_update_timer(mut timer: ResMut<'_, SimulationUpdateTimer>) {
    timer.reset();
}


/// Advance the simulation a single tick (generation).
fn advance_simulation(life: ResMut<'_, Life>, mut actions: EventReader<'_, '_, InputAction>) {
    /// Wrap:
    /// ```
    /// max_x -> min_x
    /// min_x - 1 -> max_x - 1
    /// max_y -> min_y
    /// min_y - 1 -> max_y - 1
    /// ```
    /// Max value is wrapped to minimum because iteration range `min_x..max_x` doesn't include
    /// `max_x`.
    fn wrap(bounds: &IRect, xy: IVec2) -> IVec2 {
        let mut x = xy.x;
        let mut y = xy.y;

        let min_x = bounds.min.x;
        let max_x = bounds.max.x;

        // Wrap horizontally.
        if x < min_x {
            x = max_x - (x - min_x).abs();
        } else if x >= max_x {
            x = min_x + (x - max_x).abs();
        }

        let min_y = bounds.min.y;
        let max_y = bounds.max.y;

        // Wrap vertically.
        if y < min_y {
            y = max_y - (y - min_y).abs();
        } else if y >= max_y {
            y = min_y + (y - max_y).abs();
        }

        IVec2 { x, y }
    }

    let life = life.into_inner();

    for action in &mut actions {
        if let InputAction::AdvanceSimulation = action {
            // Re-borrow.
            debug!("Hash map capacity is {}", life.cells.capacity());

            let mut next_gen: HashMap<IVec2, Cell> = HashMap::with_capacity(life.cells.capacity());

            let min_x = life.bounds.min.x;
            let max_x = life.bounds.max.x;
            let min_y = life.bounds.min.y;
            let max_y = life.bounds.max.y;

            for y in min_y..max_y {
                for x in min_x..max_x {
                    let pt = IVec2::new(x, y);

                    // We count the number of alive cells, including the inner cell, in the
                    // neighborhood of each cell. If the count is 3, then the  state of the inner
                    // cell in the next generation is alive; if the count is 4, then the state of
                    // the inner cell remains the same; if the count is anything else, then the
                    // state of the inner cell is dead.

                    // Extend `NEIGHBOR_OFFSET` with a invariant offset for the inner cell.
                    let offsets = NEIGHBOR_OFFSETS
                        .iter()
                        .chain([IVec2 { x: 0, y: 0 }].iter())
                        .collect::<Vec<_>>();

                    let mut count = 0;
                    for offset in offsets {
                        let pt = wrap(&life.bounds, pt + *offset);
                        if let Some(cell) = life.cells.get(&pt) {
                            if cell.alive {
                                count += 1;
                            }
                        }
                    }

                    match count {
                        3 => {
                            // Cell at `pt` either stays alive or spawns new life.
                            if let Some(cell) = life.cells.get(&pt) {
                                next_gen.insert(pt, Cell::new(cell.alive, cell.age + 1));
                            } else {
                                next_gen.insert(pt, Cell::default());
                            }
                        }
                        4 => {
                            // Existing cells stay as they were.
                            if let Some(cell) = life.cells.get(&pt) {
                                next_gen.insert(pt, Cell::new(cell.alive, cell.age + 1));
                            }
                        }
                        _ => {} // Cell at `pt` dies.
                    }
                }
            }

            if life.history.len() >= Life::MAX_HISTORY_SIZE {
                life.history.pop_back();
            }
            life.history
                .push_front(std::mem::replace(&mut life.cells, next_gen));
            life.generation += 1;
        }
    }
}


/// Rewind the simulation a single tick (generation).
fn rewind_simulation(mut life: ResMut<'_, Life>, mut actions: EventReader<'_, '_, InputAction>) {
    for action in &mut actions {
        if let InputAction::RewindSimulation = action {
            let Some(prev_gen) = life.history.pop_front() else {
                info!("History is empty");
                return;
            };
            life.cells = prev_gen;
            life.generation -= 1;
        }
    }
}


/// Update the presentation.
fn update_presentation(
    life: Res<'_, Life>,
    mut q_sprites: Query<'_, '_, (&Position, &mut TextureAtlasSprite)>,
) {
    use config::cells::{get_age_color, DEAD_COLOR};

    for (position, mut sprite) in &mut q_sprites {
        if let Some(cell) = life.cells.get(position) {
            sprite.index = 254;
            sprite.color = get_age_color(cell.age);
        } else {
            sprite.index = 255;
            sprite.color = DEAD_COLOR;
        }
    }
}


fn toggle_cell(mut life: ResMut<'_, Life>, mut actions: EventReader<'_, '_, InputAction>) {
    for action in &mut actions {
        if let InputAction::ToggleCell(xy) = action {
            if life.cells.contains_key(xy) {
                life.cells.remove(xy);
            } else {
                life.cells.insert(*xy, Cell::default());
            }
        }
    }
}


fn track_window_focus(
    mut focus: Local<'_, WindowFocus>,
    mut ev_focused_bevy: EventReader<'_, '_, bevy::window::WindowFocused>,
    mut ev_focused: EventWriter<'_, WindowFocused>,
) {
    let focused = focus.focused;

    // Aggregate focus events.
    for event in &mut ev_focused_bevy {
        debug!("{event:?}");
        focus.focused = event.focused;
    }

    if focus.focused != focused {
        ev_focused.send(WindowFocused {
            focused: focus.focused,
        });
    }
}
