//
// Copyright (c) 2023 Martin Green. All rights reserved.
//

use std::collections::VecDeque;

use ahash::AHashMap as HashMap;
use bevy::math::IRect;
use bevy::prelude::*;

use crate::input::InputAction;
use crate::{config, GameState};


#[derive(Clone, PartialEq, Eq, Hash, Debug, SystemSet)]
pub struct GameLogicSet;


pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let tps = config::sim::DEFAULT_TICKS_PER_SECOND;

        #[allow(clippy::cast_precision_loss)]
        app.insert_resource(SimulationConfig {
            ticks_per_second: tps,
        })
        .insert_resource(SimulationUpdateTimer(Timer::from_seconds(
            1.0 / tps as f32,
            TimerMode::Repeating,
        )))
        .configure_sets(OnEnter(GameState::Running), GameLogicSet)
        .configure_sets(Update, GameLogicSet.run_if(on_event::<InputAction>()))
        .add_systems(
            OnEnter(GameState::Running),
            setup_simulation.in_set(GameLogicSet).run_if(run_once()),
        )
        .add_systems(
            Update,
            (advance_simulation, rewind_simulation, toggle_cell).in_set(GameLogicSet),
        )
        .add_systems(
            Update,
            tick_simulation_update_timer.run_if(in_state(GameState::Running)),
        )
        .add_systems(OnEnter(GameState::Paused), reset_simulation_update_timer);
    }
}


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
    pub ticks_per_second: i32,
}


#[derive(Resource, Deref, DerefMut)]
pub struct SimulationUpdateTimer(pub Timer);


#[derive(Copy, Clone)]
pub struct Cell {
    pub alive: bool,
    pub age: usize,
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

    #[allow(clippy::cast_possible_wrap)]
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


fn setup_simulation(mut life: ResMut<'_, Life>) {
    // "Butterfly" pattern.
    life.cells.insert(IVec2::new(0, 3), Cell::default());
    life.cells.insert(IVec2::new(0, 2), Cell::default());
    life.cells.insert(IVec2::new(0, 1), Cell::default());
    life.cells.insert(IVec2::new(0, 0), Cell::default());
    life.cells.insert(IVec2::new(0, -1), Cell::default());
    life.cells.insert(IVec2::new(0, -2), Cell::default());
    life.cells.insert(IVec2::new(0, -3), Cell::default());

    life.cells.insert(IVec2::new(1, 0), Cell::default());
    life.cells.insert(IVec2::new(-1, 0), Cell::default());
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
pub fn reset_simulation_update_timer(mut timer: ResMut<'_, SimulationUpdateTimer>) {
    timer.reset();
}


/// Advance the simulation a single tick (generation).
pub fn advance_simulation(life: ResMut<'_, Life>, mut actions: EventReader<'_, '_, InputAction>) {
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

    for action in actions.read() {
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
                    // neighborhood of each cell.

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

                    // If the count is 3, then the  state of the inner cell in the next generation
                    // is alive; if the count is 4, then the state of the inner cell remains the
                    // same; if the count is anything else, then the state of the inner cell is
                    // dead.
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
pub fn rewind_simulation(
    mut life: ResMut<'_, Life>,
    mut actions: EventReader<'_, '_, InputAction>,
) {
    for action in actions.read() {
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


fn toggle_cell(mut life: ResMut<'_, Life>, mut actions: EventReader<'_, '_, InputAction>) {
    for action in actions.read() {
        if let InputAction::ToggleCell(xy) = action {
            if life.cells.contains_key(xy) {
                life.cells.remove(xy);
            } else {
                life.cells.insert(*xy, Cell::default());
            }
        }
    }
}
