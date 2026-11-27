use bevy::prelude::*;

use crate::core::events::{BloonPoppedEvent, SpawnBloonEvent};
use crate::core::state::MatchSettings;
use crate::core::types::{BloonLayer, Health, Side};
use crate::data::bloons_cfg::bloon_cfg;
use crate::plugins::bloons::Bloon;

use super::Tower;

#[derive(Component, Debug, Clone)]
pub struct SpikeExplosionFx {
    pub life: f32,
    pub max_life: f32,
    pub target_size: f32,
}

pub fn spike_explosion_fx_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut fx_q: Query<(Entity, &mut SpikeExplosionFx, &mut Sprite)>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;
    for (e, mut fx, mut sprite) in &mut fx_q {
        fx.life -= dt;
        if fx.life <= 0.0 {
            commands.entity(e).despawn_recursive();
            continue;
        }
        let t = 1.0 - (fx.life / fx.max_life).clamp(0.0, 1.0);
        let size = 16.0 + (fx.target_size - 16.0) * t;
        sprite.custom_size = Some(Vec2::splat(size));
        sprite.color = Color::srgba(0.98, 0.36, 0.08, 0.80 * (1.0 - t));
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_spike_damage(
    commands: &mut Commands,
    bloon_entity: Entity,
    side: Side,
    bloon: &Bloon,
    hp: &mut Health,
    damage: i32,
    mut tower: Option<&mut Tower>,
    spawn_writer: &mut EventWriter<SpawnBloonEvent>,
    pop_writer: &mut EventWriter<BloonPoppedEvent>,
) {
    if damage <= 0 || hp.current <= 0 {
        return;
    }
    if hp.current > damage {
        hp.current -= damage;
        if let Some(ref mut t) = tower {
            t.pops += damage as u32;
        }
        pop_writer.send(BloonPoppedEvent {
            side,
            cash_gain: 0,
            pops: damage as u32,
        });
        return;
    }

    let mut rem_dmg = damage - hp.current;
    let mut total_pops = hp.current as u32;
    let mut total_cash = 1_i32;
    hp.current = 0;

    commands.entity(bloon_entity).despawn_recursive();

    let mut spawns: Vec<(BloonLayer, u8)> = Vec::new();
    if let Some((child_layer, count)) = bloon_cfg(bloon.layer).child {
        let mut cur_layer = child_layer;
        let mut cur_count = count;
        loop {
            let child_hp = bloon_cfg(cur_layer).hp;
            if rem_dmg >= child_hp && !cur_layer.is_moab_class() {
                rem_dmg -= child_hp;
                total_pops += child_hp as u32;
                total_cash += 1;
                if cur_count > 1 {
                    spawns.push((cur_layer, cur_count - 1));
                }
                if let Some((next_layer, next_count)) = bloon_cfg(cur_layer).child {
                    cur_layer = next_layer;
                    cur_count = next_count;
                } else {
                    break;
                }
            } else {
                spawns.push((cur_layer, cur_count));
                break;
            }
        }
    }

    if let Some(t) = tower {
        t.pops += total_pops;
    }
    pop_writer.send(BloonPoppedEvent {
        side,
        cash_gain: total_cash,
        pops: total_pops,
    });

    let mut spawn_idx = 0usize;
    for (layer, count) in spawns {
        for _ in 0..count {
            spawn_writer.send(SpawnBloonEvent {
                side,
                layer,
                camo: bloon.camo,
                lead: false,
                path_progress: (bloon.path_progress - (spawn_idx as f32) * 10.0).max(0.0),
                from_send: bloon.from_send,
                permafrost: bloon.permafrost,
                regrab_immune_tower: bloon.regrab_immune_tower,
                regrab_cooldown: bloon.regrab_cooldown,
            });
            spawn_idx += 1;
        }
    }
}
