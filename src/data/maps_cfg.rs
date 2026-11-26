use bevy::prelude::*;

use crate::core::state::MapId;
use crate::core::types::Side;

pub const ARENA_W: f32 = 1280.0;
pub const ARENA_H: f32 = 720.0;
pub const HALF_W: f32 = 640.0;
pub const DIVIDER_HALF: f32 = 4.0;
pub const ROAD_WIDTH: f32 = 36.0;
pub const ARENA_TOP: f32 = 290.0;
pub const ARENA_BOTTOM: f32 = -265.0;

#[derive(Debug, Clone, Copy)]
pub struct MapCfg {
    pub grass_color: Color,
    pub road_color: Color,
    pub road_border_color: Color,
    pub left_waypoints: &'static [Vec2],
}

const MEADOW_WAYPOINTS: &[Vec2] = &[
    Vec2::new(-640.0, 200.0),
    Vec2::new(-160.0, 200.0),
    Vec2::new(-160.0, 55.0),
    Vec2::new(-490.0, 55.0),
    Vec2::new(-490.0, -100.0),
    Vec2::new(-120.0, -100.0),
    Vec2::new(-120.0, -220.0),
    Vec2::new(-410.0, -220.0),
    Vec2::new(-410.0, -265.0),
];

const ZIGZAG_WAYPOINTS: &[Vec2] = &[
    Vec2::new(-530.0, 290.0),
    Vec2::new(-530.0, -205.0),
    Vec2::new(-375.0, -205.0),
    Vec2::new(-375.0, 220.0),
    Vec2::new(-220.0, 220.0),
    Vec2::new(-220.0, -205.0),
    Vec2::new(-75.0, -205.0),
    Vec2::new(-75.0, 90.0),
    Vec2::new(-4.0, 90.0),
];

const CANYON_WAYPOINTS: &[Vec2] = &[
    Vec2::new(-640.0, 235.0),
    Vec2::new(-100.0, 235.0),
    Vec2::new(-100.0, -225.0),
    Vec2::new(-545.0, -225.0),
    Vec2::new(-545.0, 125.0),
    Vec2::new(-225.0, 125.0),
    Vec2::new(-225.0, -110.0),
    Vec2::new(-420.0, -110.0),
    Vec2::new(-420.0, 15.0),
    Vec2::new(-315.0, 15.0),
];

pub fn map_cfg(map_id: MapId) -> MapCfg {
    match map_id {
        MapId::Meadow => MapCfg {
            grass_color: Color::srgb(0.24, 0.49, 0.22),
            road_color: Color::srgb(0.76, 0.61, 0.42),
            road_border_color: Color::srgb(0.55, 0.42, 0.26),
            left_waypoints: MEADOW_WAYPOINTS,
        },
        MapId::Zigzag => MapCfg {
            grass_color: Color::srgb(0.18, 0.43, 0.30),
            road_color: Color::srgb(0.71, 0.64, 0.52),
            road_border_color: Color::srgb(0.48, 0.42, 0.33),
            left_waypoints: ZIGZAG_WAYPOINTS,
        },
        MapId::Canyon => MapCfg {
            grass_color: Color::srgb(0.55, 0.36, 0.24),
            road_color: Color::srgb(0.83, 0.71, 0.51),
            road_border_color: Color::srgb(0.61, 0.49, 0.32),
            left_waypoints: CANYON_WAYPOINTS,
        },
    }
}

pub fn waypoints_for_side(map_id: MapId, side: Side) -> Vec<Vec2> {
    let base = map_cfg(map_id).left_waypoints;
    match side {
        Side::Player => base.to_vec(),
        Side::Bot => base.iter().map(|p| Vec2::new(-p.x, p.y)).collect(),
    }
}
