use bevy::prelude::*;

use crate::core::events::{BuildTowerEvent, SendBloonPackEvent, UpgradeTowerEvent};
use crate::core::state::{Difficulty, GameState, MapId, MatchSettings};
use crate::core::types::{Position, Side, SidesResource, TowerType};
use crate::data::bloons_cfg::SEND_PACKS;
use crate::data::maps_cfg::{waypoints_for_side, ARENA_BOTTOM, ARENA_TOP, DIVIDER_HALF, HALF_W};
use crate::data::towers_cfg::{compute_effective_stats, tower_cfg};
use crate::plugins::bloons::Bloon;
use crate::plugins::towers::Tower;
use crate::utils::{is_valid_tower_spot, path_total_length};

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AiState>()
            .add_systems(OnEnter(GameState::Playing), reset_ai_timer)
            .add_systems(
                Update,
                utility_ai_system.run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Resource, Default)]
pub struct AiState {
    pub timer: f32,
}

fn reset_ai_timer(mut ai: ResMut<AiState>, settings: Res<MatchSettings>) {
    ai.timer = settings.difficulty.ai_interval();
}

fn find_best_bot_spot(
    map_id: MapId,
    existing_bot_towers: &[Vec2],
) -> Option<Vec2> {
    let wps = waypoints_for_side(map_id, Side::Bot);
    let mut best: Option<(Vec2, f32)> = None;

    let mut gx = DIVIDER_HALF + 46.0;
    while gx <= HALF_W - 42.0 {
        let mut gy = ARENA_BOTTOM + 44.0;
        while gy <= ARENA_TOP - 44.0 {
            let candidate = Vec2::new(gx, gy);
            if is_valid_tower_spot(candidate, Side::Bot, &wps, existing_bot_towers) {
                let mut score = 0.0;
                for pair in wps.windows(2) {
                    let mid = (pair[0] + pair[1]) * 0.5;
                    for pt in [pair[0], mid, pair[1]] {
                        let d = candidate.distance(pt);
                        if d <= 118.0 {
                            score += 1.0;
                        }
                        if (42.0..=85.0).contains(&d) {
                            score += 0.4;
                        }
                    }
                }
                if best.map_or(true, |(_, bs)| score > bs) {
                    best = Some((candidate, score));
                }
            }
            gy += 36.0;
        }
        gx += 36.0;
    }

    best.map(|(p, _)| p)
}

