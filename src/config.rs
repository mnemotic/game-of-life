//
// Copyright (c) 2023 Martin Green. All rights reserved.
//

pub mod window {
    pub const WIDTH: u32 = 1280;
    pub const HEIGHT: u32 = 720;
}

pub mod cells {
    use std::sync::LazyLock;

    use bevy::color::Srgba;
    use bevy::math::Vec2;

    use crate::color_gradient::{ColorGradient, ColorPoint};

    pub const SPRITE_SIZE: Vec2 = Vec2::splat(20.0);
    pub const SPRITE_WORLD_OFFSET: Vec2 = Vec2::new(10.0, 10.0);

    pub const DEAD_COLOR: Srgba = bevy::color::palettes::css::GRAY;

    pub fn get_age_color(q: f32) -> Srgba {
        static GRADIENT: LazyLock<ColorGradient> = LazyLock::new(|| {
            let mut gradient = ColorGradient::new();
            gradient.insert(ColorPoint::new(0.0, Srgba::rgb_u8(143, 0, 255))); // violet (electric)
            gradient.insert(ColorPoint::new(0.2, Srgba::rgb_u8(178, 34, 34))); // red (fire brick)
            gradient.insert(ColorPoint::new(0.4, Srgba::rgb_u8(255, 121, 0))); // orange (safety)
            gradient.insert(ColorPoint::new(0.6, Srgba::rgb_u8(255, 211, 0))); // yellow (ncs)
            gradient.insert(ColorPoint::new(0.8, Srgba::rgb_u8(50, 205, 50))); // green (lime)
            gradient.insert(ColorPoint::new(1.0, Srgba::rgb_u8(0, 183, 235))); // cyan (sub. primary)

            gradient
        });

        GRADIENT.sample(q)
    }
}

pub mod sim {
    pub const DEFAULT_TICKS_PER_SECOND: i32 = 4;
}
