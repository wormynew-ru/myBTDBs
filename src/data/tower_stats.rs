use crate::core::types::TowerType;

use super::towers_cfg::tower_cfg;

#[derive(Debug, Clone, Copy)]
pub struct VillageBuff {
    pub camo: bool,
    pub lead: bool,
    pub speed_mult: f32,
    pub range_mult: f32,
    pub pierce_bonus: u32,
}

impl Default for VillageBuff {
    fn default() -> Self {
        Self {
            camo: false,
            lead: false,
            speed_mult: 1.0,
            range_mult: 1.0,
            pierce_bonus: 0,
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
    // Village aura stats
    pub aura_camo: bool,
    pub aura_lead: bool,
    pub aura_speed_mult: f32,
    pub aura_range_mult: f32,
    pub aura_pierce_bonus: u32,
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
        aura_speed_mult: 1.12,
        aura_range_mult: 1.0,
        aura_pierce_bonus: 0,
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
    }

    s
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
    stats.range *= buff.range_mult;
    stats.splash_radius *= buff.range_mult;
    stats.pierce += buff.pierce_bonus;
    stats
}