#[allow(clippy::too_many_arguments)]
fn utility_ai_system(
    time: Res<Time>,
    settings: Res<MatchSettings>,
    sides: Res<SidesResource>,
    mut ai: ResMut<AiState>,
    towers: Query<(Entity, &Side, &Tower, &Position)>,
    bloons: Query<(&Side, &Bloon)>,
    mut build_writer: EventWriter<BuildTowerEvent>,
    mut upgrade_writer: EventWriter<UpgradeTowerEvent>,
    mut send_writer: EventWriter<SendBloonPackEvent>,
) {
    if settings.paused {
        return;
    }
    ai.timer -= time.delta_secs() * settings.speed_mult;
    if ai.timer > 0.0 {
        return;
    }
    ai.timer = settings.difficulty.ai_interval();

    let bot_cash = sides.bot.cash;
    let bot_wps = waypoints_for_side(settings.map_id, Side::Bot);
    let total_len = path_total_length(&bot_wps).max(1.0);

    let mut bot_towers = Vec::new();
    let mut bot_positions = Vec::new();
    let mut player_has_camo = false;
    let mut player_has_lead = false;

    for (entity, side, t, pos) in &towers {
        let stats = compute_effective_stats(t.tower_type, t.path1, t.path2);
        match side {
            Side::Bot => {
                bot_towers.push((entity, t, stats));
                bot_positions.push(pos.0);
            }
            Side::Player => {
                if stats.can_hit_camo {
                    player_has_camo = true;
                }
                if stats.can_pop_lead {
                    player_has_lead = true;
                }
            }
        }
    }

    let mut max_threat = 0.0_f32;
    let mut has_camo_threat = false;
    let mut has_lead_threat = false;
    for (side, bloon) in &bloons {
        if *side == Side::Bot {
            max_threat = max_threat.max(bloon.path_progress / total_len);
            if bloon.camo {
                has_camo_threat = true;
            }
            if bloon.lead {
                has_lead_threat = true;
            }
        }
    }

    let bot_has_camo = bot_towers.iter().any(|(_, _, s)| s.can_hit_camo);
    let bot_has_lead = bot_towers.iter().any(|(_, _, s)| s.can_pop_lead);

    // 1. Starter defense
    if bot_towers.is_empty() && bot_cash >= tower_cfg(TowerType::DartMonkey).cost {
        if let Some(pos) = find_best_bot_spot(settings.map_id, &bot_positions) {
            build_writer.send(BuildTowerEvent {
                side: Side::Bot,
                tower_type: TowerType::DartMonkey,
                pos,
            });
            return;
        }
    }

    // 2. Counter Camo
    if (has_camo_threat || (settings.difficulty != Difficulty::Easy && sides.wave >= 4))
        && !bot_has_camo
    {
        if bot_cash >= tower_cfg(TowerType::NinjaMonkey).cost {
            if let Some(pos) = find_best_bot_spot(settings.map_id, &bot_positions) {
                build_writer.send(BuildTowerEvent {
                    side: Side::Bot,
                    tower_type: TowerType::NinjaMonkey,
                    pos,
                });
                return;
            }
        }
        if has_camo_threat {
            return;
        }
    }

    // 3. Counter Lead
    if (has_lead_threat || (settings.difficulty != Difficulty::Easy && sides.wave >= 5))
        && !bot_has_lead
    {
        if bot_cash >= tower_cfg(TowerType::Cannon).cost {
            if let Some(pos) = find_best_bot_spot(settings.map_id, &bot_positions) {
                build_writer.send(BuildTowerEvent {
                    side: Side::Bot,
                    tower_type: TowerType::Cannon,
                    pos,
                });
                return;
            }
        }
        if has_lead_threat {
            return;
        }
    }

    // 4. Urgent defense or upgrade
    if max_threat > 0.52 || bot_towers.len() < 2 {
        if let Some((entity, tower, _)) = bot_towers.iter().find(|(_, t, _)| t.path1 < 3 || t.path2 < 3) {
            let branch = if tower.path1 <= tower.path2 { 1 } else { 2 };
            upgrade_writer.send(UpgradeTowerEvent {
                side: Side::Bot,
                tower_entity: *entity,
                branch,
            });
            return;
        }
        if let Some(pos) = find_best_bot_spot(settings.map_id, &bot_positions) {
            let pick = match bot_towers.len() % 4 {
                1 => TowerType::Cannon,
                2 => TowerType::NinjaMonkey,
                3 => TowerType::IceTower,
                _ => TowerType::DartMonkey,
            };
            if bot_cash >= tower_cfg(pick).cost {
                build_writer.send(BuildTowerEvent {
                    side: Side::Bot,
                    tower_type: pick,
                    pos,
                });
                return;
            }
        }
    }

    // 5. Rush Player when weak against Camo/Lead or late game
    if settings.difficulty != Difficulty::Easy && sides.wave >= 4 && max_threat < 0.45 {
        if !player_has_camo && bot_cash >= SEND_PACKS[4].cost {
            send_writer.send(SendBloonPackEvent {
                sender: Side::Bot,
                pack_index: 4,
            });
            return;
        }
        if sides.wave >= 5 && !player_has_lead && bot_cash >= SEND_PACKS[5].cost {
            send_writer.send(SendBloonPackEvent {
                sender: Side::Bot,
                pack_index: 5,
            });
            return;
        }
        if settings.difficulty == Difficulty::Hard && sides.wave >= 8 && bot_cash >= SEND_PACKS[7].cost {
            send_writer.send(SendBloonPackEvent {
                sender: Side::Bot,
                pack_index: 7,
            });
            return;
        }
    }

    // 6. Calm phase Eco scaling
    if max_threat < 0.42 {
        if let Some((idx, _)) = SEND_PACKS
            .iter()
            .enumerate()
            .rev()
            .find(|(_, p)| sides.wave >= p.unlock_wave && bot_cash >= p.cost)
        {
            send_writer.send(SendBloonPackEvent {
                sender: Side::Bot,
                pack_index: idx,
            });
            return;
        }
    }

    // 7. Steady upgrade
    if let Some((entity, tower, _)) = bot_towers.iter().find(|(_, t, _)| t.path1 < 3 || t.path2 < 3) {
        let branch = if tower.path1 <= tower.path2 { 1 } else { 2 };
        upgrade_writer.send(UpgradeTowerEvent {
            side: Side::Bot,
            tower_entity: *entity,
            branch,
        });
    }
}
