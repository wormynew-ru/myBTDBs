#[path = "ai_loadout.rs"]
mod ai_loadout;
#[path = "ai_spots.rs"]
mod ai_spots;

use bevy::prelude::*;

use crate::core::events::{
    ActivateAbilityEvent, BuildTowerEvent, ChangePriorityEvent, SendBloonPackEvent,
    UpgradeTowerEvent,
};
use crate::core::state::{Difficulty, GameState, MatchSettings};
use crate::core::types::{Position, Side, SidesResource, TargetPriority, TowerType};
use crate::data::maps_cfg::waypoints_for_side;
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats};
use crate::data::towers_cfg::tower_cfg;
use crate::plugins::bloons::Bloon;
use crate::plugins::towers::{ChipperState, GlueState, SaboteurState, Tower, TowerBuff};
use crate::utils::path_total_length;

use ai_loadout::{
    generate_blind_bot_loadout, pick_camo_from_loadout, pick_eco_pack_index,
    pick_general_from_loadout, pick_lead_from_loadout, pick_rush_pack_index,
    pick_starter_from_loadout, pick_upgrade_branch,
};
use ai_spots::{find_bot_spot, find_commander_spot, find_farm_spot, pick_spot_for_tower};

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AiState>()
            .add_systems(OnEnter(GameState::Playing), reset_ai_match)
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

