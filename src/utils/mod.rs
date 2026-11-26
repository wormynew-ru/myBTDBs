use bevy::prelude::*;

use crate::core::types::Side;
use crate::data::maps_cfg::{ARENA_BOTTOM, ARENA_TOP, DIVIDER_HALF, HALF_W, ROAD_WIDTH};
use crate::data::towers_cfg::TOWER_RADIUS;

pub fn dist_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq <= f32::EPSILON {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    let proj = a + ab * t;
    p.distance(proj)
}

pub fn path_total_length(waypoints: &[Vec2]) -> f32 {
    let mut total = 0.0;
    for pair in waypoints.windows(2) {
        total += pair[0].distance(pair[1]);
    }
    total
}

pub fn sample_path(waypoints: &[Vec2], distance_along: f32) -> (Vec2, bool) {
    if waypoints.is_empty() {
        return (Vec2::ZERO, true);
    }
    if distance_along <= 0.0 {
        return (waypoints[0], false);
    }

    let mut remaining = distance_along;
    for pair in waypoints.windows(2) {
        let a = pair[0];
        let b = pair[1];
        let seg_len = a.distance(b);
        if remaining <= seg_len {
            let t = if seg_len > f32::EPSILON {
                remaining / seg_len
            } else {
                0.0
            };
            return (a.lerp(b, t), false);
        }
        remaining -= seg_len;
    }

    (waypoints[waypoints.len() - 1], true)
}

pub fn circles_overlap(a: Vec2, ra: f32, b: Vec2, rb: f32) -> bool {
    a.distance_squared(b) <= (ra + rb) * (ra + rb)
}

pub fn is_valid_tower_spot(
    pos: Vec2,
    side: Side,
    waypoints: &[Vec2],
    existing_towers: &[Vec2],
) -> bool {
    let margin = TOWER_RADIUS + 6.0;
    let (min_x, max_x) = match side {
        Side::Player => (-HALF_W + margin, -DIVIDER_HALF - margin),
        Side::Bot => (DIVIDER_HALF + margin, HALF_W - margin),
    };
    let min_y = ARENA_BOTTOM + margin;
    let max_y = ARENA_TOP - margin;

    if pos.x < min_x || pos.x > max_x || pos.y < min_y || pos.y > max_y {
        return false;
    }

    let road_clearance = ROAD_WIDTH * 0.5 + TOWER_RADIUS * 0.72;
    for pair in waypoints.windows(2) {
        if dist_to_segment(pos, pair[0], pair[1]) < road_clearance {
            return false;
        }
    }

    let min_tower_dist = TOWER_RADIUS * 2.0;
    for &other in existing_towers {
        if pos.distance(other) < min_tower_dist {
            return false;
        }
    }

    true
}
