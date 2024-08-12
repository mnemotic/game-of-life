//
// Copyright (c) 2023 Martin Green. All rights reserved.
//

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use game::Life;

use crate::assets::GlyphAtlas;


mod assets;
mod camera;
mod color_gradient;
mod config;
mod game;
mod input;
mod ui;


#[derive(Default, Resource)]
struct WindowFocus {
    focused: bool,
}

#[derive(Event)]
struct WindowFocused {
    focused: bool,
}


fn main() {
    // @REVIEW: See <https://github.com/bevy-cheatbook/bevy-cheatbook/issues/196>.
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let width = config::window::WIDTH;
    let height = config::window::HEIGHT;

    #[allow(clippy::cast_precision_loss)]
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Life::new(width / 20, height / 20))
        .add_event::<WindowFocused>()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (width as f32, height as f32).into(),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        title: String::from("Conway's Game of Life"),
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .init_state::<AppState>()
        .add_plugins(input::InputPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(assets::AssetPlugin)
        .add_plugins(game::GamePlugin)
        .add_systems(
            Startup,
            |mut next_state: ResMut<'_, NextState<AppState>>| next_state.set(AppState::Startup),
        )
        .add_systems(PreUpdate, track_window_focus)
        .add_systems(
            OnEnter(AppState::Running),
            init_presentation.run_if(run_once()),
        )
        .add_systems(Update, update_presentation)
        .run();
}


#[derive(States, Clone, Debug, Default, PartialEq, Eq, Hash)]
enum AppState {
    #[default]
    None,
    Startup,
    Running,
    Paused,
}


#[derive(Component, Deref)]
struct Position(pub IVec2);


fn init_presentation(
    mut commands: Commands<'_, '_>,
    world: Res<'_, Life>,
    glyphs: Res<'_, GlyphAtlas>,
) {
    use config::cells::{get_age_color, DEAD_COLOR, SPRITE_SIZE, SPRITE_WORLD_OFFSET};

    for y in world.bounds.min.y..world.bounds.max.y {
        for x in world.bounds.min.x..world.bounds.max.x {
            let (atlas, sprite) = if world.cells.contains_key(&IVec2::new(x, y)) {
                (
                    TextureAtlas {
                        layout: glyphs.0.clone(),
                        index: 254,
                    },
                    Sprite {
                        color: get_age_color(0f32).into(),
                        custom_size: Some(SPRITE_SIZE),
                        ..default()
                    },
                )
            } else {
                (
                    TextureAtlas {
                        layout: glyphs.0.clone(),
                        index: 255,
                    },
                    Sprite {
                        color: DEAD_COLOR.into(),
                        custom_size: Some(SPRITE_SIZE),
                        ..default()
                    },
                )
            };

            #[allow(clippy::cast_precision_loss)]
            let transform = Transform::from_translation(
                (Vec2::new(x as f32, y as f32) * SPRITE_SIZE + SPRITE_WORLD_OFFSET).extend(0.0),
            );
            commands.spawn((
                SpriteBundle {
                    texture: glyphs.1.clone(),
                    sprite,
                    transform,
                    ..default()
                },
                atlas,
                Position(IVec2::new(x, y)),
            ));
        }
    }
}


/// Update the presentation.
fn update_presentation(
    life: Res<'_, Life>,
    mut q_sprites: Query<'_, '_, (&Position, &mut TextureAtlas, &mut Sprite)>,
) {
    use config::cells::{get_age_color, DEAD_COLOR};

    for (position, mut atlas, mut sprite) in &mut q_sprites {
        if let Some(cell) = life.cells.get(position) {
            // FIXME: Magic number.
            atlas.index = 254;

            // REVIEW:
            //   There should be a better way to handle this. Fortunately, any bugs will only
            //   manifest when cell age is greater than 2^24 (16,777,216).
            #[allow(clippy::cast_precision_loss)]
            let q = (cell.age as f32) / (life.max_age as f32);
            sprite.color = get_age_color(q).into();
        } else {
            // FIXME: Magic number.
            atlas.index = 255;
            sprite.color = DEAD_COLOR.into();
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
    for event in ev_focused_bevy.read() {
        debug!("{event:?}");
        focus.focused = event.focused;
    }

    if focus.focused != focused {
        ev_focused.send(WindowFocused {
            focused: focus.focused,
        });
    }
}
