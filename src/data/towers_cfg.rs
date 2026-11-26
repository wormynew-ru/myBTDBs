use bevy::prelude::*;

use crate::core::types::TowerType;

pub const STARTING_LIVES: i32 = 150;
pub const STARTING_CASH: i32 = 650;
pub const STARTING_ECO: i32 = 250;
pub const ECO_INTERVAL_SECS: f32 = 6.0;
pub const SELL_REFUND_RATIO: f32 = 0.7;
pub const TOWER_RADIUS: f32 = 18.0;

#[derive(Debug, Clone, Copy)]
pub struct UpgradeCfg {
    pub name: &'static str,
    pub cost: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct TowerCfg {
    pub tower_type: TowerType,
    pub name: &'static str,
    pub short_name: &'static str,
    pub cost: i32,
    pub range: f32,
    pub fire_rate: f32,
    pub damage: i32,
    pub pierce: u32,
    pub proj_speed: f32,
    pub splash_radius: f32,
    pub can_hit_camo: bool,
    pub can_pop_lead: bool,
    pub slow_factor: f32,
    pub slow_duration: f32,
    pub color: Color,
    pub path1: [UpgradeCfg; 3],
    pub path2: [UpgradeCfg; 3],
}

pub const ALL_TOWERS: [TowerType; 4] = [
    TowerType::DartMonkey,
    TowerType::Cannon,
    TowerType::IceTower,
    TowerType::NinjaMonkey,
];

pub fn tower_cfg(tower_type: TowerType) -> TowerCfg {
    match tower_type {
        TowerType::DartMonkey => TowerCfg {
            tower_type,
            name: "Dart Monkey",
            short_name: "Dart",
            cost: 200,
            range: 112.0,
            fire_rate: 1.2,
            damage: 1,
            pierce: 2,
            proj_speed: 460.0,
            splash_radius: 0.0,
            can_hit_camo: false,
            can_pop_lead: false,
            slow_factor: 1.0,
            slow_duration: 0.0,
            color: Color::srgb(0.70, 0.35, 0.08),
            path1: [
                UpgradeCfg { name: "Long Range", cost: 90 },
                UpgradeCfg { name: "Camo Eyes", cost: 135 },
                UpgradeCfg { name: "Spike-O-Pult", cost: 360 },
            ],
            path2: [
                UpgradeCfg { name: "Sharp Shots", cost: 110 },
                UpgradeCfg { name: "Quick Hands", cost: 165 },
                UpgradeCfg { name: "Triple Darts", cost: 390 },
            ],
        },
        TowerType::Cannon => TowerCfg {
            tower_type,
            name: "Cannon",
            short_name: "Cannon",
            cost: 450,
            range: 126.0,
            fire_rate: 0.72,
            damage: 1,
            pierce: 1,
            proj_speed: 340.0,
            splash_radius: 54.0,
            can_hit_camo: false,
            can_pop_lead: true,
            slow_factor: 1.0,
            slow_duration: 0.0,
            color: Color::srgb(0.24, 0.28, 0.36),
            path1: [
                UpgradeCfg { name: "Bigger Bombs", cost: 180 },
                UpgradeCfg { name: "Heavy Shells", cost: 320 },
                UpgradeCfg { name: "Bloon Impact", cost: 540 },
            ],
            path2: [
                UpgradeCfg { name: "Extra Range", cost: 130 },
                UpgradeCfg { name: "Fast Reload", cost: 240 },
                UpgradeCfg { name: "Cluster Bomb", cost: 490 },
            ],
        },
        TowerType::IceTower => TowerCfg {
            tower_type,
            name: "Ice Tower",
            short_name: "Ice",
            cost: 350,
            range: 92.0,
            fire_rate: 0.68,
            damage: 0,
            pierce: 999,
            proj_speed: 0.0,
            splash_radius: 92.0,
            can_hit_camo: false,
            can_pop_lead: false,
            slow_factor: 0.45,
            slow_duration: 2.2,
            color: Color::srgb(0.05, 0.55, 0.85),
            path1: [
                UpgradeCfg { name: "Permafrost", cost: 140 },
                UpgradeCfg { name: "Snap Freeze", cost: 260 },
                UpgradeCfg { name: "Ice Shards", cost: 480 },
            ],
            path2: [
                UpgradeCfg { name: "Wide Pulse", cost: 120 },
                UpgradeCfg { name: "Deep Freeze", cost: 210 },
                UpgradeCfg { name: "Arctic Radar", cost: 380 },
            ],
        },
        TowerType::NinjaMonkey => TowerCfg {
            tower_type,
            name: "Ninja Monkey",
            short_name: "Ninja",
            cost: 500,
            range: 130.0,
            fire_rate: 1.48,
            damage: 1,
            pierce: 3,
            proj_speed: 520.0,
            splash_radius: 0.0,
            can_hit_camo: true,
            can_pop_lead: false,
            slow_factor: 1.0,
            slow_duration: 0.0,
            color: Color::srgb(0.65, 0.12, 0.15),
            path1: [
                UpgradeCfg { name: "Discipline", cost: 180 },
                UpgradeCfg { name: "Sharp Stars", cost: 250 },
                UpgradeCfg { name: "Double Shot", cost: 580 },
            ],
            path2: [
                UpgradeCfg { name: "Seeking Star", cost: 200 },
                UpgradeCfg { name: "Distraction", cost: 290 },
                UpgradeCfg { name: "Flash Bomb", cost: 620 },
            ],
        },
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
    }

    s
}
