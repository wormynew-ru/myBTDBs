use bevy::prelude::*;

use crate::core::events::{BloonPoppedEvent, SpawnBloonEvent};
use crate::core::state::MatchSettings;
use crate::core::types::{BloonLayer, Health, MatchEntity, Position, Side, TargetPriority};
use crate::data::bloons_cfg::bloon_cfg;
use crate::data::maps_cfg::waypoints_for_side;
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats};
use crate::plugins::bloons::Bloon;
use crate::plugins::sprites::SpriteAssets;
use crate::utils::sample_path;

use super::{Tower, TowerBuff};

#[derive(Debug, Clone)]
pub struct PulledSlot {
    pub bloon: Entity,
    pub weight: u32,
    pub is_moab: bool,
}

#[derive(Debug, Clone)]
pub struct ShredSlot {
    pub bloon: Entity,
    pub weight: u32,
    pub is_moab: bool,
    pub timer: f32,
    pub moab_tick: f32,
    pub color: Color,
}

#[derive(Component, Debug, Default, Clone)]
pub struct ChipperState {
    pub pulling: Vec<PulledSlot>,
    pub shredding: Vec<ShredSlot>,
    pub supa_vac_cooldown: f32,
    pub supa_vac_active: f32,
    pub particle_timer: f32,
    pub vib_phase: f32,
}

impl ChipperState {
    pub fn current_load(&self) -> u32 {
        self.pulling.iter().map(|s| s.weight).sum::<u32>()
            + self.shredding.iter().map(|s| s.weight).sum::<u32>()
    }
    pub fn has_moab(&self) -> bool {
        self.pulling.iter().any(|s| s.is_moab) || self.shredding.iter().any(|s| s.is_moab)
    }
}

#[derive(Component)]
pub struct ShredParticle {
    pub velocity: Vec2,
    pub life: f32,
}

