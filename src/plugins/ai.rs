#[path = "ai_spots.rs"]
mod ai_spots;

use bevy::prelude::*;

use crate::core::events::{
    ActivateAbilityEvent, BuildTowerEvent, SendBloonPackEvent, UpgradeTowerEvent,
};
use crate::core::state::{Difficulty, GameState, MatchSettings};
use crate::core::types::{Position, Side, SidesResource, TowerType};
use crate::data::bloons_cfg::SEND_PACKS;
use crate::data::maps_cfg::waypoints_for_side;
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats};
use crate::data::towers_cfg::tower_cfg;
use crate::plugins::bloons::Bloon;
use crate::plugins::towers::{ChipperState, Tower, TowerBuff};
use crate::utils::path_total_length;

use ai_spots::{find_bot_spot, find_commander_spot};

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

fn pick_upgrade_branch(t: &Tower, need_camo: bool, need_lead: bool) -> u8 {
    let cfg = tower_cfg(t.tower_type);
    if t.tower_type == TowerType::CrookBoss {
        if need_camo && t.path2 == 0 {
            return 2;
        }
        if need_lead && t.path1 < 3 {
            return 1;
        }
    }
    if (t.path1 as usize) < cfg.path1.len() && t.path1 <= t.path2 {
        1
    } else {
        2
    }
}

