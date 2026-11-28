use bevy::prelude::*;

use crate::core::events::{BloonPoppedEvent, SpawnBloonEvent};
use crate::core::state::MatchSettings;
use crate::core::types::{BloonLayer, GlueStatus, MatchEntity, Position, Side, TowerType};
use crate::data::bloons_cfg::bloon_cfg;
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats};
use crate::plugins::bloons::Bloon;
use crate::plugins::sprites::SpriteAssets;

use super::chipper::{ChipperState, ShredParticle};
use super::{Tower, TowerBuff};

#[allow(clippy::too_many_arguments)]
pub(super) fn emit_shred_pop(
    commands: &mut Commands,
    bloon_entity: Entity,
    side: Side,
    b: &Bloon,
    pos: Vec2,
    tower_entity: Entity,
    child_info: Option<(BloonLayer, u8)>,
    child_progress: f32,
    child_step: f32,
    spawn_writer: &mut EventWriter<SpawnBloonEvent>,
    pop_writer: &mut EventWriter<BloonPoppedEvent>,
) {
    let cash = if b.is_ghost || b.is_mirage { 0 } else { 1 };
    let mirage_flash = if b.is_mirage && b.mirage_flash_duration > 0.0 {
        Some((b.mirage_flash_radius, b.mirage_flash_duration))
    } else {
        None
    };
    pop_writer.send(BloonPoppedEvent {
        side,
        cash_gain: cash,
        pops: if b.is_mirage { 0 } else { 1 },
        pos,
        killed: true,
        from_send: b.from_send,
        is_moab: b.layer.is_moab_class(),
        mirage_flash,
    });
    let child_glue = if b.glue.soak && b.glue.timer > 0.0 {
        GlueStatus {
            tick: b.glue.corrosive_interval,
            ..b.glue
        }
    } else {
        GlueStatus::default()
    };
    if !b.is_ghost && !b.is_mirage {
        if let Some((child_layer, count)) = child_info {
            for c in 0..count {
                spawn_writer.send(SpawnBloonEvent {
                    side,
                    layer: child_layer,
                    camo: b.camo,
                    lead: false,
                    path_progress: (child_progress - (c as f32) * child_step).max(0.0),
                    from_send: b.from_send,
                    permafrost: b.permafrost,
                    glue: child_glue,
                    regrab_immune_tower: Some(tower_entity),
                    regrab_cooldown: 0.3,
                });
            }
        }
    }
    commands.entity(bloon_entity).despawn_recursive();
}

pub(super) fn strip_normal_layer(layer: BloonLayer, layers: u8) -> Option<(BloonLayer, u8)> {
    let mut cur: Vec<(BloonLayer, u8)> = vec![(layer, 1)];
    for _ in 0..layers {
        let mut next = Vec::new();
        for (l, cnt) in cur {
            if let Some((child, mult)) = bloon_cfg(l).child {
                next.push((child, cnt * mult));
            }
        }
        cur = next;
        if cur.is_empty() {
            return None;
        }
    }
    let first_layer = cur[0].0;
    let total_count: u8 = cur.iter().map(|(_, c)| *c).sum();
    Some((first_layer, total_count))
}

pub(super) fn spawn_shred_particle(
    commands: &mut Commands,
    sprites: &SpriteAssets,
    tower_pos: Vec2,
    forward: Vec2,
    vib_phase: f32,
    p_color: Color,
) {
    let exhaust = tower_pos - forward * 12.0;
    let eject_dir = Vec2::new(-forward.x * 38.0, 44.0 + vib_phase.sin() * 14.0);
    commands.spawn((
        MatchEntity,
        ShredParticle {
            velocity: eject_dir,
            life: 0.35,
        },
        Sprite {
            image: sprites.shred_particle.clone(),
            color: p_color,
            custom_size: Some(Vec2::splat(10.0)),
            ..default()
        },
        Transform::from_xyz(exhaust.x, exhaust.y, 9.5),
    ));
}

pub fn supa_vac_system(
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut chippers: Query<(&Side, &Tower, &TowerBuff, &Position, &mut ChipperState)>,
    mut bloons: Query<(&Side, &mut Bloon, &mut Position, &mut Transform), Without<Tower>>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;

    let mut any_active_player = false;
    let mut any_active_bot = false;

    for (c_side, tower, buff, c_pos, mut chipper) in &mut chippers {
        if tower.tower_type != TowerType::Bloonchipper || tower.path2 < 4 {
            continue;
        }
        if chipper.supa_vac_cooldown > 0.0 {
            chipper.supa_vac_cooldown = (chipper.supa_vac_cooldown - dt).max(0.0);
        }

        if chipper.supa_vac_active > 0.0 {
            chipper.supa_vac_active = (chipper.supa_vac_active - dt).max(0.0);
            if chipper.supa_vac_active > 0.0 {
                match c_side {
                    Side::Player => any_active_player = true,
                    Side::Bot => any_active_bot = true,
                }
                let stats = apply_village_buff(
                    compute_effective_stats(tower.tower_type, tower.path1, tower.path2),
                    buff.0,
                );
                let hold_radius = stats.range * 0.65;

                for (b_side, mut b, mut b_pos, mut b_tf) in &mut bloons {
                    if *b_side != *c_side || b.is_inside_chipper || b.layer.is_moab_class() {
                        continue;
                    }
                    b.supa_vac_pulled = true;
                    let delta = c_pos.0 - b_pos.0;
                    let dist = delta.length();
                    if dist > hold_radius {
                        let step = (420.0 * dt).min(dist - hold_radius * 0.8);
                        b_pos.0 += (delta / dist) * step;
                        b_tf.translation.x = b_pos.0.x;
                        b_tf.translation.y = b_pos.0.y;
                    }
                }
            }
        }
    }

    for (b_side, mut b, _, _) in &mut bloons {
        if b.supa_vac_pulled {
            let active = match b_side {
                Side::Player => any_active_player,
                Side::Bot => any_active_bot,
            };
            if !active {
                b.supa_vac_pulled = false;
            }
        }
    }
}

pub fn shred_particle_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut particles: Query<(Entity, &mut ShredParticle, &mut Transform, &mut Sprite)>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;
    for (e, mut p, mut tf, mut sprite) in &mut particles {
        p.life -= dt;
        if p.life <= 0.0 {
            commands.entity(e).despawn_recursive();
            continue;
        }
        tf.translation.x += p.velocity.x * dt;
        tf.translation.y += p.velocity.y * dt;
        sprite.color = sprite.color.with_alpha((p.life / 0.35).clamp(0.0, 1.0));
    }
}
