use bevy::prelude::*;

use crate::core::state::MatchSettings;
use crate::core::types::{MatchEntity, Position, Side, TowerType};
use crate::data::maps_cfg::waypoints_for_side;
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats};
use crate::plugins::sprites::SpriteAssets;
use crate::utils::{path_total_length, sample_path};

use super::units::{PathUnit, UnitKind};
use super::{Tower, TowerBuff};

#[derive(Component, Debug, Clone)]
pub struct CrookBossState {
    pub spawn_timer: f32,
}

impl Default for CrookBossState {
    fn default() -> Self {
        Self { spawn_timer: 1.5 }
    }
}

pub fn crook_boss_spawn_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    sprites: Res<SpriteAssets>,
    mut bosses: Query<(Entity, &Side, &Tower, &TowerBuff, &mut CrookBossState)>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;
    let player_wps = waypoints_for_side(settings.map_id, Side::Player);
    let bot_wps = waypoints_for_side(settings.map_id, Side::Bot);
    let player_len = path_total_length(&player_wps);
    let bot_len = path_total_length(&bot_wps);

    for (t_e, side, tower, buff, mut state) in &mut bosses {
        if tower.tower_type != TowerType::CrookBoss {
            continue;
        }
        let stats = apply_village_buff(
            compute_effective_stats(tower.tower_type, tower.path1, tower.path2),
            buff.0,
        );
        let (wps, total_len) = match side {
            Side::Player => (&player_wps, player_len),
            Side::Bot => (&bot_wps, bot_len),
        };

        state.spawn_timer -= dt;
        if state.spawn_timer > 0.0 {
            continue;
        }
        state.spawn_timer += stats.summon_interval / buff.0.speed_mult.max(1.0);

        let (kind, hp, speed, gun_cd, gun_dmg, gun_pierce, gun_range, exp_dmg, exp_rad, size, tint) =
            match stats.crook_tier {
                0 => (UnitKind::Crook, 24, 78.0, 0.65, 1, 1, 110.0, 0, 0.0, 23.0, Color::WHITE),
                1 => (UnitKind::Crook, 45, 82.0, 0.48, 2, 1, 120.0, 0, 0.0, 24.0, Color::srgb(0.92, 0.92, 0.96)),
                2 => (
                    UnitKind::TommyGoon,
                    75,
                    84.0,
                    0.20,
                    2,
                    2,
                    130.0,
                    0,
                    0.0,
                    25.0,
                    Color::srgb(0.95, 0.88, 0.72),
                ),
                _ => (
                    UnitKind::MafiaEnforcer,
                    300,
                    68.0,
                    0.14,
                    2,
                    3,
                    145.0,
                    12,
                    56.0,
                    28.0,
                    Color::WHITE,
                ),
            };

        let is_enforcer = kind == UnitKind::MafiaEnforcer;
        for idx in 0..stats.summon_count {
            let start_prog = (total_len - (idx as f32) * 16.0).max(0.0);
            let (exit_pos, _) = sample_path(wps, start_prog);
            commands.spawn((
                MatchEntity,
                *side,
                Position(exit_pos),
                PathUnit {
                    source_tower: t_e,
                    kind,
                    hp,
                    speed,
                    path_progress: start_prog,
                    fire_cooldown: 0.2 + (idx as f32) * 0.1,
                    grenade_cooldown: 1.4,
                    gun_damage: gun_dmg + buff.0.damage_bonus,
                    gun_pierce: gun_pierce + buff.0.pierce_bonus,
                    gun_cd: gun_cd / buff.0.speed_mult.max(1.0),
                    gun_range: gun_range * buff.0.range_mult,
                    can_hit_camo: stats.can_hit_camo,
                    can_pop_lead: stats.can_pop_lead,
                    ceramic_bonus: stats.ceramic_bonus_damage,
                    moab_bonus: stats.moab_bonus_damage,
                    explode_damage: exp_dmg,
                    explode_radius: exp_rad,
                },
                Sprite {
                    image: sprites.crook_unit_handle(is_enforcer),
                    color: tint,
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_xyz(exit_pos.x, exit_pos.y, 6.8),
            ));
        }
    }
}
