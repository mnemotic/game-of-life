//
// Copyright (c) 2023 Martin Green. All rights reserved.
//

use bevy::prelude::*;
use bevy_pixel_camera::{PixelCameraPlugin, PixelViewport, PixelZoom};

use crate::config;


#[derive(Component)]
pub struct MainCamera;


pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PixelCameraPlugin)
            .add_systems(Startup, setup_camera);
    }
}


fn setup_camera(mut commands: Commands<'_, '_>) {
    #[allow(clippy::cast_possible_wrap)]
    commands.spawn((
        Camera2dBundle::default(),
        PixelZoom::FitSize {
            width: config::window::WIDTH as i32,
            height: config::window::HEIGHT as i32,
        },
        PixelViewport,
        MainCamera,
    ));
}