fn reset_ai_match(
    time: Res<Time>,
    mut ai: ResMut<AiState>,
    mut settings: ResMut<MatchSettings>,
) {
    ai.timer = settings.difficulty.ai_interval();
    let size = settings.game_mode.loadout_size();
    settings.bot_loadout = generate_blind_bot_loadout(size, time.elapsed_secs().to_bits());
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn utility_ai_system(
    time: Res<Time>,
    settings: Res<MatchSettings>,
    sides: Res<SidesResource>,
    mut ai: ResMut<AiState>,
    towers: Query<(
        Entity,
        &Side,
        &Tower,
        &TowerBuff,
        &Position,
        Option<&ChipperState>,
        Option<&GlueState>,
        Option<&SaboteurState>,
    )>,
    bloons: Query<(&Side, &Bloon)>,
    mut build_writer: EventWriter<BuildTowerEvent>,
    mut upgrade_writer: EventWriter<UpgradeTowerEvent>,
    mut send_writer: EventWriter<SendBloonPackEvent>,
    mut ability_writer: EventWriter<ActivateAbilityEvent>,
    mut prio_writer: EventWriter<ChangePriorityEvent>,
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
    let loadout = &settings.bot_loadout;

    let mut bot_towers = Vec::new();
    let mut bot_positions = Vec::new();
    let mut built_types = Vec::new();
    let mut player_has_camo = false;
    let mut player_has_lead = false;
    let mut player_tower_count = 0_usize;

    for (entity, side, t, buff, pos, chipper, glue, sab) in &towers {
        let stats = apply_village_buff(
            compute_effective_stats(t.tower_type, t.path1, t.path2),
            buff.0,
        );
        match side {
            Side::Bot => {
                bot_towers.push((entity, t, stats, chipper, glue, sab));
                bot_positions.push(pos.0);
                built_types.push(t.tower_type);
            }
            Side::Player => {
                player_tower_count += 1;
                player_has_camo |= stats.can_hit_camo;
                player_has_lead |= stats.can_pop_lead;
            }
        }
    }

    let mut max_threat = 0.0_f32;
    let mut has_camo_threat = false;
    let mut has_lead_threat = false;
    let mut bot_bloon_count = 0_usize;
    let mut bot_sent_bloons = 0_usize;
    for (side, bloon) in &bloons {
        if *side == Side::Bot {
            bot_bloon_count += 1;
            if bloon.from_send {
                bot_sent_bloons += 1;
            }
            max_threat = max_threat.max(bloon.path_progress / total_len);
            has_camo_threat |= bloon.camo;
            has_lead_threat |= bloon.lead;
        }
    }

    if player_tower_count > 0 {
        for (entity, _, _, _, _, sab) in &bot_towers {
            if let Some(s) = sab {
                if s.ability_cooldown <= 0.0 {
                    ability_writer.send(ActivateAbilityEvent { side: Side::Bot, tower_entity: *entity });
                    break;
                }
            }
        }
    }

    if max_threat > 0.55 || bot_bloon_count >= 12 {
        for (entity, t, _, chipper, glue, _) in &bot_towers {
            if let Some(c) = chipper {
                if t.path2 >= 4 && c.supa_vac_cooldown <= 0.0 {
                    ability_writer.send(ActivateAbilityEvent { side: Side::Bot, tower_entity: *entity });
                    break;
                }
            }
            if let Some(g) = glue {
                if t.path2 >= 4 && g.striker_cooldown <= 0.0 {
                    ability_writer.send(ActivateAbilityEvent { side: Side::Bot, tower_entity: *entity });
                    break;
                }
            }
        }
    }

    if sides.wave >= 8 {
        for (entity, t, _, _, _, _) in &bot_towers {
            if t.priority == TargetPriority::First
                && matches!(
                    t.tower_type,
                    TowerType::Cannon | TowerType::CrookBoss | TowerType::Necromancer | TowerType::Bloonchipper
                )
            {
                prio_writer.send(ChangePriorityEvent {
                    tower_entity: *entity,
                    priority: TargetPriority::Strong,
                });
                break;
            }
        }
    }

    let bot_has_camo = bot_towers.iter().any(|(_, _, s, _, _, _)| s.can_hit_camo);
    let bot_has_lead = bot_towers.iter().any(|(_, _, s, _, _, _)| s.can_pop_lead);
    let bot_farms = built_types.iter().filter(|&&t| t == TowerType::Farm).count();
    let bot_scraps = built_types.iter().filter(|&&t| t == TowerType::ScrapCollector).count();
    let bot_discount: f32 = bot_towers
        .iter()
        .filter(|(_, t, _, _, _, _)| t.tower_type == TowerType::Farm && t.path2 >= 2)
        .map(|(_, _, s, _, _, _)| s.farm_send_discount)
        .sum::<f32>()
        .min(0.20);

    // 1. Starter defense from bot's blind loadout
    if bot_towers.is_empty() {
        if let Some(starter) = pick_starter_from_loadout(loadout) {
            if bot_cash >= tower_cfg(starter).cost {
                if let Some(pos) = find_bot_spot(settings.map_id, &bot_positions, starter == TowerType::SpikeFactory) {
                    build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: starter, pos });
                    return;
                }
            }
        }
    }

    // 2. Counter Camo using bot's loadout
    if (has_camo_threat || (settings.difficulty != Difficulty::Easy && sides.wave >= 4)) && !bot_has_camo {
        if let Some((e, t, _, _, _, _)) = bot_towers.iter().find(|(_, t, _, _, _, _)| {
            t.jammed_timer <= 0.0
                && (matches!(t.tower_type, TowerType::CrookBoss | TowerType::Necromancer) && t.path2 == 0
                    || (t.tower_type == TowerType::DartMonkey && t.path1 < 2))
        }) {
            let branch = if t.tower_type == TowerType::DartMonkey { 1 } else { 2 };
            let cfg = tower_cfg(t.tower_type);
            let tier = if branch == 1 { t.path1 } else { t.path2 } as usize;
            let b_cfgs = if branch == 1 { cfg.path1 } else { cfg.path2 };
            if tier < b_cfgs.len() && bot_cash >= b_cfgs[tier].cost {
                upgrade_writer.send(UpgradeTowerEvent { side: Side::Bot, tower_entity: *e, branch });
                return;
            }
        }
        if let Some(camo_t) = pick_camo_from_loadout(loadout) {
            if bot_cash >= tower_cfg(camo_t).cost {
                let near_exit = camo_t == TowerType::SpikeFactory;
                if let Some(pos) = find_bot_spot(settings.map_id, &bot_positions, near_exit) {
                    build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: camo_t, pos });
                    return;
                }
            }
        }
        if has_camo_threat { return; }
    }

    // 3. Counter Lead using bot's loadout
    if (has_lead_threat || (settings.difficulty != Difficulty::Easy && sides.wave >= 5)) && !bot_has_lead {
        if let Some((e, t, _, _, _, _)) = bot_towers.iter().find(|(_, t, _, _, _, _)| {
            t.jammed_timer <= 0.0
                && matches!(
                    t.tower_type,
                    TowerType::GlueGunner | TowerType::SpikeFactory | TowerType::Bloonchipper | TowerType::Saboteur
                )
                && t.path1 < 2
        }) {
            let cfg = tower_cfg(t.tower_type);
            let tier = t.path1 as usize;
            if tier < cfg.path1.len() && bot_cash >= cfg.path1[tier].cost {
                upgrade_writer.send(UpgradeTowerEvent { side: Side::Bot, tower_entity: *e, branch: 1 });
                return;
            }
        }
        if let Some(lead_t) = pick_lead_from_loadout(loadout) {
            if bot_cash >= tower_cfg(lead_t).cost {
                if let Some(pos) = find_bot_spot(settings.map_id, &bot_positions, lead_t == TowerType::SpikeFactory) {
                    build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: lead_t, pos });
                    return;
                }
            }
        }
        if has_lead_threat { return; }
    }

    // 3.5 Counter send spam with Scrap Collector
    if bot_sent_bloons >= 4
        && bot_scraps == 0
        && loadout.contains(&TowerType::ScrapCollector)
        && bot_cash >= tower_cfg(TowerType::ScrapCollector).cost
    {
        if let Some(pos) = find_commander_spot(settings.map_id, &bot_positions)
            .or_else(|| find_bot_spot(settings.map_id, &bot_positions, false))
        {
            build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: TowerType::ScrapCollector, pos });
            return;
        }
    }

    // 4. Farm investment (if Farm is in bot's loadout)
    let target_farms = if settings.difficulty == Difficulty::Easy { 1 } else { 2 };
    if loadout.contains(&TowerType::Farm)
        && sides.wave <= 14
        && bot_farms < target_farms
        && max_threat < 0.45
        && !bot_towers.is_empty()
        && bot_cash >= tower_cfg(TowerType::Farm).cost
    {
        if let Some(pos) = find_farm_spot(settings.map_id, &bot_positions) {
            build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: TowerType::Farm, pos });
            return;
        }
    }

    let can_upgrade = |t: &Tower| {
        let cfg = tower_cfg(t.tower_type);
        t.jammed_timer <= 0.0
            && ((t.path1 as usize) < cfg.path1.len() || (t.path2 as usize) < cfg.path2.len())
    };

    // 5. Threat response: upgrade or build from bot's loadout
    if max_threat > 0.50 || bot_towers.len() < 2 {
        if let Some((entity, tower, _, _, _, _)) = bot_towers
            .iter()
            .find(|(_, t, _, _, _, _)| t.tower_type != TowerType::Farm && can_upgrade(t))
        {
            let branch = pick_upgrade_branch(tower, !bot_has_camo, has_lead_threat || !bot_has_lead);
            upgrade_writer.send(UpgradeTowerEvent { side: Side::Bot, tower_entity: *entity, branch });
            return;
        }
        if let Some(pick) = pick_general_from_loadout(&settings, &built_types) {
            if let Some(pos) = pick_spot_for_tower(settings.map_id, &bot_positions, pick) {
                if bot_cash >= tower_cfg(pick).cost {
                    build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: pick, pos });
                    return;
                }
            }
        }
    }

    // 6. Rush Player when weak against Camo/Lead or late game
    if max_threat < 0.45 {
        if let Some(pack_index) = pick_rush_pack_index(
            settings.difficulty,
            sides.wave,
            bot_cash,
            bot_discount,
            player_has_camo,
            player_has_lead,
        ) {
            send_writer.send(SendBloonPackEvent { sender: Side::Bot, pack_index });
            return;
        }
    }

    // 7. Calm phase Eco scaling (only positive-eco packs)
    if max_threat < 0.42 {
        if let Some(idx) = pick_eco_pack_index(sides.wave, bot_cash, bot_discount) {
            send_writer.send(SendBloonPackEvent { sender: Side::Bot, pack_index: idx });
            return;
        }
    }

    // 8. Steady upgrade or expand
    if let Some((entity, tower, _, _, _, _)) = bot_towers.iter().find(|(_, t, _, _, _, _)| can_upgrade(t)) {
        let branch = pick_upgrade_branch(tower, !bot_has_camo, has_lead_threat || !bot_has_lead);
        upgrade_writer.send(UpgradeTowerEvent { side: Side::Bot, tower_entity: *entity, branch });
    } else if let Some(pick) = pick_general_from_loadout(&settings, &built_types) {
        if bot_cash >= tower_cfg(pick).cost {
            if let Some(pos) = pick_spot_for_tower(settings.map_id, &bot_positions, pick) {
                build_writer.send(BuildTowerEvent { side: Side::Bot, tower_type: pick, pos });
            }
        }
    }
}
