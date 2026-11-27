use bevy::prelude::*;

use crate::core::events::{BloonPoppedEvent, SpawnBloonEvent};
use crate::core::state::MatchSettings;
use crate::core::types::{Health, MatchEntity, Position, Side, TargetPriority, TowerType};
use crate::data::bloons_cfg::bloon_cfg;
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats};
use crate::plugins::bloons::Bloon;
use crate::plugins::sprites::SpriteAssets;

use super::combat_proj::apply_damage;
use super::{Projectile, Tower, TowerBuff};

fn select_target(
    tower_pos: Vec2,
    side: Side,
    priority: TargetPriority,
    range: f32,
    can_hit_camo: bool,
    bloons: &Query<(Entity, &Side, &mut Bloon, &mut Health, &Position), Without<Tower>>,
) -> Option<(Entity, Vec2)> {
    let mut best: Option<(Entity, Vec2, f32)> = None;
    for (entity, b_side, bloon, hp, b_pos) in bloons.iter() {
        if *b_side != side
            || hp.current <= 0
            || bloon.is_inside_chipper
            || (bloon.camo && !can_hit_camo)
        {
            continue;
        }
        let d = tower_pos.distance(b_pos.0);
        if d > range {
            continue;
        }
        let score = match priority {
            TargetPriority::First => bloon.path_progress,
            TargetPriority::Last => -bloon.path_progress,
            TargetPriority::Close => -d,
            TargetPriority::Strong => {
                (bloon_cfg(bloon.layer).rbe as f32) * 10000.0 + bloon.path_progress
            }
        };
        if best.map_or(true, |(_, _, b_score)| score > b_score) {
            best = Some((entity, b_pos.0, score));
        }
    }
    best.map(|(e, p, _)| (e, p))
}

#[allow(clippy::too_many_arguments)]
pub fn tower_attack_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    sprites: Res<SpriteAssets>,
    mut towers: Query<(Entity, &Side, &mut Tower, &TowerBuff, &Position, &mut Transform)>,
    mut bloons: Query<(Entity, &Side, &mut Bloon, &mut Health, &Position), Without<Tower>>,
    mut spawn_writer: EventWriter<SpawnBloonEvent>,
    mut pop_writer: EventWriter<BloonPoppedEvent>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;

    for (t_entity, t_side, mut tower, buff, t_pos, mut tf) in &mut towers {
        if matches!(
            tower.tower_type,
            TowerType::Bloonchipper
                | TowerType::Village
                | TowerType::SpikeFactory
                | TowerType::MilitaryBase
                | TowerType::Farm
        ) {
            continue;
        }

        tower.cooldown = (tower.cooldown - dt).max(0.0);
        if tower.cooldown > 0.0 {
            continue;
        }
        let stats = apply_village_buff(
            compute_effective_stats(tower.tower_type, tower.path1, tower.path2),
            buff.0,
        );

        if tower.tower_type == TowerType::IceTower {
            let any_in_range = bloons.iter().any(|(_, bs, b, hp, bp)| {
                *bs == *t_side
                    && hp.current > 0
                    && !b.is_inside_chipper
                    && (!b.camo || stats.can_hit_camo)
                    && t_pos.0.distance(bp.0) <= stats.range
            });
            if !any_in_range {
                continue;
            }
            tower.cooldown = 1.0 / stats.fire_rate;
            for (b_entity, bs, mut bloon, mut hp, bp) in &mut bloons {
                if *bs != *t_side
                    || hp.current <= 0
                    || bloon.is_inside_chipper
                    || (bloon.camo && !stats.can_hit_camo)
                    || t_pos.0.distance(bp.0) > stats.splash_radius
                {
                    continue;
                }
                if !bloon.lead || stats.can_pop_lead {
                    bloon.freeze_timer = bloon.freeze_timer.max(stats.slow_duration);
                    bloon.slow_factor = bloon.slow_factor.min(stats.slow_factor);
                    if stats.permafrost {
                        bloon.permafrost = true;
                    }
                    if stats.damage > 0 {
                        apply_damage(
                            &mut commands,
                            b_entity,
                            *t_side,
                            &bloon,
                            &mut hp,
                            stats.damage,
                            Some(&mut tower),
                            &mut spawn_writer,
                            &mut pop_writer,
                        );
                    }
                }
            }
            continue;
        }

        let Some((_, target_pos)) = select_target(
            t_pos.0,
            *t_side,
            tower.priority,
            stats.range,
            stats.can_hit_camo,
            &bloons,
        ) else {
            continue;
        };

        let dir = (target_pos - t_pos.0).normalize_or(Vec2::X);
        let base_angle = dir.y.atan2(dir.x);
        tf.rotation = Quat::from_rotation_z(base_angle);
        tower.cooldown = 1.0 / stats.fire_rate;
        tower.shot_count += 1;

        let is_flash = stats.flash_every_4 && tower.shot_count % 4 == 0;
        let offsets: &[f32] = if is_flash || stats.multi_shot == 1 {
            &[0.0]
        } else if stats.multi_shot == 2 {
            &[-0.11, 0.11]
        } else {
            &[-0.20, 0.0, 0.20]
        };

        for &off in offsets {
            let ang = base_angle + off;
            let spd = if is_flash { 440.0 } else { stats.proj_speed };
            let vel = Vec2::new(ang.cos(), ang.sin()) * spd;
            let proj_img = match tower.tower_type {
                TowerType::Cannon => sprites.proj_bomb.clone(),
                TowerType::NinjaMonkey => sprites.proj_shuriken.clone(),
                _ => sprites.proj_dart.clone(),
            };
            let color = if is_flash {
                Color::srgb(0.99, 0.94, 0.54)
            } else if tower.tower_type == TowerType::CrookBoss && stats.is_golden_boss {
                Color::srgb(0.98, 0.80, 0.15)
            } else if tower.tower_type == TowerType::CrookBoss {
                Color::srgb(0.95, 0.90, 0.72)
            } else {
                Color::WHITE
            };

            commands.spawn((
                MatchEntity,
                *t_side,
                Position(t_pos.0 + Vec2::new(ang.cos(), ang.sin()) * 14.0),
                Projectile {
                    source_tower: t_entity,
                    velocity: vel,
                    damage: if is_flash { 2 } else { stats.damage },
                    pierce_left: if is_flash { 1 } else { stats.pierce },
                    splash_radius: if is_flash { 68.0 } else { stats.splash_radius },
                    can_pop_lead: is_flash || stats.can_pop_lead,
                    can_hit_camo: is_flash || stats.can_hit_camo,
                    knockback_chance: stats.knockback_chance,
                    stun_duration: if is_flash { 0.85 } else { stats.stun_duration },
                    ceramic_bonus: stats.ceramic_bonus_damage,
                    moab_bonus: stats.moab_bonus_damage,
                    life: 1.15,
                    hit_ids: Vec::new(),
                },
                Sprite {
                    image: proj_img,
                    color,
                    custom_size: Some(Vec2::splat(if is_flash { 18.0 } else { 14.0 })),
                    ..default()
                },
                Transform::from_xyz(t_pos.0.x, t_pos.0.y, 9.0)
                    .with_rotation(Quat::from_rotation_z(ang)),
            ));
        }
    }
}
