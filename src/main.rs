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
use std::ops::{Neg, Not};

use ahash::AHashMap as HashMap;

use bevy::asset::LoadState;
// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::IRect;
use bevy::prelude::*;
use bevy_pixel_camera::{PixelCameraBundle, PixelCameraPlugin};

use input::ToggleCellTriggeredEvent;


mod input;

mod config {
    pub mod window {
        pub const WIDTH: u32 = 1280;
        pub const HEIGHT: u32 = 720;
    }

    pub mod cells {
        use bevy::math::Vec2;
        use bevy::prelude::Color;

        pub const DEAD_COLOR: Color = Color::GRAY;
        pub const LIVE_COLOR: Color = Color::GREEN;
        pub const SPRITE_SIZE: Vec2 = Vec2::splat(20.0);
        pub const OFFSET: Vec2 = Vec2::new(10.0, 10.0);
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

const SIMULATION_RATE: f32 = 1.0 / 2.0;


fn main() {
    use bevy::window::close_on_esc;

    // @REVIEW: See <https://github.com/bevy-cheatbook/bevy-cheatbook/issues/196>.
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let width = config::window::WIDTH;
    let height = config::window::HEIGHT;

    App::new()
        .init_resource::<GameAssets>()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Life::new(width / 20, height / 20))
        .insert_resource(SimulationUpdateTimer(Timer::from_seconds(
            SIMULATION_RATE,
            TimerMode::Repeating,
        )))
        .add_state::<GameState>()
        .add_event::<AdvanceSimTriggeredEvent>()
        .add_event::<RewindSimTriggeredEvent>()
        .add_event::<PauseSimTriggeredEvent>()
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
        .add_plugins((
            PixelCameraPlugin,
            // LogDiagnosticsPlugin::default(),
            // FrameTimeDiagnosticsPlugin,
        ))
        .add_plugins(input::InputPlugin)
        .add_systems(Startup, (init_camera, load_assets))
        .add_systems(
            Startup,
            |mut next_state: ResMut<'_, NextState<GameState>>| next_state.set(GameState::Startup),
        )
        .add_systems(
            Update,
            check_asset_loading.run_if(in_state(GameState::Startup)),
        )
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
                .run_if(on_event::<AdvanceSimTriggeredEvent>()),
        )
        .add_systems(
            Update,
            (rewind_simulation, update_presentation)
                .chain()
                .run_if(on_event::<RewindSimTriggeredEvent>()),
        )
        .add_systems(OnEnter(GameState::Paused), reset_simulation_update_timer)
        .add_systems(
            Update,
            (toggle_cell, update_presentation)
                .chain()
                .run_if(on_event::<ToggleCellTriggeredEvent>()),
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


#[derive(Default, Resource, Deref, DerefMut)]
struct GameAssets(pub Vec<HandleUntyped>);


#[derive(Resource, Deref, DerefMut)]
struct GlyphAtlas(pub Handle<TextureAtlas>);


#[derive(Resource, Deref, DerefMut)]
struct SimulationUpdateTimer(Timer);


#[derive(Event)]
struct AdvanceSimTriggeredEvent;

#[derive(Event)]
struct RewindSimTriggeredEvent;

#[derive(Event)]
struct PauseSimTriggeredEvent {
    pause: bool,
}


#[derive(Component, Deref)]
struct Position(pub IVec2);


#[derive(Component)]
struct MainCamera;


#[derive(Resource)]
pub struct Life {
    pub bounds: IRect,
    pub history: VecDeque<HashMap<IVec2, bool>>,
    pub cells: HashMap<IVec2, bool>,
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
        let min = max.neg();

        let capacity = (width * height).try_into().unwrap();
        Self {
            bounds: IRect::from_corners(min, max),
            cells: HashMap::with_capacity(capacity),
            history: VecDeque::with_capacity(Self::MAX_HISTORY_SIZE),
        }
    }
}


fn init_camera(mut commands: Commands<'_, '_>) {
    commands.spawn((
        PixelCameraBundle::from_resolution(
            config::window::WIDTH as i32,
            config::window::HEIGHT as i32,
            true,
        ),
        MainCamera,
    ));
}


fn load_assets(
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
    mut assets: ResMut<'_, GameAssets>,
    mut texture_atlases: ResMut<'_, Assets<TextureAtlas>>,
) {
    const FONTSHEET_PATH: &str = "cp437_10x10.png";

    let fontsheet = asset_server.load(FONTSHEET_PATH);
    assets.push(fontsheet.clone_untyped());

    let atlas = TextureAtlas::from_grid(fontsheet, Vec2::splat(10.0), 16, 16, None, None);
    commands.insert_resource(GlyphAtlas(texture_atlases.add(atlas)));
}


fn check_asset_loading(
    asset_server: Res<'_, AssetServer>,
    assets: Res<'_, GameAssets>,
    mut next_state: ResMut<'_, NextState<GameState>>,
) {
    match asset_server.get_group_load_state(assets.iter().map(HandleUntyped::id)) {
        LoadState::Loading => {
            info!("Loading assets...");
        }
        LoadState::Loaded => {
            info!("Assets loaded.");
            next_state.set(GameState::Running);
        }
        LoadState::Failed => panic!("Failed to load assets."),
        _ => {}
    }
}


fn init_simulation(mut world: ResMut<'_, Life>) {
    world.cells.insert(IVec2::new(0, 0), true);

    world.cells.insert(IVec2::new(1, 0), true);
    world.cells.insert(IVec2::new(-1, 0), true);

    world.cells.insert(IVec2::new(0, 1), true);
    world.cells.insert(IVec2::new(0, -1), true);
}


fn init_presentation(
    mut commands: Commands<'_, '_>,
    world: Res<'_, Life>,
    glyphs: Res<'_, GlyphAtlas>,
) {
    use config::cells::{DEAD_COLOR, LIVE_COLOR, OFFSET, SPRITE_SIZE};

    for y in world.bounds.min.y..world.bounds.max.y {
        for x in world.bounds.min.x..world.bounds.max.x {
            let sprite = if world.cells.contains_key(&IVec2::new(x, y)) {
                TextureAtlasSprite {
                    index: 254,
                    color: LIVE_COLOR,
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
                (Vec2::new(x as f32, y as f32) * SPRITE_SIZE + OFFSET).extend(0.0),
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
    mut ev_trigger: EventWriter<'_, AdvanceSimTriggeredEvent>,
) {
    if timer.tick(time.delta()).finished() {
        ev_trigger.send(AdvanceSimTriggeredEvent);
    }
}

/// Reset simulation update timer.
///
/// Executed on entering the `GameState::Paused` state.
fn reset_simulation_update_timer(mut timer: ResMut<'_, SimulationUpdateTimer>) {
    timer.reset();
}


/// Advance the simulation a single tick (generation).
fn advance_simulation(life: ResMut<'_, Life>) {
    /// Wrap:
    /// ```
    /// max_x -> min_x
    /// min_x - 1 -> max_x - 1
    /// max_y -> min_y
    /// min_y - 1 -> max_y - 1
    /// ```
    /// Max value is wrapped to minimum because iteration range `min_x..max_x` doesn't include `max_x`.
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

    // Re-borrow.
    let life = life.into_inner();

    let mut next_gen: HashMap<IVec2, bool> = HashMap::with_capacity(
        (life.bounds.width() * life.bounds.height())
            .try_into()
            .unwrap(),
    );

    let min_x = life.bounds.min.x;
    let max_x = life.bounds.max.x;
    let min_y = life.bounds.min.y;
    let max_y = life.bounds.max.y;

    for y in min_y..max_y {
        for x in min_x..max_x {
            let pt = IVec2::new(x, y);

            // We count the number of alive cells, including the inner cell, in the neighborhood
            // of each cell. If the count is 3, then the state of the inner cell in the next generation
            // is alive; if the count is 4, then the state of the inner cell remains the same; if the
            // count is anything else, then the state of the inner cell is dead.

            let mut count = 0;
            if life.cells.get(&pt).copied().unwrap_or(false) {
                count += 1;
            }

            for offset in NEIGHBOR_OFFSETS {
                let pt = wrap(&life.bounds, pt + offset);
                if life.cells.get(&pt).copied().unwrap_or(false) {
                    count += 1;
                }
            }

            match count {
                3 => {
                    // Alive.
                    next_gen.insert(pt, true);
                }
                4 => {
                    // Same. If cell was empty before, it was dead (`false`).
                    next_gen.insert(pt, life.cells.get(&pt).copied().unwrap_or(false));
                }
                _ => {
                    // Dead.
                    next_gen.insert(pt, false);
                }
            }
        }
    }

    if life.history.len() >= Life::MAX_HISTORY_SIZE {
        life.history.pop_back();
    }
    life.history
        .push_front(std::mem::replace(&mut life.cells, next_gen));
}


/// Rewind the simulation a single tick (generation).
fn rewind_simulation(mut life: ResMut<'_, Life>) {
    let Some(prev_gen) = life.history.pop_front() else {
        info!("History is empty.");
        return;
    };
    life.cells = prev_gen;
}


/// Update the presentation.
fn update_presentation(
    life: Res<'_, Life>,
    mut q_sprites: Query<'_, '_, (&Position, &mut TextureAtlasSprite)>,
) {
    use config::cells::{DEAD_COLOR, LIVE_COLOR};

    for (position, mut sprite) in &mut q_sprites {
        if life.cells.get(&position.0).copied().unwrap_or(false) {
            sprite.index = 254;
            sprite.color = LIVE_COLOR;
        } else {
            sprite.index = 255;
            sprite.color = DEAD_COLOR;
        }
    }
}


fn toggle_cell(
    mut life: ResMut<'_, Life>,
    mut ev_toggle: EventReader<'_, '_, ToggleCellTriggeredEvent>,
) {
    for xy in &mut ev_toggle {
        if let Some(cell) = life.cells.get_mut(xy) {
            *cell = cell.not();
        } else {
            life.cells.insert(**xy, true);
        }
    }
}
