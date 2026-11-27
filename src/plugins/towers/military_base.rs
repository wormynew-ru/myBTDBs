use bevy::prelude::*;

use crate::core::events::{BloonPoppedEvent, SpawnBloonEvent};
use crate::core::state::MatchSettings;
use crate::core::types::{Health, MatchEntity, Position, Side, TowerType};
use crate::data::bloons_cfg::bloon_cfg;
use crate::data::maps_cfg::waypoints_for_side;
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats};
use crate::plugins::bloons::Bloon;
use crate::plugins::sprites::SpriteAssets;
use crate::utils::{circles_overlap, path_total_length, sample_path};

use super::units::{PathUnit, UnitKind};
use super::units_fx::{apply_multi_layer_damage, UnitExplosionFx};
use super::{Tower, TowerBuff};

#[derive(Component, Debug, Clone)]
pub struct MilitaryBaseState {
    pub spawn_timer: f32,
    pub airstrike_timer: f32,
}

impl Default for MilitaryBaseState {
    fn default() -> Self {
        Self {
            spawn_timer: 1.5,
            airstrike_timer: 6.0,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn military_base_spawn_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    sprites: Res<SpriteAssets>,
    mut bases: Query<(Entity, &Side, &mut Tower, &TowerBuff, &mut MilitaryBaseState)>,
    mut bloons: Query<(Entity, &Side, &Bloon, &mut Health, &Position), Without<Tower>>,
    mut spawn_writer: EventWriter<SpawnBloonEvent>,
    mut pop_writer: EventWriter<BloonPoppedEvent>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;
    let player_wps = waypoints_for_side(settings.map_id, Side::Player);
    let bot_wps = waypoints_for_side(settings.map_id, Side::Bot);
    let player_len = path_total_length(&player_wps);
    let bot_len = path_total_length(&bot_wps);

    for (t_e, side, mut tower, buff, mut state) in &mut bases {
        if tower.tower_type != TowerType::MilitaryBase {
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
        if state.spawn_timer <= 0.0 {
            state.spawn_timer += 1.0 / stats.fire_rate.max(0.04);
            let (exit_pos, _) = sample_path(wps, total_len);
            let (size, tint) = match stats.unit_tier {
                0..=1 => (
                    24.0,
                    if stats.unit_tier == 1 {
                        Color::srgb(0.75, 0.85, 0.92)
                    } else {
                        Color::WHITE
                    },
                ),
                2 => (26.0, Color::srgb(0.55, 0.65, 0.72)),
                3 => (30.0, Color::WHITE),
                _ => (33.0, Color::WHITE),
            };
            let (gun_cd, gun_dmg, gun_pierce, moab_b, gun_range) = match stats.unit_tier {
                0 => (0.0, 0, 0, 0, 0.0),
                1 => (0.30, 1, 1, 0, 130.0),
                2 => (0.18, 2, 2, 0, 130.0),
                3 => (0.85, 4, 1, 0, 130.0),
                _ => (0.90, 14, 4, 75, 175.0),
            };
            commands.spawn((
                MatchEntity,
                *side,
                Position(exit_pos),
                PathUnit {
                    source_tower: t_e,
                    kind: UnitKind::MilitaryVehicle(stats.unit_tier),
                    hp: stats.unit_hp,
                    speed: stats.unit_speed,
                    path_progress: total_len,
                    fire_cooldown: 0.25,
                    grenade_cooldown: 0.0,
                    gun_damage: gun_dmg,
                    gun_pierce,
                    gun_cd,
                    gun_range,
                    can_hit_camo: true,
                    can_pop_lead: stats.unit_tier >= 2 || stats.can_pop_lead,
                    ceramic_bonus: 0,
                    moab_bonus: moab_b,
                    explode_damage: 10 + (stats.unit_tier as i32) * 8,
                    explode_radius: 46.0 + (stats.unit_tier as f32) * 6.0,
                },
                Sprite {
                    image: sprites.unit_handle(stats.unit_tier),
                    color: tint,
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_xyz(exit_pos.x, exit_pos.y, 6.8),
            ));
        }

        if stats.has_airstrike {
            state.airstrike_timer -= dt;
            if state.airstrike_timer <= 0.0 {
                state.airstrike_timer += 14.0 / buff.0.speed_mult.max(1.0);
                let side_pts: Vec<Vec2> = bloons
                    .iter()
                    .filter(|(_, bs, b, hp, _)| {
                        **bs == *side && hp.current > 0 && !b.is_inside_chipper
                    })
                    .map(|(_, _, _, _, p)| p.0)
                    .collect();

                if let Some((best_pos, _)) = side_pts
                    .iter()
                    .map(|&p| {
                        (
                            p,
                            side_pts.iter().filter(|&&o| p.distance(o) <= 88.0).count(),
                        )
                    })
                    .max_by_key(|(_, c)| *c)
                {
                    commands.spawn((
                        MatchEntity,
                        UnitExplosionFx {
                            life: 0.36,
                            max_life: 0.36,
                            target_size: 185.0,
                            color: Color::srgba(0.98, 0.45, 0.12, 0.85),
                        },
                        Sprite::from_color(
                            Color::srgba(0.98, 0.45, 0.12, 0.85),
                            Vec2::splat(20.0),
                        ),
                        Transform::from_xyz(best_pos.x, best_pos.y, 9.2),
                    ));
                    for (b_e, bs, b, mut hp, bp) in &mut bloons {
                        if *bs != *side || hp.current <= 0 || b.is_inside_chipper {
                            continue;
                        }
                        if circles_overlap(best_pos, 90.0, bp.0, bloon_cfg(b.layer).size * 0.5) {
                            let dmg = if b.layer.is_moab_class() { 55 } else { 20 };
                            apply_multi_layer_damage(
                                &mut commands,
                                b_e,
                                *side,
                                b,
                                &mut hp,
                                dmg,
                                Some(&mut tower),
                                &mut spawn_writer,
                                &mut pop_writer,
                            );
                        }
                    }
                }
            }
        }
    }
}
