//
// Copyright (c) 2023 Martin Green. All rights reserved.
//

use bevy::asset::LoadState;
use bevy::prelude::*;

use crate::GameState;


#[derive(Default, Resource, Deref, DerefMut)]
pub struct GameAssets(pub Vec<HandleUntyped>);


#[derive(Resource, Deref, DerefMut)]
pub struct GlyphAtlas(pub Handle<TextureAtlas>);


pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>()
            .add_systems(Startup, load_assets)
            .add_systems(
                Update,
                check_asset_loading.run_if(in_state(GameState::Startup)),
            );
    }
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
            info!("Assets loaded");
            next_state.set(GameState::Running);
        }
        LoadState::Failed => panic!("Failed to load assets"),
        _ => {}
    }
}
