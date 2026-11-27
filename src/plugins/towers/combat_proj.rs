use bevy::prelude::*;

use crate::core::events::{BloonPoppedEvent, SpawnBloonEvent};
use crate::core::state::MatchSettings;
use crate::core::types::{BloonLayer, Health, Side, TowerType};
use crate::data::bloons_cfg::bloon_cfg;
use crate::data::maps_cfg::{DIVIDER_HALF, HALF_W};
use crate::plugins::bloons::Bloon;
use crate::utils::circles_overlap;

use super::{Projectile, Tower};

#[allow(clippy::too_many_arguments)]
pub fn projectile_update_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut projectiles: Query<(Entity, &Side, &mut Projectile, &mut crate::core::types::Position, &mut Transform)>,
    mut bloons: Query<
        (Entity, &Side, &mut Bloon, &mut Health, &crate::core::types::Position),
        (Without<Projectile>, Without<Tower>),
    >,
    mut towers: Query<&mut Tower, (Without<Projectile>, Without<Bloon>)>,
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
        for (b_entity, b_side, bloon, hp, b_pos) in bloons.iter() {
            if *b_side != *p_side
                || hp.current <= 0
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
                if *b_side != *p_side
                    || hp.current <= 0
                    || bloon.is_inside_chipper
                    || (bloon.camo && !proj.can_hit_camo)
                {
                    continue;
                }
                let r = bloon_cfg(bloon.layer).size * 0.5;
                if circles_overlap(p_pos.0, proj.splash_radius, b_pos.0, r) {
                    if proj.stun_duration > 0.0 {
                        bloon.freeze_timer = bloon.freeze_timer.max(proj.stun_duration);
                        bloon.slow_factor = bloon.slow_factor.min(0.15);
                    }
                    if !bloon.lead || proj.can_pop_lead {
                        let dmg = projectile_damage(&proj, bloon.layer);
                        apply_damage(
                            &mut commands,
                            b_entity,
                            *p_side,
                            &bloon,
                            &mut hp,
                            dmg,
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
            if (!bloon.lead || proj.can_pop_lead) && hp.current > 0 {
                let dmg = projectile_damage(&proj, bloon.layer);
                apply_damage(
                    &mut commands,
                    first_hit,
                    *p_side,
                    &bloon,
                    &mut hp,
                    dmg,
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

fn projectile_damage(proj: &Projectile, layer: BloonLayer) -> i32 {
    let mut dmg = proj.damage;
    if layer.is_moab_class() {
        dmg += proj.moab_bonus;
    }
    if layer == BloonLayer::Ceramic {
        dmg += proj.ceramic_bonus;
    }
    dmg
}

#[allow(clippy::too_many_arguments)]
pub fn apply_damage(
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
    let mut bonus_cash = 0;
    if let Some(t) = tower {
        let prev_tens = t.pops / 10;
        t.pops += dealt as u32;
        let new_tens = t.pops / 10;
        if t.tower_type == TowerType::CrookBoss && t.path1 >= 4 && new_tens > prev_tens {
            bonus_cash = ((new_tens - prev_tens) as i32) * 15;
        }
    }

    if hp.current > 0 {
        pop_writer.send(BloonPoppedEvent {
            side,
            cash_gain: bonus_cash,
            pops: dealt as u32,
        });
        return;
    }

    pop_writer.send(BloonPoppedEvent {
        side,
        cash_gain: 1 + bonus_cash,
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
