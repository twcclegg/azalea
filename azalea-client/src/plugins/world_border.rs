//! The world border, as the server sends it.

use azalea_core::{position::Vec3, tick::GameTick};
use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;

/// Keeps every client's [`WorldBorder`] moving while it grows or shrinks.
pub struct WorldBorderPlugin;
impl Plugin for WorldBorderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(GameTick, tick_world_borders);
    }
}

/// The world border of the world that a client is in, as vanilla's
/// `WorldBorder`: a square around a center, which the server can grow or shrink
/// over a number of ticks.
#[derive(Clone, Component, Debug, PartialEq)]
pub struct WorldBorder {
    pub center_x: f64,
    pub center_z: f64,
    /// The edges never go further from 0 than this.
    pub absolute_max_size: f64,
    /// How close to the border the player can be before vanilla's client shows
    /// a warning, in blocks.
    pub warning_blocks: u32,
    /// How many ticks before a shrinking border would reach the player that
    /// vanilla's client shows a warning.
    pub warning_time: u32,
    extent: BorderExtent,
}

#[derive(Clone, Debug, PartialEq)]
enum BorderExtent {
    Static {
        size: f64,
    },
    Moving {
        from: f64,
        to: f64,
        /// The number of ticks the move takes.
        duration: u64,
        /// The number of ticks left until it's done.
        remaining: u64,
        size: f64,
        previous_size: f64,
    },
}

/// Whether the border is still, growing or shrinking.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BorderStatus {
    Stationary,
    Growing,
    Shrinking,
}

impl Default for WorldBorder {
    /// Vanilla's default border, which is as large as it can be.
    fn default() -> Self {
        Self {
            center_x: 0.,
            center_z: 0.,
            absolute_max_size: 29_999_984.,
            warning_blocks: 5,
            warning_time: 15 * 20,
            extent: BorderExtent::Static { size: 5.999_997e7 },
        }
    }
}

impl WorldBorder {
    /// The border's size (its width along each axis) right now.
    pub fn size(&self) -> f64 {
        match self.extent {
            BorderExtent::Static { size } => size,
            BorderExtent::Moving { size, .. } => size,
        }
    }

    /// The size the border is moving to, or its size if it's still.
    pub fn lerp_target(&self) -> f64 {
        match self.extent {
            BorderExtent::Static { size } => size,
            BorderExtent::Moving { to, .. } => to,
        }
    }

    /// The number of ticks until the border stops moving.
    pub fn lerp_remaining_ticks(&self) -> u64 {
        match self.extent {
            BorderExtent::Static { .. } => 0,
            BorderExtent::Moving { remaining, .. } => remaining,
        }
    }

    /// How fast the border's size is changing, in blocks per tick.
    pub fn lerp_speed(&self) -> f64 {
        match self.extent {
            BorderExtent::Static { .. } => 0.,
            BorderExtent::Moving {
                from, to, duration, ..
            } => (from - to).abs() / duration as f64,
        }
    }

    pub fn status(&self) -> BorderStatus {
        match self.extent {
            BorderExtent::Moving { from, to, .. } if to < from => BorderStatus::Shrinking,
            BorderExtent::Moving { .. } => BorderStatus::Growing,
            BorderExtent::Static { .. } => BorderStatus::Stationary,
        }
    }

    /// Set the border's size, stopping it if it was moving.
    pub fn set_size(&mut self, size: f64) {
        self.extent = BorderExtent::Static { size };
    }

    /// Start moving the border from one size to another over `ticks` ticks.
    pub fn lerp_size_between(&mut self, from: f64, to: f64, ticks: u64) {
        self.extent = if from == to || ticks == 0 {
            BorderExtent::Static { size: to }
        } else {
            BorderExtent::Moving {
                from,
                to,
                duration: ticks,
                remaining: ticks,
                size: from,
                previous_size: from,
            }
        };
    }

    /// Advance a moving border by one tick.
    pub fn tick(&mut self) {
        let BorderExtent::Moving {
            from,
            to,
            duration,
            remaining,
            size,
            previous_size,
        } = &mut self.extent
        else {
            return;
        };
        *remaining = remaining.saturating_sub(1);
        *previous_size = *size;
        let progress = (*duration - *remaining) as f64 / *duration as f64;
        *size = if progress < 1. {
            *from + (*to - *from) * progress
        } else {
            *to
        };
        if *remaining == 0 {
            self.extent = BorderExtent::Static { size: *to };
        }
    }

