use bevy::prelude::*;

use crate::core::events::{ActivateAbilityEvent, BloonPoppedEvent, SpawnBloonEvent};
use crate::core::state::MatchSettings;
use crate::core::types::{Health, MatchEntity, Position, Side, TowerType};
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats};
use crate::plugins::bloons::Bloon;
use crate::plugins::sprites::SpriteAssets;

use super::combat_proj::apply_damage;
use super::{Tower, TowerBuff};

#[derive(Component, Debug, Clone)]
pub struct SaboteurState {
    pub ability_cooldown: f32,
    pub pulse_timer: f32,
    pub rng_state: u32,
}

impl Default for SaboteurState {
    fn default() -> Self {
        Self {
            ability_cooldown: 0.0,
            pulse_timer: 0.6,
            rng_state: 0x517C_C1B7,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct SaboteurPulseFx {
    pub life: f32,
    pub max_life: f32,
    pub target_size: f32,
}

#[derive(Component, Debug, Clone)]
pub struct JamStaticFx {
    pub target_tower: Entity,
    pub flicker: f32,
}

fn next_rand(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    *state
}

#[allow(clippy::type_complexity)]
pub fn handle_saboteur_ability(
    time: Res<Time>,
    mut towers: Query<(Entity, &Side, &mut Tower, Option<&mut SaboteurState>)>,
    mut reader: EventReader<ActivateAbilityEvent>,
) {
    let time_bits = time.elapsed_secs().to_bits();
    for ev in reader.read() {
        let Ok((_, side, tower, Some(mut sab_state))) = towers.get_mut(ev.tower_entity) else {
            continue;
        };
        if *side != ev.side
            || tower.tower_type != TowerType::Saboteur
            || sab_state.ability_cooldown > 0.0
        {
            continue;
        }
        let stats = compute_effective_stats(tower.tower_type, tower.path1, tower.path2);
        sab_state.ability_cooldown = stats.saboteur_ability_cd;
        sab_state.rng_state ^= time_bits;
        let mut rng = sab_state.rng_state | 1;
        let jam_dur = stats.saboteur_jam_duration;
        let target_count = stats.saboteur_jam_targets;
        let enemy_side = ev.side.opponent();

        let mut candidates: Vec<(Entity, i32, bool)> = towers
            .iter()
            .filter(|(_, s, t, _)| **s == enemy_side && t.total_invested > 0)
            .map(|(e, _, t, _)| (e, t.total_invested, t.jammed_timer <= 0.0))
            .collect();
        if candidates.is_empty() {
            continue;
        }
        candidates.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| b.1.cmp(&a.1)));

        let mut top_pool: Vec<Entity> = candidates
            .into_iter()
            .take(4)
            .map(|(e, _, _)| e)
            .collect();

        for _ in 0..target_count {
            if top_pool.is_empty() {
                break;
            }
            let idx = (next_rand(&mut rng) as usize) % top_pool.len();
            let chosen = top_pool.swap_remove(idx);
            if let Ok((_, _, mut enemy_t, _)) = towers.get_mut(chosen) {
                enemy_t.jammed_timer = enemy_t.jammed_timer.max(jam_dur);
            }
        }
        sab_state.rng_state = rng;
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn saboteur_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    sprites: Res<SpriteAssets>,
    mut towers: Query<(
        Entity,
        &Side,
        &mut Tower,
        &TowerBuff,
        &Position,
        Option<&mut SaboteurState>,
    )>,
    mut bloons: Query<(Entity, &Side, &mut Bloon, &mut Health, &Position), Without<Tower>>,
    mut pulse_fx: Query<
        (Entity, &mut SaboteurPulseFx, &mut Sprite),
        (Without<Tower>, Without<JamStaticFx>),
    >,
    mut static_fx: Query<
        (Entity, &mut JamStaticFx, &mut Transform, &mut Sprite),
        (Without<Tower>, Without<SaboteurPulseFx>),
    >,
    mut spawn_writer: EventWriter<SpawnBloonEvent>,
    mut pop_writer: EventWriter<BloonPoppedEvent>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;

    for (e, mut fx, mut sprite) in &mut pulse_fx {
        fx.life -= dt;
        if fx.life <= 0.0 {
            commands.entity(e).despawn_recursive();
            continue;
        }
        let t = 1.0 - (fx.life / fx.max_life).clamp(0.0, 1.0);
        sprite.custom_size = Some(Vec2::splat(20.0 + (fx.target_size - 20.0) * t));
        sprite.color = Color::srgba(0.10, 0.85, 0.95, 0.45 * (1.0 - t));
    }

    let mut jammed_towers: Vec<(Entity, Vec2)> = Vec::new();
    for (t_e, t_side, mut tower, buff, t_pos, sab_opt) in &mut towers {
        if tower.jammed_timer > 0.0 {
            tower.jammed_timer = (tower.jammed_timer - dt).max(0.0);
            if tower.jammed_timer > 0.0 {
                jammed_towers.push((t_e, t_pos.0));
            }
        }

        let Some(mut sab) = sab_opt else {
            continue;
        };
        if tower.tower_type != TowerType::Saboteur {
            continue;
        }
        if sab.ability_cooldown > 0.0 {
            sab.ability_cooldown = (sab.ability_cooldown - dt).max(0.0);
        }

        let stats = apply_village_buff(
            compute_effective_stats(tower.tower_type, tower.path1, tower.path2),
            buff.0,
        );
        sab.pulse_timer -= dt;
        if sab.pulse_timer > 0.0 {
            continue;
        }
        let any_in_range = bloons.iter().any(|(_, bs, b, hp, bp)| {
            *bs == *t_side
                && hp.current > 0
                && !b.is_inside_chipper
                && t_pos.0.distance(bp.0) <= stats.range
        });
        if !any_in_range {
            sab.pulse_timer = 0.1;
            continue;
        }
        sab.pulse_timer = 1.0 / stats.fire_rate.max(0.2);

        commands.spawn((
            MatchEntity,
            SaboteurPulseFx {
                life: 0.30,
                max_life: 0.30,
                target_size: stats.range * 2.0,
            },
            Sprite::from_color(Color::srgba(0.10, 0.85, 0.95, 0.45), Vec2::splat(20.0)),
            Transform::from_xyz(t_pos.0.x, t_pos.0.y, 4.4),
        ));

        for (b_e, bs, mut bloon, mut hp, bp) in &mut bloons {
            if *bs != *t_side
                || hp.current <= 0
                || bloon.is_inside_chipper
                || t_pos.0.distance(bp.0) > stats.range
            {
                continue;
            }
            if stats.saboteur_slow < 1.0 {
                bloon.freeze_timer = bloon.freeze_timer.max(1.4);
                bloon.slow_factor = bloon.slow_factor.min(stats.saboteur_slow);
            }
            if !bloon.lead || stats.can_pop_lead {
                apply_damage(
                    &mut commands,
                    b_e,
                    *t_side,
                    &bloon,
                    bp.0,
                    &mut hp,
                    stats.damage,
                    Some(&mut tower),
                    &mut spawn_writer,
                    &mut pop_writer,
                );
            }
        }
    }

    let mut tracked_entities: Vec<Entity> = Vec::new();
    for (fx_e, mut fx, mut tf, mut sprite) in &mut static_fx {
        if let Some((_, pos)) = jammed_towers.iter().find(|(te, _)| *te == fx.target_tower) {
            fx.flicker += dt * 14.0;
            tf.translation.x = pos.x + (fx.flicker.sin() * 2.0);
            tf.translation.y = pos.y + 8.0;
            sprite.color = Color::WHITE.with_alpha(0.65 + 0.35 * fx.flicker.cos().abs());
            tracked_entities.push(fx.target_tower);
        } else {
            commands.entity(fx_e).despawn_recursive();
        }
    }

    for (te, pos) in jammed_towers {
        if !tracked_entities.contains(&te) {
            commands.spawn((
                MatchEntity,
                JamStaticFx {
                    target_tower: te,
                    flicker: 0.0,
                },
                Sprite {
                    image: sprites.jam_static.clone(),
                    custom_size: Some(Vec2::splat(36.0)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y + 8.0, 10.5),
            ));
        }
    }
}
