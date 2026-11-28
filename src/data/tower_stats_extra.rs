use crate::core::types::TowerType;

use super::EffectiveStats;

pub fn apply_extra_stats(tower_type: TowerType, path1: u8, path2: u8, s: &mut EffectiveStats) {
    match tower_type {
        TowerType::MilitaryBase => {
            if path1 >= 1 { s.unit_hp = 75; s.unit_speed = 90.0; s.unit_tier = 1; }
            if path1 >= 2 { s.unit_hp = 160; s.unit_speed = 82.0; s.unit_tier = 2; s.can_pop_lead = true; }
            if path1 >= 3 { s.unit_hp = 450; s.unit_speed = 68.0; s.unit_tier = 3; }
            if path1 >= 4 { s.unit_hp = 1400; s.unit_speed = 62.0; s.unit_tier = 4; }
            let spawn_secs = match path2 {
                0 => 16.2,
                1 => 13.77,
                2 => 11.88,
                3 => 10.26,
                _ => 8.775,
            };
            s.fire_rate = 1.0 / spawn_secs;
            if path2 >= 1 { s.unit_speed *= 1.15; }
            if path2 >= 2 { s.unit_speed *= 1.13; }
            if path2 >= 3 { s.unit_speed *= 1.12; s.unit_hp = ((s.unit_hp as f32) * 1.20).round() as i32; }
            if path2 >= 4 { s.unit_speed *= 1.10; s.has_airstrike = true; }
        }
        TowerType::CrookBoss => {
            if path1 >= 1 { s.fire_rate *= 1.25; s.range += 22.0; }
            if path1 >= 2 { s.fire_rate *= 2.40; s.proj_speed = 620.0; }
            if path1 >= 3 {
                s.can_pop_lead = true;
                s.ceramic_bonus_damage = 2;
                s.moab_bonus_damage = 2;
            }
            if path1 >= 4 {
                s.fire_rate *= 1.90;
                s.damage += 2;
                s.pierce = 4;
                s.is_golden_boss = true;
            }
            if path2 >= 1 { s.can_hit_camo = true; }
            if path2 >= 2 { s.summon_interval = 13.5; s.crook_tier = 1; }
            if path2 >= 3 { s.summon_count = 2; s.crook_tier = 2; }
            if path2 >= 4 { s.summon_interval = 12.15; s.summon_count = 1; s.crook_tier = 3; }
        }
        TowerType::Farm => {
            if path1 >= 1 { s.farm_interval = 3.5; s.farm_cash = 22; }
            if path1 >= 2 { s.farm_cash = 38; }
            if path1 >= 3 { s.farm_interval = 3.2; s.farm_cash = 70; }
            if path1 >= 4 { s.farm_interval = 3.0; s.farm_cash = 136; }
            if path2 >= 1 { s.farm_eco_bonus = 5; }
            if path2 >= 2 { s.farm_eco_bonus = 11; s.farm_send_discount = 0.05; }
            if path2 >= 3 { s.farm_eco_bonus = 24; }
            if path2 >= 4 {
                s.farm_eco_bonus = 54;
                s.farm_dividend_cash = 160;
                s.farm_dividend_interval = 15.0;
            }
        }
        TowerType::GlueGunner => {
            if path1 >= 1 { s.glue_soak = true; }
            if path1 >= 2 { s.glue_corrosive_dmg = 1; s.glue_corrosive_interval = 1.5; s.can_pop_lead = true; }
            if path1 >= 3 { s.glue_corrosive_interval = 0.5; s.glue_ceramic_bonus = 1; }
            if path1 >= 4 {
                s.glue_corrosive_dmg = 2;
                s.glue_corrosive_interval = 0.25;
                s.glue_moab_capable = true;
            }
            if path2 >= 1 { s.pierce = 2; }
            if path2 >= 2 { s.splash_radius = 52.0; s.pierce = 6; }
            if path2 >= 3 { s.fire_rate *= 3.0; }
            if path2 >= 4 { s.has_glue_striker = true; }
        }
        TowerType::SoulEater => {}
        TowerType::Necromancer => {
            if path1 >= 1 { s.soul_capacity = 45; s.damage += 1; s.pierce += 1; }
            if path1 >= 2 { s.damage += 2; s.splash_radius = 36.0; s.ceramic_bonus_damage = 2; }
            if path1 >= 3 { s.soul_capacity = 65; s.damage += 2; s.fire_rate *= 1.25; }
            if path1 >= 4 {
                s.soul_capacity = 80;
                s.damage += 3;
                s.moab_bonus_damage = 4;
                s.summon_ghost_knight = true;
            }
            if path2 >= 1 { s.can_hit_camo = true; s.ghost_camo = true; }
            if path2 >= 2 { s.rift_interval = 7.0; s.ghost_speed_mult = 1.25; }
            if path2 >= 3 { s.rift_interval = 5.0; s.ghost_speed_mult = 1.50; s.ghost_hp = 4; }
            if path2 >= 4 {
                s.rift_interval = 3.5;
                s.ghost_speed_mult = 1.75;
                s.ghost_hp = 8;
                s.ghost_lead = true;
            }
        }
        TowerType::Saboteur => {
            if path1 >= 1 { s.saboteur_jam_duration = 6.0; s.range += 15.0; s.splash_radius += 15.0; }
            if path1 >= 2 { s.damage += 1; s.can_pop_lead = true; s.saboteur_jam_duration = 6.5; }
            if path1 >= 3 { s.saboteur_jam_targets = 2; s.saboteur_jam_duration = 7.2; }
            if path1 >= 4 { s.saboteur_jam_duration = 8.0; s.damage += 1; }
            if path2 >= 1 { s.saboteur_ability_cd = 22.0; s.saboteur_slow = 0.78; }
            if path2 >= 2 { s.saboteur_ability_cd = 19.0; s.saboteur_slow = 0.65; }
            if path2 >= 3 { s.saboteur_ability_cd = 15.0; s.saboteur_slow = 0.50; s.fire_rate *= 1.25; }
            if path2 >= 4 { s.saboteur_ability_cd = 12.0; s.saboteur_slow = 0.40; s.range += 20.0; s.splash_radius += 20.0; }
        }
        TowerType::ScrapCollector => {
            if path1 >= 1 { s.scrap_cash = 5; s.scrap_moab_cash = 45; }
            if path1 >= 2 { s.scrap_cash = 7; s.scrap_moab_cash = 75; s.range += 15.0; }
            if path1 >= 3 { s.scrap_cash = 9; s.scrap_moab_cash = 115; }
            if path1 >= 4 { s.scrap_cash = 12; s.scrap_moab_cash = 180; s.range += 15.0; }
            if path2 >= 1 { s.scrap_max_speed_bonus = 0.20; }
            if path2 >= 2 { s.range += 25.0; s.scrap_max_speed_bonus = 0.23; }
            if path2 >= 3 { s.scrap_max_speed_bonus = 0.26; s.scrap_heat_decay = 2.4; }
            if path2 >= 4 { s.scrap_max_speed_bonus = 0.30; s.range += 20.0; s.scrap_pierce_bonus = 1; }
        }
        TowerType::MirageGenerator => {
            if path1 >= 1 { s.mirage_interval = 14.0; }
            if path1 >= 2 { s.mirage_interval = 12.0; s.mirage_speed_mult = 1.25; }
            if path1 >= 3 { s.mirage_interval = 10.5; s.mirage_count = 2; }
            if path1 >= 4 { s.mirage_interval = 9.0; s.mirage_count = 2; s.mirage_force_zomg = true; }
            if path2 >= 1 { s.mirage_flash_duration = 0.9; s.mirage_flash_radius = 95.0; }
            if path2 >= 2 { s.mirage_flash_duration = 1.5; s.mirage_flash_radius = 125.0; }
            if path2 >= 3 { s.mirage_flash_duration = 2.0; s.mirage_flash_radius = 160.0; }
            if path2 >= 4 {
                s.mirage_flash_duration = 2.6;
                s.mirage_flash_radius = 200.0;
                s.mirage_interval = s.mirage_interval.min(13.0);
            }
        }
        _ => {}
    }
}