    /// The border's size `partial_tick` of the way from the last tick to the
    /// current one.
    fn size_at(&self, partial_tick: f64) -> f64 {
        match self.extent {
            BorderExtent::Static { size } => size,
            BorderExtent::Moving {
                size,
                previous_size,
                ..
            } => previous_size + (size - previous_size) * partial_tick,
        }
    }

    fn edge(&self, center: f64, offset: f64) -> f64 {
        (center + offset).clamp(-self.absolute_max_size, self.absolute_max_size)
    }

    pub fn min_x(&self) -> f64 {
        self.edge(self.center_x, -self.size() / 2.)
    }
    pub fn max_x(&self) -> f64 {
        self.edge(self.center_x, self.size() / 2.)
    }
    pub fn min_z(&self) -> f64 {
        self.edge(self.center_z, -self.size() / 2.)
    }
    pub fn max_z(&self) -> f64 {
        self.edge(self.center_z, self.size() / 2.)
    }

    /// The edges as (min x, max x, min z, max z), `partial_tick` of the way
    /// from the last tick to the current one, for drawing a moving border
    /// smoothly.
    pub fn edges_at(&self, partial_tick: f64) -> (f64, f64, f64, f64) {
        let half = self.size_at(partial_tick) / 2.;
        (
            self.edge(self.center_x, -half),
            self.edge(self.center_x, half),
            self.edge(self.center_z, -half),
            self.edge(self.center_z, half),
        )
    }

    /// Whether the position is inside the border, or within `margin` blocks
    /// outside of it.
    pub fn is_within_bounds(&self, pos: Vec3, margin: f64) -> bool {
        pos.x >= self.min_x() - margin
            && pos.x < self.max_x() + margin
            && pos.z >= self.min_z() - margin
            && pos.z < self.max_z() + margin
    }

    /// The distance from the position to the nearest edge of the border, which
    /// is negative if it's outside.
    pub fn distance_to_border(&self, pos: Vec3) -> f64 {
        (pos.z - self.min_z())
            .min(self.max_z() - pos.z)
            .min(pos.x - self.min_x())
            .min(self.max_x() - pos.x)
    }
}

pub fn tick_world_borders(mut query: Query<&mut WorldBorder>) {
    for mut border in &mut query {
        if border.lerp_remaining_ticks() > 0 {
            border.tick();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrinks_over_ticks() {
        let mut border = WorldBorder::default();
        border.lerp_size_between(100., 50., 10);
        assert_eq!(border.status(), BorderStatus::Shrinking);
        assert_eq!((border.min_x(), border.max_x()), (-50., 50.));

        for _ in 0..4 {
            border.tick();
        }
        assert_eq!(border.size(), 80.);
        assert_eq!(border.lerp_speed(), 5.);
        assert_eq!(border.lerp_remaining_ticks(), 6);
        // between ticks, it's between the last two sizes (85 and 80)
        assert_eq!(border.edges_at(0.5).1, 41.25);

        for _ in 0..6 {
            border.tick();
        }
        assert_eq!(border.size(), 50.);
        assert_eq!(border.status(), BorderStatus::Stationary);
        assert_eq!(border.lerp_speed(), 0.);
    }

    #[test]
    fn bounds_and_distance() {
        let mut border = WorldBorder::default();
        border.center_x = 10.;
        border.set_size(20.);
        assert!(border.is_within_bounds(Vec3::new(0., 64., -10.), 0.));
        // the max edges are exclusive
        assert!(!border.is_within_bounds(Vec3::new(20., 64., 0.), 0.));
        assert!(border.is_within_bounds(Vec3::new(20.5, 64., 0.), 1.));
        assert_eq!(border.distance_to_border(Vec3::new(15., 64., 0.)), 5.);
        assert_eq!(border.distance_to_border(Vec3::new(25., 64., 0.)), -5.);
    }

    #[test]
    fn edges_are_clamped() {
        let mut border = WorldBorder::default();
        border.center_x = 29_999_990.;
        border.set_size(100.);
        assert_eq!(border.max_x(), 29_999_984.);
    }
}
