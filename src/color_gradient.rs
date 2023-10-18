//
// Copyright (c) 2023 Martin Green <martin@bk2x.com>. All rights reserved.
//

use bevy::prelude::*;

use ordered_float::OrderedFloat;


pub struct ColorGradient {
    /// Sampling points.
    points: Vec<ColorPoint>,
}

impl ColorGradient {
    /// Create a new gradient without any sampling points.
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    /// Insert a sampling point into the gradient.
    pub fn insert(&mut self, point: ColorPoint) {
        // @TODO:
        // Check if this gradient already has a sampling point at `point.point`. If it does, replace
        // it with the new one.
        self.points.push(point);
        self.points.sort_unstable_by_key(|pt| pt.point);
    }

    /// Sample the gradient at the given point. `point` will be clamped to [0.0, 1.0] range. Panics
    /// if the gradient has less than 2 sampling points
    pub fn sample(&self, point: f32) -> Color {
        assert!(self.points.len() >= 2);

        let point: OrderedFloat<f32> = point.clamp(0.0, 1.0).into();

        match self
            .points
            .binary_search_by_key(&point, |sample_pt| sample_pt.point)
        {
            // Exact match. Just return the color value of the sampling point.
            Ok(i) => self.points[i].value,

            // Sampling a point on the left of the first sampling point on the gradient.
            Err(0) => self.points.first().unwrap().value,

            // Sampling a point on the right of the last sampling point on the gradient.
            Err(i) if i >= self.len() => self.points.last().unwrap().value,

            // Sampling a point between two sampling points on the gradient.
            Err(i) => {
                let left = self.points[i - 1];
                let right = self.points[i];

                assert!(point > left.point);
                assert!(point < right.point);

                // Remap the sampling point into the range between `left` and `right` for interpolation.
                Color::from(Vec4::from(left.value).lerp(
                    Vec4::from(right.value),
                    ((point - left.point) / (right.point - left.point)).into(),
                ))
            }
        }
    }

    /// Return the number of sampling points in this gradient.
    pub fn len(&self) -> usize {
        self.points.len()
    }
}

impl Default for ColorGradient {
    fn default() -> Self {
        // Opaque black to opaque white gradient.
        Self {
            points: vec![
                ColorPoint::new(0.0, Color::rgba_linear(0.0, 0.0, 0.0, 1.0)),
                ColorPoint::new(1.0, Color::rgba_linear(1.0, 1.0, 1.0, 1.0)),
            ],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ColorPoint {
    point: OrderedFloat<f32>,
    value: Color,
}

impl ColorPoint {
    /// Create a new color sampling point. `point` will be clamped to [0.0, 1.0] range.
    pub fn new(point: f32, value: Color) -> Self {
        Self {
            point: point.clamp(0.0, 1.0).into(),
            value,
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::{ColorGradient, ColorPoint};

    #[test]
    pub fn test_default_gradient() {
        let gradient = ColorGradient::default();

        assert_eq!(gradient.sample(0.0), Color::rgba_linear(0.0, 0.0, 0.0, 1.0));
        assert_eq!(gradient.sample(0.5), Color::rgba_linear(0.5, 0.5, 0.5, 1.0));
        assert_eq!(gradient.sample(1.0), Color::rgba_linear(1.0, 1.0, 1.0, 1.0));
    }

    #[test]
    pub fn test_custom_gradient() {
        let mut gradient = ColorGradient::new();

        gradient.insert(ColorPoint::new(0.2, Color::rgba_linear(0.0, 0.0, 0.0, 1.0)));
        gradient.insert(ColorPoint::new(
            0.75,
            Color::rgba_linear(1.0, 1.0, 1.0, 1.0),
        ));

        assert_eq!(
            gradient.sample(-0.1),
            Color::rgba_linear(0.0, 0.0, 0.0, 1.0)
        );
        assert_eq!(gradient.sample(0.0), Color::rgba_linear(0.0, 0.0, 0.0, 1.0));
        assert_eq!(gradient.sample(0.2), Color::rgba_linear(0.0, 0.0, 0.0, 1.0));
        assert_eq!(
            gradient.sample(0.35),
            Color::rgba_linear(0.272_727_25, 0.272_727_25, 0.272_727_25, 1.0)
        );
        assert_eq!(
            gradient.sample(0.5),
            Color::rgba_linear(0.545_454_56, 0.545_454_56, 0.545_454_56, 1.0)
        );
        assert_eq!(
            gradient.sample(0.65),
            Color::rgba_linear(0.818_181_75, 0.818_181_75, 0.818_181_75, 1.0)
        );
        assert_eq!(
            gradient.sample(0.75),
            Color::rgba_linear(1.0, 1.0, 1.0, 1.0)
        );
        assert_eq!(gradient.sample(1.0), Color::rgba_linear(1.0, 1.0, 1.0, 1.0));
        assert_eq!(gradient.sample(1.1), Color::rgba_linear(1.0, 1.0, 1.0, 1.0));
    }
}
