use bevy::prelude::*;

use crate::core::state::MapId;
use crate::core::types::{Side, TowerType};
use crate::data::maps_cfg::{waypoints_for_side, ARENA_BOTTOM, ARENA_TOP, DIVIDER_HALF, HALF_W};
use crate::utils::{is_valid_tower_spot, path_total_length, sample_path};

pub fn find_bot_spot(
    map_id: MapId,
    existing_bot_towers: &[Vec2],
    near_exit: bool,
) -> Option<Vec2> {
    let wps = waypoints_for_side(map_id, Side::Bot);
    let total_len = path_total_length(&wps).max(1.0);
    let mut road_samples = Vec::new();
    let mut d = 0.0;
    while d <= total_len {
        let (pt, _) = sample_path(&wps, d);
        road_samples.push((pt, d / total_len));
        d += 14.0;
    }

    let mut best: Option<(Vec2, f32)> = None;
    let mut gx = DIVIDER_HALF + 42.0;
    while gx <= HALF_W - 40.0 {
        let mut gy = ARENA_BOTTOM + 42.0;
        while gy <= ARENA_TOP - 42.0 {
            let candidate = Vec2::new(gx, gy);
            if is_valid_tower_spot(candidate, Side::Bot, &wps, existing_bot_towers) {
                let mut score = 0.0;
                let mut max_prog = -1.0_f32;
                let mut hits = 0_u32;
                for &(pt, prog) in &road_samples {
                    let dist = candidate.distance(pt);
                    if near_exit {
                        if dist <= 68.0 {
                            max_prog = max_prog.max(prog);
                            hits += 1;
                        }
                    } else {
                        if dist <= 118.0 {
                            score += 1.0;
                        }
                        if (42.0..=85.0).contains(&dist) {
                            score += 0.4;
                        }
                    }
                }
                if !near_exit || hits > 0 {
                    if near_exit {
                        score = max_prog * 100.0 + (hits as f32) * 0.4;
                    }
                    if best.map_or(true, |(_, bs)| score > bs) {
                        best = Some((candidate, score));
                    }
                }
            }
            gy += 32.0;
        }
        gx += 32.0;
    }
    best.map(|(p, _)| p)
}

pub fn find_commander_spot(map_id: MapId, existing_bot_towers: &[Vec2]) -> Option<Vec2> {
    let wps = waypoints_for_side(map_id, Side::Bot);
    let mut best: Option<(Vec2, usize)> = None;
    let mut gx = DIVIDER_HALF + 42.0;
    while gx <= HALF_W - 40.0 {
        let mut gy = ARENA_BOTTOM + 42.0;
        while gy <= ARENA_TOP - 42.0 {
            let candidate = Vec2::new(gx, gy);
            if is_valid_tower_spot(candidate, Side::Bot, &wps, existing_bot_towers) {
                let count = existing_bot_towers
                    .iter()
                    .filter(|&&p| candidate.distance(p) <= 130.0)
                    .count();
                if count > 0 && best.map_or(true, |(_, bc)| count > bc) {
                    best = Some((candidate, count));
                }
            }
            gy += 32.0;
        }
        gx += 32.0;
    }
    best.map(|(p, _)| p)
}

pub fn find_farm_spot(map_id: MapId, existing_bot_towers: &[Vec2]) -> Option<Vec2> {
    let wps = waypoints_for_side(map_id, Side::Bot);
    let mut best: Option<(Vec2, f32)> = None;
    let mut gx = DIVIDER_HALF + 42.0;
    while gx <= HALF_W - 40.0 {
        let mut gy = ARENA_BOTTOM + 42.0;
        while gy <= ARENA_TOP - 42.0 {
            let candidate = Vec2::new(gx, gy);
            if is_valid_tower_spot(candidate, Side::Bot, &wps, existing_bot_towers) {
                let mut min_road_dist = f32::MAX;
                for pair in wps.windows(2) {
                    let seg = pair[1] - pair[0];
                    let len_sq = seg.length_squared().max(0.001);
                    let t = ((candidate - pair[0]).dot(seg) / len_sq).clamp(0.0, 1.0);
                    let proj = pair[0] + seg * t;
                    min_road_dist = min_road_dist.min(candidate.distance(proj));
                }
                if best.map_or(true, |(_, bd)| min_road_dist > bd) {
                    best = Some((candidate, min_road_dist));
                }
            }
            gy += 32.0;
        }
        gx += 32.0;
    }
    best.map(|(p, _)| p)
}

pub fn pick_spot_for_tower(
    map_id: MapId,
    existing_bot_towers: &[Vec2],
    tower_type: TowerType,
) -> Option<Vec2> {
    match tower_type {
        TowerType::Commander
        | TowerType::SoulEater
        | TowerType::ScrapCollector
        | TowerType::Village => find_commander_spot(map_id, existing_bot_towers)
            .or_else(|| find_bot_spot(map_id, existing_bot_towers, false)),
        TowerType::Farm | TowerType::MirageGenerator => {
            find_farm_spot(map_id, existing_bot_towers)
        }
        TowerType::SpikeFactory => find_bot_spot(map_id, existing_bot_towers, true),
        _ => find_bot_spot(map_id, existing_bot_towers, false),
    }
}
