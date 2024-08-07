//
// Copyright (c) 2023 Martin Green. All rights reserved.
//

use bevy::asset::LoadState;
use bevy::prelude::*;

use crate::AppState;


#[derive(Default, Resource, Deref, DerefMut)]
pub struct GameAssets(pub Vec<UntypedHandle>);


#[derive(Default, Resource)]
pub struct GlyphAtlas(pub Handle<TextureAtlasLayout>, pub Handle<Image>);


pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>()
            .add_systems(Startup, load_fontsheet)
            .add_systems(
                Update,
                check_fontsheet_loading.run_if(in_state(AppState::Startup)),
            );
    }
}


fn load_fontsheet(
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
    mut assets: ResMut<'_, GameAssets>,
    mut texture_atlases: ResMut<'_, Assets<TextureAtlasLayout>>,
) {
    const FONTSHEET_PATH: &str = "cp437_10x10.png";

    let fontsheet = asset_server.load(FONTSHEET_PATH);
    assets.push(fontsheet.clone().untyped());

    let layout = TextureAtlasLayout::from_grid(Vec2::splat(10.0), 16, 16, None, None);
    commands.insert_resource(GlyphAtlas(texture_atlases.add(layout), fontsheet));
}


fn check_fontsheet_loading(
    asset_server: Res<'_, AssetServer>,
    assets: Res<'_, GameAssets>,
    mut next_state: ResMut<'_, NextState<AppState>>,
) {
    let mut load_state = LoadState::Loaded;
    for handle in assets.iter() {
        if handle.path().is_some() {
            let handle_id = handle.id();
            match asset_server.get_load_state(handle_id) {
                Some(handle_load_state) => match handle_load_state {
                    LoadState::Loaded => continue,
                    LoadState::Loading => load_state = LoadState::Loading,
                    LoadState::Failed => {
                        load_state = LoadState::Failed;
                        break;
                    }
                    LoadState::NotLoaded => {
                        load_state = LoadState::NotLoaded;
                        break;
                    }
                },
                None => panic!("no such asset"),
            }
        }
    }

    match load_state {
        LoadState::Loading | LoadState::NotLoaded => {
            info!("Loading assets...");
        }
        LoadState::Loaded => {
            info!("Assets loaded");
            next_state.set(AppState::Running);
        }
        LoadState::Failed => panic!("failed to load assets"),
    }
}