#[allow(clippy::too_many_arguments)]
fn utility_ai_system(
    time: Res<Time>,
    settings: Res<MatchSettings>,
    sides: Res<SidesResource>,
    mut ai: ResMut<AiState>,
    towers: Query<(Entity, &Side, &Tower, &TowerBuff, &Position, Option<&ChipperState>)>,
    bloons: Query<(&Side, &Bloon)>,
    mut build_writer: EventWriter<BuildTowerEvent>,
    mut upgrade_writer: EventWriter<UpgradeTowerEvent>,
    mut send_writer: EventWriter<SendBloonPackEvent>,
    mut ability_writer: EventWriter<ActivateAbilityEvent>,
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

    for (entity, side, t, buff, pos, chipper) in &towers {
        let stats = apply_village_buff(
            compute_effective_stats(t.tower_type, t.path1, t.path2),
            buff.0,
        );
        match side {
            Side::Bot => {
                bot_towers.push((entity, t, stats, chipper));
                bot_positions.push(pos.0);
            }
            Side::Player => {
                player_has_camo |= stats.can_hit_camo;
                player_has_lead |= stats.can_pop_lead;
            }
        }
    }

    let mut max_threat = 0.0_f32;
    let mut has_camo_threat = false;
    let mut has_lead_threat = false;
    let mut bot_bloon_count = 0_usize;
    for (side, bloon) in &bloons {
        if *side == Side::Bot {
            bot_bloon_count += 1;
            max_threat = max_threat.max(bloon.path_progress / total_len);
            has_camo_threat |= bloon.camo;
            has_lead_threat |= bloon.lead;
        }
    }

    if max_threat > 0.55 {
        for (entity, t, _, chipper) in &bot_towers {
            if let Some(c) = chipper {
                if t.path2 >= 4 && c.supa_vac_cooldown <= 0.0 {
                    ability_writer.send(ActivateAbilityEvent {
                        side: Side::Bot,
                        tower_entity: *entity,
                    });
                    break;
                }
            }
        }
    }

    let bot_has_camo = bot_towers.iter().any(|(_, _, s, _)| s.can_hit_camo);
    let bot_has_lead = bot_towers.iter().any(|(_, _, s, _)| s.can_pop_lead);
    let bot_has_spike = bot_towers.iter().any(|(_, t, _, _)| t.tower_type == TowerType::SpikeFactory);
    let bot_has_cmdr = bot_towers.iter().any(|(_, t, _, _)| t.tower_type == TowerType::Commander);
    let bot_has_base = bot_towers.iter().any(|(_, t, _, _)| t.tower_type == TowerType::MilitaryBase);
    let bot_has_crook = bot_towers.iter().any(|(_, t, _, _)| t.tower_type == TowerType::CrookBoss);

    // 1. Starter defense
    if bot_towers.is_empty() && bot_cash >= tower_cfg(TowerType::DartMonkey).cost {
        if let Some(pos) = find_bot_spot(settings.map_id, &bot_positions, false) {
            build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: TowerType::DartMonkey, pos });
            return;
        }
    }

    // 2. Counter Camo (Crook Boss 0-1, Spike Factory, or Ninja)
    if (has_camo_threat || (settings.difficulty != Difficulty::Easy && sides.wave >= 4)) && !bot_has_camo {
        if let Some((e, _, _, _)) = bot_towers
            .iter()
            .find(|(_, t, _, _)| t.tower_type == TowerType::CrookBoss && t.path2 == 0)
        {
            if bot_cash >= tower_cfg(TowerType::CrookBoss).path2[0].cost {
                upgrade_writer.send(UpgradeTowerEvent { side: Side::Bot, tower_entity: *e, branch: 2 });
                return;
            }
        }
        if !bot_has_crook
            && bot_cash >= tower_cfg(TowerType::CrookBoss).cost + tower_cfg(TowerType::CrookBoss).path2[0].cost
        {
            if let Some(pos) = find_bot_spot(settings.map_id, &bot_positions, false) {
                build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: TowerType::CrookBoss, pos });
                return;
            }
        }
        if !bot_has_spike && bot_cash >= tower_cfg(TowerType::SpikeFactory).cost {
            if let Some(pos) = find_bot_spot(settings.map_id, &bot_positions, true) {
                build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: TowerType::SpikeFactory, pos });
                return;
            }
        }
        if bot_cash >= tower_cfg(TowerType::NinjaMonkey).cost {
            if let Some(pos) = find_bot_spot(settings.map_id, &bot_positions, false) {
                build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: TowerType::NinjaMonkey, pos });
                return;
            }
        }
        if has_camo_threat { return; }
    }
    if !bot_has_spike && max_threat > 0.58 && bot_cash >= tower_cfg(TowerType::SpikeFactory).cost {
        if let Some(pos) = find_bot_spot(settings.map_id, &bot_positions, true) {
            build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: TowerType::SpikeFactory, pos });
            return;
        }
    }

    // 3. Counter Lead
    if (has_lead_threat || (settings.difficulty != Difficulty::Easy && sides.wave >= 5)) && !bot_has_lead {
        if bot_cash >= tower_cfg(TowerType::Cannon).cost {
            if let Some(pos) = find_bot_spot(settings.map_id, &bot_positions, false) {
                build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: TowerType::Cannon, pos });
                return;
            }
        }
        if has_lead_threat { return; }
    }

    // 4. Versatile Crook Boss (against groups / camo), Commander, MilitaryBase
    if !bot_has_crook
        && (bot_bloon_count >= 8 || bot_towers.len() >= 1)
        && bot_cash >= tower_cfg(TowerType::CrookBoss).cost
    {
        if let Some(pos) = find_bot_spot(settings.map_id, &bot_positions, false) {
            build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: TowerType::CrookBoss, pos });
            return;
        }
    }
    if !bot_has_cmdr && bot_towers.len() >= 2 && bot_cash >= tower_cfg(TowerType::Commander).cost {
        if let Some(pos) = find_commander_spot(settings.map_id, &bot_positions) {
            build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: TowerType::Commander, pos });
            return;
        }
    }
    if !bot_has_base && bot_towers.len() >= 3 && bot_cash >= tower_cfg(TowerType::MilitaryBase).cost {
        if let Some(pos) = find_bot_spot(settings.map_id, &bot_positions, false) {
            build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: TowerType::MilitaryBase, pos });
            return;
        }
    }

    let can_upgrade = |t: &Tower| {
        let cfg = tower_cfg(t.tower_type);
        (t.path1 as usize) < cfg.path1.len() || (t.path2 as usize) < cfg.path2.len()
    };

    if max_threat > 0.52 || bot_towers.len() < 2 {
        if let Some((entity, tower, _, _)) = bot_towers.iter().find(|(_, t, _, _)| can_upgrade(t)) {
            let branch = pick_upgrade_branch(tower, !bot_has_camo, has_lead_threat || !bot_has_lead);
            upgrade_writer.send(UpgradeTowerEvent { side: Side::Bot, tower_entity: *entity, branch });
            return;
        }
        let pick = match bot_towers.len() % 10 {
            1 => TowerType::CrookBoss,
            2 => TowerType::Cannon,
            3 => TowerType::SpikeFactory,
            4 => TowerType::Commander,
            5 => TowerType::MilitaryBase,
            6 => TowerType::Bloonchipper,
            7 => TowerType::NinjaMonkey,
            8 => TowerType::IceTower,
            9 => TowerType::Village,
            _ => TowerType::DartMonkey,
        };
        let spot = if pick == TowerType::Commander {
            find_commander_spot(settings.map_id, &bot_positions)
                .or_else(|| find_bot_spot(settings.map_id, &bot_positions, false))
        } else {
            find_bot_spot(settings.map_id, &bot_positions, pick == TowerType::SpikeFactory)
        };
        if let Some(pos) = spot {
            if bot_cash >= tower_cfg(pick).cost {
                build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: pick, pos });
                return;
            }
        }
    }

    // 5. Rush Player when weak against Camo/Lead or late game
    if settings.difficulty != Difficulty::Easy && sides.wave >= 4 && max_threat < 0.45 {
        if !player_has_camo && bot_cash >= SEND_PACKS[4].cost {
            send_writer.send(SendBloonPackEvent { sender: Side::Bot, pack_index: 4 });
            return;
        }
        if sides.wave >= 5 && !player_has_lead && bot_cash >= SEND_PACKS[5].cost {
            send_writer.send(SendBloonPackEvent { sender: Side::Bot, pack_index: 5 });
            return;
        }
        if settings.difficulty == Difficulty::Hard && sides.wave >= 11 && bot_cash >= SEND_PACKS[10].cost {
            send_writer.send(SendBloonPackEvent { sender: Side::Bot, pack_index: 10 });
            return;
        }
        if settings.difficulty == Difficulty::Hard && sides.wave >= 8 && bot_cash >= SEND_PACKS[7].cost {
            send_writer.send(SendBloonPackEvent { sender: Side::Bot, pack_index: 7 });
            return;
        }
    }

    // 6. Calm phase Eco scaling (only positive-eco packs)
    if max_threat < 0.42 {
        if let Some((idx, _)) = SEND_PACKS
            .iter().enumerate().rev()
            .find(|(_, p)| sides.wave >= p.unlock_wave && p.eco_gain > 0 && bot_cash >= p.cost)
        {
            send_writer.send(SendBloonPackEvent { sender: Side::Bot, pack_index: idx });
            return;
        }
    }

    // 7. Steady upgrade
    if let Some((entity, tower, _, _)) = bot_towers.iter().find(|(_, t, _, _)| can_upgrade(t)) {
        let branch = pick_upgrade_branch(tower, !bot_has_camo, has_lead_threat || !bot_has_lead);
        upgrade_writer.send(UpgradeTowerEvent { side: Side::Bot, tower_entity: *entity, branch });
    }
}
