use bevy::prelude::*;

use crate::core::events::{BloonPoppedEvent, SpawnBloonEvent};
use crate::core::state::MatchSettings;
use crate::core::types::{BloonLayer, Health, Side, TowerType};
use crate::data::bloons_cfg::bloon_cfg;
use crate::plugins::bloons::Bloon;

use super::Tower;

#[derive(Component, Debug, Clone)]
pub struct UnitExplosionFx {
    pub life: f32,
    pub max_life: f32,
    pub target_size: f32,
    pub color: Color,
}

pub fn children_layers_total(layer: BloonLayer) -> i32 {
    match bloon_cfg(layer).child {
        None => 0,
        Some((cl, cnt)) => (cnt as i32) * (bloon_cfg(cl).hp + children_layers_total(cl)),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn apply_multi_layer_damage(
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
    if damage <= 0 || hp.current <= 0 {
        return;
    }
    if hp.current > damage {
        hp.current -= damage;
        let mut bonus_cash = 0;
        if let Some(t) = tower {
            let prev_tens = t.pops / 10;
            t.pops += damage as u32;
            let new_tens = t.pops / 10;
            if t.tower_type == TowerType::CrookBoss && t.path1 >= 4 && new_tens > prev_tens {
                bonus_cash = ((new_tens - prev_tens) as i32) * 15;
            }
        }
        pop_writer.send(BloonPoppedEvent {
            side,
            cash_gain: bonus_cash,
            pops: damage as u32,
        });
        return;
    }

    let mut rem = damage - hp.current;
    let mut total_pops = hp.current as u32;
    let mut total_cash = 1_i32;
    hp.current = 0;
    commands.entity(bloon_entity).despawn_recursive();

    let mut survivors: Vec<(BloonLayer, u8)> = Vec::new();
    let mut stack: Vec<(BloonLayer, u8)> = Vec::new();
    if let Some((cl, cnt)) = bloon_cfg(bloon.layer).child {
        stack.push((cl, cnt));
    }

    while let Some((layer, count)) = stack.pop() {
        let lhp = bloon_cfg(layer).hp;
        for idx in 0..count {
            if rem >= lhp && !layer.is_moab_class() {
                rem -= lhp;
                total_pops += lhp as u32;
                total_cash += 1;
                if let Some((sub_l, sub_c)) = bloon_cfg(layer).child {
                    stack.push((sub_l, sub_c));
                }
            } else {
                survivors.push((layer, count - idx));
                rem = 0;
                break;
            }
        }
    }

    let mut bonus_cash = 0;
    if let Some(t) = tower {
        let prev_tens = t.pops / 10;
        t.pops += total_pops;
        let new_tens = t.pops / 10;
        if t.tower_type == TowerType::CrookBoss && t.path1 >= 4 && new_tens > prev_tens {
            bonus_cash = ((new_tens - prev_tens) as i32) * 15;
        }
    }
    pop_writer.send(BloonPoppedEvent {
        side,
        cash_gain: total_cash + bonus_cash,
        pops: total_pops,
    });

    let mut spawn_i = 0_usize;
    for (layer, cnt) in survivors {
        for _ in 0..cnt {
            spawn_writer.send(SpawnBloonEvent {
                side,
                layer,
                camo: bloon.camo,
                lead: false,
                path_progress: (bloon.path_progress - (spawn_i as f32) * 10.0).max(0.0),
                from_send: bloon.from_send,
                permafrost: bloon.permafrost,
                regrab_immune_tower: bloon.regrab_immune_tower,
                regrab_cooldown: bloon.regrab_cooldown,
            });
            spawn_i += 1;
        }
    }
}

pub fn unit_explosion_fx_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut fx_q: Query<(Entity, &mut UnitExplosionFx, &mut Sprite)>,
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
        sprite.custom_size = Some(Vec2::splat(16.0 + (fx.target_size - 16.0) * t));
        sprite.color = fx.color.with_alpha(0.85 * (1.0 - t));
    }
}
