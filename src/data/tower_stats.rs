use crate::core::types::TowerType;

use super::towers_cfg::tower_cfg;

#[path = "tower_stats_extra.rs"]
mod tower_stats_extra;

#[derive(Debug, Clone, Copy)]
pub struct VillageBuff {
    pub camo: bool,
    pub lead: bool,
    pub speed_mult: f32,
    pub range_mult: f32,
    pub pierce_bonus: u32,
    pub damage_bonus: i32,
}

impl Default for VillageBuff {
    fn default() -> Self {
        Self {
            camo: false,
            lead: false,
            speed_mult: 1.0,
            range_mult: 1.0,
            pierce_bonus: 0,
            damage_bonus: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EffectiveStats {
    pub range: f32,
    pub fire_rate: f32,
    pub damage: i32,
    pub pierce: u32,
    pub proj_speed: f32,
    pub splash_radius: f32,
    pub can_hit_camo: bool,
    pub can_pop_lead: bool,
    pub multi_shot: u8,
    pub knockback_chance: f32,
    pub stun_duration: f32,
    pub permafrost: bool,
    pub slow_factor: f32,
    pub slow_duration: f32,
    pub flash_every_4: bool,
    // Bloonchipper stats
    pub max_capacity: u32,
    pub barrels: usize,
    pub pull_speed: f32,
    pub shred_duration: f32,
    pub layers_per_shred: u8,
    pub can_suck_heavy: bool,
    pub can_suck_moab: bool,
    pub has_supa_vac: bool,
    // Village / Commander aura stats
    pub aura_camo: bool,
    pub aura_lead: bool,
    pub aura_speed_mult: f32,
    pub aura_range_mult: f32,
    pub aura_pierce_bonus: u32,
    pub aura_damage_bonus: i32,
    // Spike Factory stats
    pub spike_life: f32,
    pub ceramic_bonus_damage: i32,
    pub spike_persist_waves: bool,
    pub spike_explode: bool,
    pub has_spike_storm: bool,
    // Commander stats
    pub has_call_to_arms: bool,
    pub guard_count: u8,
    pub guard_elite: bool,
    // Military Base stats
    pub unit_hp: i32,
    pub unit_speed: f32,
    pub unit_tier: u8,
    pub has_airstrike: bool,
    // Crook Boss stats
    pub moab_bonus_damage: i32,
    pub is_golden_boss: bool,
    pub summon_interval: f32,
    pub summon_count: u8,
    pub crook_tier: u8,
    // Farm stats
    pub farm_cash: i32,
    pub farm_interval: f32,
    pub farm_eco_bonus: i32,
    pub farm_send_discount: f32,
    pub farm_dividend_cash: i32,
    pub farm_dividend_interval: f32,
    // Glue Gunner stats
    pub glue_soak: bool,
    pub glue_corrosive_dmg: i32,
    pub glue_corrosive_interval: f32,
    pub glue_ceramic_bonus: i32,
    pub glue_moab_capable: bool,
    pub has_glue_striker: bool,
    // Necromancer stats
    pub soul_capacity: u32,
    pub rift_interval: f32,
    pub ghost_camo: bool,
    pub ghost_lead: bool,
    pub ghost_speed_mult: f32,
    pub ghost_hp: i32,
    pub summon_ghost_knight: bool,
    // Saboteur stats
    pub saboteur_jam_duration: f32,
    pub saboteur_jam_targets: usize,
    pub saboteur_ability_cd: f32,
    pub saboteur_slow: f32,
    // Scrap Collector stats
    pub scrap_cash: i32,
    pub scrap_moab_cash: i32,
    pub scrap_max_speed_bonus: f32,
    pub scrap_heat_decay: f32,
    pub scrap_pierce_bonus: u32,
    // Mirage Generator stats
    pub mirage_interval: f32,
    pub mirage_count: u8,
    pub mirage_speed_mult: f32,
    pub mirage_force_zomg: bool,
    pub mirage_flash_radius: f32,
    pub mirage_flash_duration: f32,
}

pub fn compute_effective_stats(
    tower_type: TowerType,
    path1: u8,
    path2: u8,
) -> EffectiveStats {
    let base = tower_cfg(tower_type);
    let mut s = EffectiveStats {
        range: base.range,
        fire_rate: base.fire_rate,
        damage: base.damage,
        pierce: base.pierce,
        proj_speed: base.proj_speed,
        splash_radius: base.splash_radius,
        can_hit_camo: base.can_hit_camo,
        can_pop_lead: base.can_pop_lead,
        multi_shot: 1,
        knockback_chance: 0.0,
        stun_duration: 0.0,
        permafrost: false,
        slow_factor: base.slow_factor,
        slow_duration: base.slow_duration,
        flash_every_4: false,
        max_capacity: 3,
        barrels: 1,
        pull_speed: 260.0,
        shred_duration: 0.80,
        layers_per_shred: 1,
        can_suck_heavy: false,
        can_suck_moab: false,
        has_supa_vac: false,
        aura_camo: false,
        aura_lead: false,
        aura_speed_mult: if tower_type == TowerType::Commander { 1.15 } else { 1.12 },
        aura_range_mult: 1.0,
        aura_pierce_bonus: 0,
        aura_damage_bonus: 0,
        spike_life: 15.0,
        ceramic_bonus_damage: 0,
        spike_persist_waves: false,
        spike_explode: false,
        has_spike_storm: false,
        has_call_to_arms: false,
        guard_count: 0,
        guard_elite: false,
        unit_hp: 35,
        unit_speed: 95.0,
        unit_tier: 0,
        has_airstrike: false,
        moab_bonus_damage: 0,
        is_golden_boss: false,
        summon_interval: 18.9,
        summon_count: 1,
        crook_tier: 0,
        farm_cash: 18,
        farm_interval: 4.5,
        farm_eco_bonus: 0,
        farm_send_discount: 0.0,
        farm_dividend_cash: 0,
        farm_dividend_interval: 15.0,
        glue_soak: false,
        glue_corrosive_dmg: 0,
        glue_corrosive_interval: 0.0,
        glue_ceramic_bonus: 0,
        glue_moab_capable: false,
        has_glue_striker: false,
        soul_capacity: 30,
        rift_interval: 10.0,
        ghost_camo: false,
        ghost_lead: false,
        ghost_speed_mult: 1.0,
        ghost_hp: 2,
        summon_ghost_knight: false,
        saboteur_jam_duration: 5.0,
        saboteur_jam_targets: 1,
        saboteur_ability_cd: 25.0,
        saboteur_slow: 1.0,
        scrap_cash: 3,
        scrap_moab_cash: 25,
        scrap_max_speed_bonus: 0.15,
        scrap_heat_decay: 3.5,
        scrap_pierce_bonus: 0,
        mirage_interval: 16.0,
        mirage_count: 1,
        mirage_speed_mult: 1.0,
        mirage_force_zomg: false,
        mirage_flash_radius: 0.0,
        mirage_flash_duration: 0.0,
    };

    match tower_type {
        TowerType::DartMonkey => {
            if path1 >= 1 { s.range += 28.0; }
            if path1 >= 2 { s.range += 22.0; s.can_hit_camo = true; }
            if path1 >= 3 { s.pierce += 3; s.damage += 1; s.can_pop_lead = true; }
            if path2 >= 1 { s.pierce += 2; }
            if path2 >= 2 { s.fire_rate *= 1.4; }
            if path2 >= 3 { s.multi_shot = 3; }
        }
        TowerType::Cannon => {
            if path1 >= 1 { s.splash_radius += 18.0; s.damage += 1; }
            if path1 >= 2 { s.damage += 1; }
            if path1 >= 3 { s.splash_radius += 22.0; s.damage += 1; s.stun_duration = 0.65; }
            if path2 >= 1 { s.range += 28.0; }
            if path2 >= 2 { s.fire_rate *= 1.42; }
            if path2 >= 3 { s.fire_rate *= 1.35; s.splash_radius += 16.0; }
        }
        TowerType::IceTower => {
            if path1 >= 1 { s.permafrost = true; }
            if path1 >= 2 { s.damage += 1; }
            if path1 >= 3 { s.damage += 1; }
            if path2 >= 1 { s.range += 26.0; s.splash_radius += 26.0; }
            if path2 >= 2 { s.fire_rate *= 1.45; s.slow_factor = 0.35; s.slow_duration = 2.8; }
            if path2 >= 3 { s.can_hit_camo = true; s.can_pop_lead = true; }
        }
        TowerType::NinjaMonkey => {
            if path1 >= 1 { s.range += 24.0; s.fire_rate *= 1.2; }
            if path1 >= 2 { s.pierce += 2; }
            if path1 >= 3 { s.multi_shot = 2; }
            if path2 >= 1 { s.range += 16.0; }
            if path2 >= 2 { s.knockback_chance = 0.22; }
            if path2 >= 3 { s.flash_every_4 = true; }
        }
        TowerType::Bloonchipper => {
            if path1 >= 1 { s.pull_speed *= 1.5; }
            if path1 >= 2 { s.can_suck_heavy = true; s.can_pop_lead = true; }
            if path1 >= 3 { s.layers_per_shred = 2; }
            if path1 >= 4 { s.can_suck_moab = true; }
            if path2 >= 1 { s.range *= 1.25; }
            if path2 >= 2 { s.shred_duration *= 0.55; }
            if path2 >= 3 { s.max_capacity = 9; s.barrels = 3; }
            if path2 >= 4 { s.has_supa_vac = true; }
        }
        TowerType::Village => {
            if path1 >= 1 { s.range += 35.0; }
            if path1 >= 2 { s.aura_speed_mult = 1.22; }
            if path1 >= 3 { s.aura_range_mult += 0.15; s.aura_pierce_bonus = 1; }
            if path2 >= 1 { s.aura_range_mult += 0.08; s.range += 20.0; }
            if path2 >= 2 { s.aura_camo = true; }
            if path2 >= 3 { s.aura_lead = true; }
        }
        TowerType::SpikeFactory => {
            if path1 >= 1 { s.pierce = 10; }
            if path1 >= 2 { s.can_pop_lead = true; s.damage = 2; }
            if path1 >= 3 { s.pierce = 14; s.damage = 3; s.ceramic_bonus_damage = 3; }
            if path1 >= 4 { s.spike_explode = true; s.splash_radius = 68.0; }
            if path2 >= 1 { s.fire_rate *= 1.25; }
            if path2 >= 2 { s.fire_rate *= 1.35; }
            if path2 >= 3 { s.spike_life = 32.0; s.spike_persist_waves = true; }
            if path2 >= 4 { s.has_spike_storm = true; }
        }
        TowerType::Commander => {
            if path1 >= 1 { s.range += 30.0; s.aura_speed_mult = 1.20; }
            if path1 >= 2 { s.range += 20.0; s.aura_speed_mult = 1.28; }
            if path1 >= 3 { s.aura_speed_mult = 1.35; s.aura_lead = true; s.aura_damage_bonus = 1; }
            if path1 >= 4 { s.has_call_to_arms = true; }
            if path2 >= 1 { s.fire_rate *= 1.45; }
            if path2 >= 2 { s.can_hit_camo = true; s.pierce += 1; }
            if path2 >= 3 { s.guard_count = 2; }
            if path2 >= 4 { s.guard_elite = true; s.damage += 1; s.can_pop_lead = true; }
        }
        TowerType::MilitaryBase
        | TowerType::CrookBoss
        | TowerType::Farm
        | TowerType::GlueGunner
        | TowerType::SoulEater
        | TowerType::Necromancer
        | TowerType::Saboteur
        | TowerType::ScrapCollector
        | TowerType::MirageGenerator => {
            tower_stats_extra::apply_extra_stats(tower_type, path1, path2, &mut s);
        }
    }

    s
}

pub fn discounted_send_cost(base_cost: i32, discount: f32) -> i32 {
    let clamped = discount.clamp(0.0, 0.20);
    ((base_cost as f32) * (1.0 - clamped)).round() as i32
}

pub fn apply_village_buff(mut stats: EffectiveStats, buff: VillageBuff) -> EffectiveStats {
    if buff.camo {
        stats.can_hit_camo = true;
    }
    if buff.lead {
        stats.can_pop_lead = true;
    }
    stats.fire_rate *= buff.speed_mult;
    stats.shred_duration /= buff.speed_mult.max(1.0);
    stats.pull_speed *= buff.speed_mult;
    stats.range *= buff.range_mult;
    stats.splash_radius *= buff.range_mult;
    stats.pierce += buff.pierce_bonus;
    if stats.damage > 0 {
        stats.damage += buff.damage_bonus;
    }
    stats
}
