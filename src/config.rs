//
// Copyright (c) 2023 Martin Green. All rights reserved.
//

pub mod window {
    pub const WIDTH: u32 = 1280;
    pub const HEIGHT: u32 = 720;
}

pub mod cells {
    use bevy::math::Vec2;
    use bevy::prelude::Color;
    use once_cell::sync::Lazy;

    use crate::color_gradient::{ColorGradient, ColorPoint};

    pub const SPRITE_SIZE: Vec2 = Vec2::splat(20.0);
    pub const SPRITE_WORLD_OFFSET: Vec2 = Vec2::new(10.0, 10.0);

    pub const DEAD_COLOR: Color = Color::GRAY;

    pub fn get_age_color(age: usize) -> Color {
        // @REVIEW
        //
        // Replace with [`std::cell::LazyCell`](https://doc.rust-lang.org/std/cell/struct.LazyCell.html)
        // once it drops in stable.

        static GRADIENT: Lazy<ColorGradient> = Lazy::new(|| {
            let mut gradient = ColorGradient::new();
            gradient.insert(ColorPoint::new(0.0, Color::rgb_u8(139, 190, 28)));
            gradient.insert(ColorPoint::new(0.2, Color::rgb_u8(162, 201, 38)));
            gradient.insert(ColorPoint::new(0.4, Color::rgb_u8(185, 212, 47)));
            gradient.insert(ColorPoint::new(0.6, Color::rgb_u8(209, 222, 57)));
            gradient.insert(ColorPoint::new(0.8, Color::rgb_u8(232, 233, 66)));
            gradient.insert(ColorPoint::new(1.0, Color::rgb_u8(255, 244, 76)));

            gradient
        });

        GRADIENT.sample((age as f32) / 10.0)
    }
}

pub mod sim {
    pub const DEFAULT_TICKS_PER_SECOND: i32 = 4;
}
