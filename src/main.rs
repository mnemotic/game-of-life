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

use std::ops::Neg;

use ahash::AHashMap as HashMap;

use bevy::asset::LoadState;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::IRect;
use bevy::prelude::*;
use bevy_pixel_camera::{PixelCameraBundle, PixelCameraPlugin};

mod input;

mod config {
    pub mod window {
        pub const WIDTH: u32 = 1920;
        pub const HEIGHT: u32 = 1080;
    }

    pub mod cells {
        use bevy::math::Vec2;
        use bevy::prelude::Color;

        pub const DEAD_COLOR: Color = Color::GRAY;
        pub const LIVE_COLOR: Color = Color::GREEN;
        pub const SPRITE_SIZE: Vec2 = Vec2::splat(20.0);
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


fn main() {
    use bevy::window::close_on_esc;

    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let width = config::window::WIDTH;
    let height = config::window::HEIGHT;

    App::new()
        .init_resource::<GameAssets>()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Life::new(width / 20, height / 20))
        .insert_resource(LifeUpdateTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )))
        .add_state::<GameState>()
        .add_event::<LifeUpdateTickEvent>()
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
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_plugins(input::InputPlugin)
        .add_systems(Startup, (setup_camera, load_assets))
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
            (seed_life, spawn_cell_sprites).chain().run_if(run_once()),
        )
        .add_systems(Update, close_on_esc)
        .add_systems(
            Update,
            tick_life_update_timer.run_if(in_state(GameState::Running)),
        )
        .add_systems(
            Update,
            (update_life, update_cell_sprites)
                .chain()
                .run_if(on_event::<LifeUpdateTickEvent>()),
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
struct LifeUpdateTimer(Timer);


#[derive(Event)]
struct LifeUpdateTickEvent;


#[derive(Component, Deref)]
pub struct Position(pub IVec2);


#[derive(Resource)]
pub struct Life {
    pub bounds: IRect,
    pub cells: HashMap<IVec2, bool>,
}

impl Life {
    pub fn new(width: u32, height: u32) -> Self {
        let half_width = (width / 2) as i32;
        let half_height = (height / 2) as i32;

        let max = IVec2 {
            x: half_width,
            y: half_height,
        };
        let min = max.neg();

        Self {
            bounds: IRect::from_corners(min, max),
            cells: HashMap::with_capacity((width * height).try_into().unwrap()),
        }
    }
}


fn setup_camera(mut commands: Commands<'_, '_>) {
    commands.spawn(PixelCameraBundle::from_resolution(
        config::window::WIDTH as i32,
        config::window::HEIGHT as i32,
        true,
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


fn seed_life(mut world: ResMut<'_, Life>) {
    world.cells.insert(IVec2::new(0, 0), true);

    world.cells.insert(IVec2::new(1, 0), true);
    world.cells.insert(IVec2::new(-1, 0), true);

    world.cells.insert(IVec2::new(0, 1), true);
    world.cells.insert(IVec2::new(0, -1), true);
}


fn spawn_cell_sprites(
    mut commands: Commands<'_, '_>,
    world: Res<'_, Life>,
    glyphs: Res<'_, GlyphAtlas>,
) {
    use config::cells::{DEAD_COLOR, LIVE_COLOR, SPRITE_SIZE};

    let offset = Vec2::new(10.0, 10.0);

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
                (Vec2::new(x as f32, y as f32) * SPRITE_SIZE + offset).extend(0.0),
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


fn tick_life_update_timer(
    mut timer: ResMut<'_, LifeUpdateTimer>,
    time: Res<'_, Time>,
    mut ev_update: EventWriter<'_, LifeUpdateTickEvent>,
) {
    if timer.tick(time.delta()).finished() {
        ev_update.send(LifeUpdateTickEvent);
    }
}


fn update_life(mut life: ResMut<'_, Life>) {
    fn wrap(bounds: &IRect, xy: IVec2) -> IVec2 {
        let mut x = xy.x;
        let mut y = xy.y;

        let min_x = bounds.min.x;
        let max_x = bounds.max.x;

        // Wrap horizontally.
        if x < min_x {
            x = max_x - (x - min_x);
        } else if x > max_x {
            x = min_x + (x - max_x);
        }

        let min_y = bounds.min.y;
        let max_y = bounds.max.y;

        // Wrap vertically.
        if y < min_y {
            y = max_y - (y - min_y);
        } else if y > max_y {
            y = min_y + (y - max_y);
        }

        IVec2 { x, y }
    }


    let mut next_gen: HashMap<IVec2, bool> = HashMap::with_capacity(
        (life.bounds.width() * life.bounds.height())
            .try_into()
            .unwrap(),
    );

    for y in life.bounds.min.y..life.bounds.max.y {
        for x in life.bounds.min.x..life.bounds.max.x {
            let pt = IVec2::new(x, y);

            // We count the number of life cells, including the inner cell, in the neighborhood
            // of each cell. If the count is 3, then the state of the inner cell in the next generation
            // is life; if the count is 4, then the state of the inner cell stays the same; if the
            // count is anything else, then the state of the inner cell is death.

            let mut count = 0;
            if life.cells.get(&pt).copied().unwrap_or_default() {
                count += 1;
            }

            for offset in NEIGHBOR_OFFSETS {
                let pt = wrap(&life.bounds, pt + offset);
                if life.cells.get(&pt).copied().unwrap_or_default() {
                    count += 1;
                }
            }

            match count {
                3 => {
                    // Life.
                    next_gen.insert(pt, true);
                }
                4 => {
                    // Same. If cell was empty before, it was dead (`false`).
                    next_gen.insert(pt, life.cells.get(&pt).copied().unwrap_or_default());
                }
                _ => {
                    // Death.
                    next_gen.insert(pt, false);
                }
            }
        }
    }

    life.cells = next_gen;
    info!("Tick.");
}


fn update_cell_sprites(
    world: Res<'_, Life>,
    mut q_sprites: Query<'_, '_, (&Position, &mut TextureAtlasSprite)>,
) {
    use config::cells::{DEAD_COLOR, LIVE_COLOR};

    for (position, mut sprite) in &mut q_sprites {
        if world.cells.get(&position.0).copied().unwrap_or_default() {
            sprite.index = 254;
            sprite.color = LIVE_COLOR;
        } else {
            sprite.index = 255;
            sprite.color = DEAD_COLOR;
        }
    }
}
