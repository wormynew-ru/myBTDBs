use bevy::prelude::*;

use crate::core::state::MatchSettings;
use crate::core::types::{Position, Side, TowerType};
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats};
use crate::plugins::bloons::Bloon;

use super::chipper::{ChipperState, ShredParticle};
use super::{Tower, TowerBuff};

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
