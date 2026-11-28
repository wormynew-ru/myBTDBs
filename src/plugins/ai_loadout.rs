use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;

use crate::core::state::{Difficulty, MatchSettings};
use crate::core::types::TowerType;
use crate::data::bloons_cfg::SEND_PACKS;
use crate::data::tower_stats::discounted_send_cost;
use crate::data::towers_cfg::{tower_cfg, ALL_TOWERS};
use crate::plugins::towers::Tower;

fn next_rand(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    *state
}

pub fn generate_blind_bot_loadout(size: usize, extra_seed: u32) -> Vec<TowerType> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() ^ (d.as_secs() as u32))
        .unwrap_or(0xA5A5_5A5A);
    let mut rng = (nanos ^ extra_seed.rotate_left(13)) | 1;

    let mut deck: Vec<TowerType> = Vec::with_capacity(size);
    let push_unique = |deck: &mut Vec<TowerType>, t: TowerType| {
        if deck.len() < size && !deck.contains(&t) {
            deck.push(t);
        }
    };

    // 1. Reliable early defender
    let starters = [
        TowerType::DartMonkey,
        TowerType::NinjaMonkey,
        TowerType::CrookBoss,
        TowerType::GlueGunner,
        TowerType::Necromancer,
    ];
    push_unique(&mut deck, starters[(next_rand(&mut rng) as usize) % starters.len()]);

    // 2. Camo / Lead coverage
    let counters = [
        TowerType::SpikeFactory,
        TowerType::CrookBoss,
        TowerType::NinjaMonkey,
        TowerType::Cannon,
        TowerType::Necromancer,
        TowerType::MilitaryBase,
        TowerType::Saboteur,
    ];
    for _ in 0..8 {
        let pick = counters[(next_rand(&mut rng) as usize) % counters.len()];
        if !deck.contains(&pick) {
            push_unique(&mut deck, pick);
            break;
        }
    }

    // 3. Support / Economy / Tactical / Evolution
    let flex = [
        TowerType::Farm,
        TowerType::Commander,
        TowerType::Village,
        TowerType::SoulEater,
        TowerType::GlueGunner,
        TowerType::ScrapCollector,
        TowerType::MirageGenerator,
        TowerType::Saboteur,
    ];
    for _ in 0..8 {
        let pick = flex[(next_rand(&mut rng) as usize) % flex.len()];
        if !deck.contains(&pick) {
            push_unique(&mut deck, pick);
            break;
        }
    }

    // 4. Fill remaining slots randomly from ALL_TOWERS
    let mut pool: Vec<TowerType> = ALL_TOWERS
        .iter()
        .copied()
        .filter(|t| !deck.contains(t))
        .collect();
    while deck.len() < size && !pool.is_empty() {
        let idx = (next_rand(&mut rng) as usize) % pool.len();
        deck.push(pool.swap_remove(idx));
    }

    deck
}

pub fn pick_starter_from_loadout(loadout: &[TowerType]) -> Option<TowerType> {
    loadout
        .iter()
        .copied()
        .filter(|&t| {
            !matches!(
                t,
                TowerType::Farm
                    | TowerType::Village
                    | TowerType::SoulEater
                    | TowerType::ScrapCollector
                    | TowerType::MirageGenerator
            )
        })
        .min_by_key(|&t| tower_cfg(t).cost)
}

pub fn pick_camo_from_loadout(loadout: &[TowerType]) -> Option<TowerType> {
    let priority = [
        TowerType::CrookBoss,
        TowerType::SpikeFactory,
        TowerType::NinjaMonkey,
        TowerType::Necromancer,
        TowerType::Saboteur,
        TowerType::MilitaryBase,
        TowerType::DartMonkey,
        TowerType::Commander,
        TowerType::IceTower,
        TowerType::Village,
    ];
    priority.into_iter().find(|t| loadout.contains(t))
}