fn strip_normal_layer(layer: BloonLayer, layers: u8) -> Option<(BloonLayer, u8)> {
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

#[allow(clippy::too_many_arguments)]
pub fn chipper_fsm_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    sprites: Res<SpriteAssets>,
    mut chippers: Query<(Entity, &Side, &mut Tower, &TowerBuff, &Position, &mut Transform, &mut ChipperState)>,
    mut bloons: Query<(Entity, &Side, &mut Bloon, &mut Health, &mut Position, &mut Transform, &mut Visibility), Without<Tower>>,
    mut spawn_writer: EventWriter<SpawnBloonEvent>,
    mut pop_writer: EventWriter<BloonPoppedEvent>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;
    let player_wps = waypoints_for_side(settings.map_id, Side::Player);
    let bot_wps = waypoints_for_side(settings.map_id, Side::Bot);

    for (t_entity, t_side, mut tower, buff, t_pos, mut t_tf, mut chipper) in &mut chippers {
        let stats = apply_village_buff(
            compute_effective_stats(tower.tower_type, tower.path1, tower.path2),
            buff.0,
        );

        // Clean up any pulled bloons popped mid-pull by other towers
        chipper.pulling.retain(|slot| bloons.get(slot.bloon).is_ok());

        // 1. SCANNING: fill available barrel slots & buffer capacity
        while chipper.pulling.len() < stats.barrels && chipper.current_load() < stats.max_capacity {
            let load = chipper.current_load();
            let has_moab = chipper.has_moab();
            let mut best: Option<(Entity, Vec2, u32, bool, f32)> = None;

            for (b_entity, b_side, b, _, b_pos, _, _) in bloons.iter() {
                if *b_side != *t_side
                    || b.is_pulled
                    || b.is_inside_chipper
                    || b.supa_vac_pulled
                    || b.regrab_immune_tower == Some(t_entity)
                    || (b.camo && !stats.can_hit_camo)
                    || b.layer.is_zomg()
                {
                    continue;
                }
                let is_moab = b.layer.is_moab_class();
                if is_moab && (!stats.can_suck_moab || has_moab) {
                    continue;
                }
                let is_heavy = b.lead || b.layer == BloonLayer::Ceramic;
                if is_heavy && !stats.can_suck_heavy {
                    continue;
                }
                let weight = bloon_cfg(b.layer).weight;
                if load + weight > stats.max_capacity {
                    continue;
                }
                let d = t_pos.0.distance(b_pos.0);
                if d > stats.range {
                    continue;
                }
                let score = match tower.priority {
                    TargetPriority::First => b.path_progress,
                    TargetPriority::Last => -b.path_progress,
                    TargetPriority::Close => -d,
                    TargetPriority::Strong => (bloon_cfg(b.layer).rbe as f32) * 10000.0 + b.path_progress,
                };
                if best.map_or(true, |(_, _, _, _, bs)| score > bs) {
                    best = Some((b_entity, b_pos.0, weight, is_moab, score));
                }
            }

            let Some((target_e, target_pos, weight, is_moab, _)) = best else {
                break;
            };
            if let Ok((_, _, mut b, _, _, _, _)) = bloons.get_mut(target_e) {
                b.is_pulled = true;
                let dir = (target_pos - t_pos.0).normalize_or(Vec2::X);
                t_tf.rotation = Quat::from_rotation_z(dir.y.atan2(dir.x));
                chipper.pulling.push(PulledSlot { bloon: target_e, weight, is_moab });
            } else {
                break;
            }
        }

        // 2. PULLING -> transition to SHREDDING
        let forward = (t_tf.rotation * Vec3::X).truncate().normalize_or(Vec2::X);
        let funnel_pos = t_pos.0 + forward * 12.0;
        let mut still_pulling = Vec::new();
        for slot in chipper.pulling.drain(..) {
            let Ok((_, _, mut b, _, mut b_pos, mut b_tf, mut b_vis)) = bloons.get_mut(slot.bloon) else {
                continue;
            };
            let to_funnel = funnel_pos - b_pos.0;
            let dist = to_funnel.length();
            if dist <= 14.0 {
                b.is_pulled = false;
                b.is_inside_chipper = true;
                *b_vis = Visibility::Hidden;
                let cfg = bloon_cfg(b.layer);
                chipper.shredding.push(ShredSlot {
                    bloon: slot.bloon,
                    weight: slot.weight,
                    is_moab: slot.is_moab,
                    timer: if slot.is_moab { 1.35 } else { stats.shred_duration },
                    moab_tick: 0.25,
                    color: if b.lead { Color::srgb(0.42, 0.48, 0.56) } else { cfg.color },
                });
            } else {
                let step = (stats.pull_speed * dt).min(dist);
                b_pos.0 += (to_funnel / dist) * step;
                b_tf.translation.x = b_pos.0.x;
                b_tf.translation.y = b_pos.0.y;
                still_pulling.push(slot);
            }
        }
        chipper.pulling = still_pulling;

        // 3. SHREDDING & 4. EJECTING + Vibration + Exhaust Particles
        let mut still_shredding = Vec::new();
        let wps = if *t_side == Side::Player { &player_wps } else { &bot_wps };

        for mut slot in chipper.shredding.drain(..) {
            let Ok((b_e, _, mut b, mut hp, mut b_pos, mut b_tf, mut b_vis)) = bloons.get_mut(slot.bloon) else {
                continue;
            };
            slot.timer -= dt;

            if slot.is_moab {
                slot.moab_tick -= dt;
                if slot.moab_tick <= 0.0 {
                    slot.moab_tick += 0.25;
                    let dmg = 18;
                    let dealt = hp.current.min(dmg).max(0);
                    hp.current -= dmg;
                    tower.pops += dealt as u32;
                    pop_writer.send(BloonPoppedEvent { side: *t_side, cash_gain: 0, pops: dealt as u32 });
                    if hp.current <= 0 {
                        pop_writer.send(BloonPoppedEvent { side: *t_side, cash_gain: 1, pops: 1 });
                        if let Some((child_layer, count)) = bloon_cfg(b.layer).child {
                            for c in 0..count {
                                spawn_writer.send(SpawnBloonEvent {
                                    side: *t_side,
                                    layer: child_layer,
                                    camo: b.camo,
                                    lead: false,
                                    path_progress: (b.path_progress - 24.0 - (c as f32) * 14.0).max(0.0),
                                    from_send: b.from_send,
                                    permafrost: b.permafrost,
                                    regrab_immune_tower: Some(t_entity),
                                    regrab_cooldown: 0.3,
                                });
                            }
                        }
                        commands.entity(b_e).despawn_recursive();
                        continue;
                    }
                }
                if slot.timer <= 0.0 {
                    b.is_inside_chipper = false;
                    b.path_progress = (b.path_progress - 38.0).max(0.0);
                    b.regrab_immune_tower = Some(t_entity);
                    b.regrab_cooldown = 0.35;
                    let (back_pos, _) = sample_path(wps, b.path_progress);
                    b_pos.0 = back_pos;
                    b_tf.translation.x = back_pos.x;
                    b_tf.translation.y = back_pos.y;
                    *b_vis = Visibility::Visible;
                } else {
                    still_shredding.push(slot);
                }
                continue;
            }

            if slot.timer > 0.0 {
                still_shredding.push(slot);
                continue;
            }

            // Ejecting normal/heavy bloon
            let eject_progress = (b.path_progress - 24.0).max(0.0);
            if b.layer == BloonLayer::Ceramic {
                let dmg = stats.layers_per_shred as i32;
                let dealt = hp.current.min(dmg).max(0);
                hp.current -= dmg;
                tower.pops += dealt as u32;
                if hp.current > 0 {
                    pop_writer.send(BloonPoppedEvent { side: *t_side, cash_gain: 0, pops: dealt as u32 });
                    b.is_inside_chipper = false;
                    b.path_progress = eject_progress;
                    b.regrab_immune_tower = Some(t_entity);
                    b.regrab_cooldown = 0.3;
                    let (back_pos, _) = sample_path(wps, eject_progress);
                    b_pos.0 = back_pos;
                    b_tf.translation.x = back_pos.x;
                    b_tf.translation.y = back_pos.y;
                    *b_vis = Visibility::Visible;
                    continue;
                }
            } else {
                tower.pops += stats.layers_per_shred as u32;
            }

            pop_writer.send(BloonPoppedEvent { side: *t_side, cash_gain: 1, pops: 1 });
            if let Some((child_layer, count)) = strip_normal_layer(b.layer, stats.layers_per_shred) {
                for c in 0..count {
                    spawn_writer.send(SpawnBloonEvent {
                        side: *t_side,
                        layer: child_layer,
                        camo: b.camo,
                        lead: false,
                        path_progress: (eject_progress - (c as f32) * 10.0).max(0.0),
                        from_send: b.from_send,
                        permafrost: b.permafrost,
                        regrab_immune_tower: Some(t_entity),
                        regrab_cooldown: 0.3,
                    });
                }
            }
            commands.entity(b_e).despawn_recursive();
        }
        chipper.shredding = still_shredding;

        // Vibration & exhaust rubber particles when current_load > 0
        if chipper.current_load() > 0 {
            chipper.vib_phase += dt * 45.0;
            t_tf.translation.x = t_pos.0.x + chipper.vib_phase.sin() * 2.2;
            t_tf.translation.y = t_pos.0.y + (chipper.vib_phase * 1.3).cos() * 2.2;

            chipper.particle_timer -= dt;
            if chipper.particle_timer <= 0.0 {
                chipper.particle_timer = 0.09;
                let p_color = chipper
                    .shredding
                    .first()
                    .map(|s| s.color)
                    .unwrap_or(Color::srgb(0.95, 0.35, 0.35));
                let exhaust = t_pos.0 - forward * 12.0;
                let eject_dir = Vec2::new(-forward.x * 38.0, 44.0 + chipper.vib_phase.sin() * 14.0);
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
        } else {
            t_tf.translation.x = t_pos.0.x;
            t_tf.translation.y = t_pos.0.y;
        }
    }
}
