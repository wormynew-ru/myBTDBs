use bevy::prelude::*;

use crate::core::types::TowerType;

#[path = "towers_cfg_extra.rs"]
mod towers_cfg_extra;

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
    pub path1: &'static [UpgradeCfg],
    pub path2: &'static [UpgradeCfg],
}

pub const ALL_TOWERS: [TowerType; 11] = [
    TowerType::DartMonkey,
    TowerType::Cannon,
    TowerType::IceTower,
    TowerType::NinjaMonkey,
    TowerType::Bloonchipper,
    TowerType::Village,
    TowerType::SpikeFactory,
    TowerType::Commander,
    TowerType::MilitaryBase,
    TowerType::CrookBoss,
    TowerType::Farm,
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
            path1: &[
                UpgradeCfg { name: "Long Range", cost: 90 },
                UpgradeCfg { name: "Camo Eyes", cost: 135 },
                UpgradeCfg { name: "Spike-O-Pult", cost: 360 },
            ],
            path2: &[
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
            path1: &[
                UpgradeCfg { name: "Bigger Bombs", cost: 180 },
                UpgradeCfg { name: "Heavy Shells", cost: 320 },
                UpgradeCfg { name: "Bloon Impact", cost: 540 },
            ],
            path2: &[
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
            path1: &[
                UpgradeCfg { name: "Permafrost", cost: 140 },
                UpgradeCfg { name: "Snap Freeze", cost: 260 },
                UpgradeCfg { name: "Ice Shards", cost: 480 },
            ],
            path2: &[
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
            path1: &[
                UpgradeCfg { name: "Discipline", cost: 180 },
                UpgradeCfg { name: "Sharp Stars", cost: 250 },
                UpgradeCfg { name: "Double Shot", cost: 580 },
            ],
            path2: &[
                UpgradeCfg { name: "Seeking Star", cost: 200 },
                UpgradeCfg { name: "Distraction", cost: 290 },
                UpgradeCfg { name: "Flash Bomb", cost: 620 },
            ],
        },
        TowerType::Bloonchipper => TowerCfg {
            tower_type,
            name: "Bloonchipper",
            short_name: "Chipper",
            cost: 420,
            range: 112.0,
            fire_rate: 1.0,
            damage: 1,
            pierce: 1,
            proj_speed: 0.0,
            splash_radius: 0.0,
            can_hit_camo: false,
            can_pop_lead: false,
            slow_factor: 1.0,
            slow_duration: 0.0,
            color: Color::srgb(0.15, 0.39, 0.92),
            path1: &[
                UpgradeCfg { name: "Fast Funnel", cost: 150 },
                UpgradeCfg { name: "Heavy Duty", cost: 320 },
                UpgradeCfg { name: "Dual Blades", cost: 620 },
                UpgradeCfg { name: "MOAB Shred", cost: 2200 },
            ],
            path2: &[
                UpgradeCfg { name: "Wide Funnel", cost: 130 },
                UpgradeCfg { name: "Faster Shred", cost: 280 },
                UpgradeCfg { name: "Triple Barrel", cost: 850 },
                UpgradeCfg { name: "Supa-Vac", cost: 2600 },
            ],
        },
        TowerType::Village => TowerCfg {
            tower_type,
            name: "Monkey Village",
            short_name: "Village",
            cost: 550,
            range: 135.0,
            fire_rate: 0.0,
            damage: 0,
            pierce: 0,
            proj_speed: 0.0,
            splash_radius: 0.0,
            can_hit_camo: false,
            can_pop_lead: false,
            slow_factor: 1.0,
            slow_duration: 0.0,
            color: Color::srgb(0.76, 0.62, 0.42),
            path1: &[
                UpgradeCfg { name: "Big Beacon", cost: 240 },
                UpgradeCfg { name: "Jungle Drums", cost: 520 },
                UpgradeCfg { name: "Training", cost: 780 },
            ],
            path2: &[
                UpgradeCfg { name: "Town Reach", cost: 260 },
                UpgradeCfg { name: "Radar Scan", cost: 580 },
                UpgradeCfg { name: "MIB Bureau", cost: 1450 },
            ],
        },
        TowerType::SpikeFactory => TowerCfg {
            tower_type,
            name: "Spike Factory",
            short_name: "Spikes",
            cost: 500,
            range: 75.0,
            fire_rate: 0.625,
            damage: 1,
            pierce: 5,
            proj_speed: 0.0,
            splash_radius: 0.0,
            can_hit_camo: true,
            can_pop_lead: false,
            slow_factor: 1.0,
            slow_duration: 0.0,
            color: Color::srgb(0.39, 0.45, 0.55),
            path1: &[
                UpgradeCfg { name: "Bigger Stacks", cost: 220 },
                UpgradeCfg { name: "Hot Spikes", cost: 420 },
                UpgradeCfg { name: "Spiked Balls", cost: 1100 },
                UpgradeCfg { name: "Spiked Mines", cost: 2800 },
            ],
            path2: &[
                UpgradeCfg { name: "Faster Prod", cost: 240 },
                UpgradeCfg { name: "Even Faster", cost: 400 },
                UpgradeCfg { name: "Long Life", cost: 950 },
                UpgradeCfg { name: "Spike Storm", cost: 2600 },
            ],
        },
        TowerType::Commander
        | TowerType::MilitaryBase
        | TowerType::CrookBoss
        | TowerType::Farm => towers_cfg_extra::extra_tower_cfg(tower_type),
    }
}
