use bevy::prelude::*;

use crate::core::events::{BloonPoppedEvent, SpawnBloonEvent};
use crate::core::state::MatchSettings;
use crate::core::types::{Health, MatchEntity, Position, Side, TargetPriority, TowerType};
use crate::data::bloons_cfg::bloon_cfg;
use crate::data::maps_cfg::{DIVIDER_HALF, HALF_W};
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats};
use crate::plugins::bloons::Bloon;
use crate::plugins::sprites::SpriteAssets;
use crate::utils::circles_overlap;

use super::{Projectile, Tower, TowerBuff};

fn select_target(
    tower_pos: Vec2,
    side: Side,
    priority: TargetPriority,
    range: f32,
    can_hit_camo: bool,
    bloons: &Query<(Entity, &Side, &mut Bloon, &mut Health, &Position)>,
) -> Option<(Entity, Vec2)> {
    let mut best: Option<(Entity, Vec2, f32)> = None;
    for (entity, b_side, bloon, _, b_pos) in bloons.iter() {
        if *b_side != side
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
    mut bloons: Query<(Entity, &Side, &mut Bloon, &mut Health, &Position)>,
    mut spawn_writer: EventWriter<SpawnBloonEvent>,
    mut pop_writer: EventWriter<BloonPoppedEvent>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;

    for (t_entity, t_side, mut tower, buff, t_pos, mut tf) in &mut towers {
        if matches!(tower.tower_type, TowerType::Bloonchipper | TowerType::Village) {
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
            let any_in_range = bloons.iter().any(|(_, bs, b, _, bp)| {
                *bs == *t_side
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

#[allow(clippy::too_many_arguments)]
pub fn projectile_update_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut projectiles: Query<(Entity, &Side, &mut Projectile, &mut Position, &mut Transform)>,
    mut bloons: Query<(Entity, &Side, &mut Bloon, &mut Health, &Position), Without<Projectile>>,
    mut towers: Query<&mut Tower>,
    mut spawn_writer: EventWriter<SpawnBloonEvent>,
    mut pop_writer: EventWriter<BloonPoppedEvent>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;

    for (p_entity, p_side, mut proj, mut p_pos, mut tf) in &mut projectiles {
        proj.life -= dt;
        p_pos.0 += proj.velocity * dt;
        tf.translation.x = p_pos.0.x;
        tf.translation.y = p_pos.0.y;

        let out_of_half = match p_side {
            Side::Player => p_pos.0.x < -HALF_W - 10.0 || p_pos.0.x > DIVIDER_HALF,
            Side::Bot => p_pos.0.x > HALF_W + 10.0 || p_pos.0.x < -DIVIDER_HALF,
        };
        if proj.life <= 0.0 || proj.pierce_left == 0 || out_of_half {
            commands.entity(p_entity).despawn_recursive();
            continue;
        }

        let mut hit_target = None;
        for (b_entity, b_side, bloon, _, b_pos) in bloons.iter() {
            if *b_side != *p_side
                || bloon.is_inside_chipper
                || (bloon.camo && !proj.can_hit_camo)
                || proj.hit_ids.contains(&b_entity)
            {
                continue;
            }
            let r = bloon_cfg(bloon.layer).size * 0.5;
            if circles_overlap(p_pos.0, 6.0, b_pos.0, r) {
                hit_target = Some(b_entity);
                break;
            }
        }

        let Some(first_hit) = hit_target else {
            continue;
        };

        let mut src_tower = towers.get_mut(proj.source_tower).ok();
        if proj.splash_radius > 0.0 {
            for (b_entity, b_side, mut bloon, mut hp, b_pos) in &mut bloons {
                if *b_side != *p_side || bloon.is_inside_chipper || (bloon.camo && !proj.can_hit_camo) {
                    continue;
                }
                let r = bloon_cfg(bloon.layer).size * 0.5;
                if circles_overlap(p_pos.0, proj.splash_radius, b_pos.0, r) {
                    if proj.stun_duration > 0.0 {
                        bloon.freeze_timer = bloon.freeze_timer.max(proj.stun_duration);
                        bloon.slow_factor = bloon.slow_factor.min(0.15);
                    }
                    if !bloon.lead || proj.can_pop_lead {
                        apply_damage(
                            &mut commands,
                            b_entity,
                            *p_side,
                            &bloon,
                            &mut hp,
                            proj.damage,
                            src_tower.as_deref_mut(),
                            &mut spawn_writer,
                            &mut pop_writer,
                        );
                    }
                }
            }
            commands.entity(p_entity).despawn_recursive();
        } else if let Ok((_, _, mut bloon, mut hp, _)) = bloons.get_mut(first_hit) {
            proj.hit_ids.push(first_hit);
            proj.pierce_left = proj.pierce_left.saturating_sub(1);
            if proj.knockback_chance > 0.0 && (proj.life * 100.0).fract() < proj.knockback_chance {
                bloon.path_progress = (bloon.path_progress - 26.0).max(0.0);
            }
            if !bloon.lead || proj.can_pop_lead {
                apply_damage(
                    &mut commands,
                    first_hit,
                    *p_side,
                    &bloon,
                    &mut hp,
                    proj.damage,
                    src_tower.as_deref_mut(),
                    &mut spawn_writer,
                    &mut pop_writer,
                );
            }
            if proj.pierce_left == 0 {
                commands.entity(p_entity).despawn_recursive();
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_damage(
    commands: &mut Commands,
    bloon_entity: Entity,
    side: Side,
    bloon: &Bloon,
    hp: &mut Health,
    damage: i32,
    tower: Option<&mut Tower>,
    spawn_writer: &mut EventWriter<SpawnBloonEvent>,
    pop_writer: &mut EventWriter<BloonPoppedEvent>,
) {
    let dealt = hp.current.min(damage).max(0);
    hp.current -= damage;
    if let Some(t) = tower {
        t.pops += dealt as u32;
    }

    if hp.current > 0 {
        pop_writer.send(BloonPoppedEvent {
            side,
            cash_gain: 0,
            pops: dealt as u32,
        });
        return;
    }

    pop_writer.send(BloonPoppedEvent {
        side,
        cash_gain: 1,
        pops: 1,
    });
    commands.entity(bloon_entity).despawn_recursive();

    if let Some((child_layer, count)) = bloon_cfg(bloon.layer).child {
        for c in 0..count {
            spawn_writer.send(SpawnBloonEvent {
                side,
                layer: child_layer,
                camo: bloon.camo,
                lead: false,
                path_progress: (bloon.path_progress - (c as f32) * 12.0).max(0.0),
                from_send: bloon.from_send,
                permafrost: bloon.permafrost,
                regrab_immune_tower: bloon.regrab_immune_tower,
                regrab_cooldown: bloon.regrab_cooldown,
            });
        }
    }
}