pub fn pick_lead_from_loadout(loadout: &[TowerType]) -> Option<TowerType> {
    let priority = [
        TowerType::Cannon,
        TowerType::Necromancer,
        TowerType::SpikeFactory,
        TowerType::Bloonchipper,
        TowerType::GlueGunner,
        TowerType::Saboteur,
        TowerType::CrookBoss,
        TowerType::MilitaryBase,
        TowerType::Commander,
        TowerType::DartMonkey,
        TowerType::IceTower,
    ];
    priority.into_iter().find(|t| loadout.contains(t))
}

pub fn pick_upgrade_branch(t: &Tower, need_camo: bool, need_lead: bool) -> u8 {
    let cfg = tower_cfg(t.tower_type);
    match t.tower_type {
        TowerType::CrookBoss => {
            if need_camo && t.path2 == 0 {
                return 2;
            }
            if need_lead && t.path1 < 3 {
                return 1;
            }
        }
        TowerType::Necromancer => {
            if need_camo && t.path2 == 0 {
                return 2;
            }
        }
        TowerType::DartMonkey => {
            if need_camo && t.path1 < 2 {
                return 1;
            }
            if need_lead && t.path1 < 3 {
                return 1;
            }
        }
        TowerType::GlueGunner | TowerType::Saboteur => {
            if need_lead && t.path1 < 2 {
                return 1;
            }
        }
        _ => {}
    }
    if (t.path1 as usize) < cfg.path1.len() && t.path1 <= t.path2 {
        1
    } else if (t.path2 as usize) < cfg.path2.len() {
        2
    } else {
        1
    }
}

pub fn pick_general_from_loadout(
    settings: &MatchSettings,
    built_types: &[TowerType],
) -> Option<TowerType> {
    let total = built_types.len();
    let count_of = |tt: TowerType| built_types.iter().filter(|&&x| x == tt).count();
    let farms = count_of(TowerType::Farm);
    let villages = count_of(TowerType::Village);
    let soul_eaters = count_of(TowerType::SoulEater);
    let scraps = count_of(TowerType::ScrapCollector);
    let saboteurs = count_of(TowerType::Saboteur);
    let mirages = count_of(TowerType::MirageGenerator);

    let candidates: Vec<TowerType> = settings
        .bot_loadout
        .iter()
        .copied()
        .filter(|&t| match t {
            TowerType::Farm => farms < 2,
            TowerType::Village => villages < 1 && total >= 2,
            TowerType::SoulEater => soul_eaters < 2 && total >= 2,
            TowerType::ScrapCollector => scraps < 1 && total >= 2,
            TowerType::Saboteur => saboteurs < 2 && total >= 1,
            TowerType::MirageGenerator => mirages < 2 && total >= 2,
            _ => true,
        })
        .collect();
    if candidates.is_empty() {
        settings.bot_loadout.first().copied()
    } else {
        Some(candidates[total % candidates.len()])
    }
}

pub fn pick_rush_pack_index(
    difficulty: Difficulty,
    wave: u32,
    bot_cash: i32,
    bot_discount: f32,
    player_has_camo: bool,
    player_has_lead: bool,
) -> Option<usize> {
    if difficulty == Difficulty::Easy || wave < 4 {
        return None;
    }
    if !player_has_camo && bot_cash >= discounted_send_cost(SEND_PACKS[4].cost, bot_discount) {
        return Some(4);
    }
    if wave >= 5 && !player_has_lead && bot_cash >= discounted_send_cost(SEND_PACKS[5].cost, bot_discount) {
        return Some(5);
    }
    if difficulty == Difficulty::Hard && wave >= 11 && bot_cash >= discounted_send_cost(SEND_PACKS[10].cost, bot_discount) {
        return Some(10);
    }
    if difficulty == Difficulty::Hard && wave >= 8 && bot_cash >= discounted_send_cost(SEND_PACKS[7].cost, bot_discount) {
        return Some(7);
    }
    None
}

pub fn pick_eco_pack_index(wave: u32, bot_cash: i32, bot_discount: f32) -> Option<usize> {
    SEND_PACKS
        .iter()
        .enumerate()
        .rev()
        .find(|(_, p)| {
            wave >= p.unlock_wave
                && p.eco_gain > 0
                && bot_cash >= discounted_send_cost(p.cost, bot_discount)
        })
        .map(|(idx, _)| idx)
}
