use bevy::prelude::*;

use crate::core::events::{BloonPoppedEvent, SpawnBloonEvent};
use crate::core::state::MatchSettings;
use crate::core::types::{
    BloonLayer, Health, MatchEntity, Position, Side, SidesResource, TowerType,
};
use crate::data::bloons_cfg::bloon_cfg;
use crate::data::maps_cfg::waypoints_for_side;
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats, EffectiveStats};
use crate::plugins::bloons::Bloon;
use crate::plugins::sprites::SpriteAssets;
use crate::utils::{circles_overlap, path_total_length, sample_path};

use super::spikes_fx::{apply_spike_damage, SpikeExplosionFx};
use super::{Tower, TowerBuff};

#[derive(Component, Debug, Clone)]
pub struct SpikeFactoryState {
    pub storm_timer: f32,
    pub rng_state: u32,
}

impl Default for SpikeFactoryState {
    fn default() -> Self {
        Self {
            storm_timer: 0.5,
            rng_state: 0x9E37_79B9,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct SpikePile {
    pub source_tower: Entity,
    pub damage: i32,
    pub ceramic_bonus: i32,
    pub pierce_left: u32,
    pub max_pierce: u32,
    pub can_pop_lead: bool,
    pub can_hit_camo: bool,
    pub life: f32,
    pub persist_waves: bool,
    pub explode_on_expire: bool,
    pub explosion_radius: f32,
    pub base_size: f32,
    pub base_color: Color,
}

fn next_rand(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    *state
}

fn rand_signed(state: &mut u32) -> f32 {
    let v = (next_rand(state) >> 16) as f32 / 65535.0;
    v * 2.0 - 1.0
}

fn spawn_spike_pile(
    commands: &mut Commands,
    sprites: &SpriteAssets,
    side: Side,
    pos: Vec2,
    source_tower: Entity,
    stats: &EffectiveStats,
    tower_path1: u8,
) {
    let is_hot = stats.can_pop_lead || tower_path1 >= 2;
    let base_color = if tower_path1 >= 4 {
        Color::srgb(0.98, 0.35, 0.08)
    } else if is_hot {
        Color::srgb(0.976, 0.451, 0.086)
    } else {
        Color::WHITE
    };
    let base_size = if tower_path1 >= 3 { 24.0 } else { 20.0 };

    commands.spawn((
        MatchEntity,
        side,
        Position(pos),
        SpikePile {
            source_tower,
            damage: stats.damage,
            ceramic_bonus: stats.ceramic_bonus_damage,
            pierce_left: stats.pierce,
            max_pierce: stats.pierce.max(1),
            can_pop_lead: stats.can_pop_lead,
            can_hit_camo: stats.can_hit_camo,
            life: stats.spike_life,
            persist_waves: stats.spike_persist_waves,
            explode_on_expire: stats.spike_explode,
            explosion_radius: stats.splash_radius,
            base_size,
            base_color,
        },
        Sprite {
            image: sprites.spike_pile.clone(),
            color: base_color,
            custom_size: Some(Vec2::splat(base_size)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 4.5),
    ));
}

pub fn spike_factory_produce_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    sprites: Res<SpriteAssets>,
    mut factories: Query<(
        Entity,
        &Side,
        &mut Tower,
        &TowerBuff,
        &Position,
        &mut SpikeFactoryState,
    )>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;
    let player_wps = waypoints_for_side(settings.map_id, Side::Player);
    let bot_wps = waypoints_for_side(settings.map_id, Side::Bot);
    let player_len = path_total_length(&player_wps);
    let bot_len = path_total_length(&bot_wps);

    for (t_entity, t_side, mut tower, buff, pos, mut state) in &mut factories {
        if tower.tower_type != TowerType::SpikeFactory {
            continue;
        }
        let stats = apply_village_buff(
            compute_effective_stats(tower.tower_type, tower.path1, tower.path2),
            buff.0,
        );
        let (wps, total_len) = match t_side {
            Side::Player => (&player_wps, player_len),
            Side::Bot => (&bot_wps, bot_len),
        };

        tower.cooldown -= dt;
        if tower.cooldown <= 0.0 {
            tower.cooldown += 1.0 / stats.fire_rate.max(0.1);
            let mut road_pts = Vec::new();
            let mut d = 0.0;
            while d <= total_len {
                let (pt, _) = sample_path(wps, d);
                if pt.distance(pos.0) <= stats.range {
                    road_pts.push(pt);
                }
                d += 8.0;
            }
            if !road_pts.is_empty() {
                let idx = (next_rand(&mut state.rng_state) as usize) % road_pts.len();
                let jx = rand_signed(&mut state.rng_state) * 5.0;
                let jy = rand_signed(&mut state.rng_state) * 5.0;
                spawn_spike_pile(
                    &mut commands,
                    &sprites,
                    *t_side,
                    road_pts[idx] + Vec2::new(jx, jy),
                    t_entity,
                    &stats,
                    tower.path1,
                );
            }
        }

        if stats.has_spike_storm {
            state.storm_timer -= dt;
            if state.storm_timer <= 0.0 {
                state.storm_timer += 28.0;
                let mut d = 18.0;
                let end = (total_len - 18.0).max(18.0);
                while d <= end {
                    let (pt, _) = sample_path(wps, d);
                    let jx = rand_signed(&mut state.rng_state) * 4.0;
                    let jy = rand_signed(&mut state.rng_state) * 4.0;
                    spawn_spike_pile(
                        &mut commands,
                        &sprites,
                        *t_side,
                        pt + Vec2::new(jx, jy),
                        t_entity,
                        &stats,
                        tower.path1,
                    );
                    d += 46.0;
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn spike_piles_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    sides: Res<SidesResource>,
    mut last_wave: Local<u32>,
    mut piles: Query<(Entity, &Side, &Position, &mut SpikePile, &mut Sprite), (Without<Bloon>, Without<Tower>)>,
    mut bloons: Query<
        (Entity, &Side, &mut Bloon, &mut Health, &Position),
        (Without<SpikePile>, Without<Tower>),
    >,
    mut towers: Query<&mut Tower, (Without<SpikePile>, Without<Bloon>)>,
    mut spawn_writer: EventWriter<SpawnBloonEvent>,
    mut pop_writer: EventWriter<BloonPoppedEvent>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;

    let wave_ended = if *last_wave == 0 || sides.wave < *last_wave {
        *last_wave = sides.wave;
        false
    } else if sides.wave > *last_wave {
        *last_wave = sides.wave;
        true
    } else {
        false
    };

    for (p_entity, p_side, p_pos, mut pile, mut sprite) in &mut piles {
        pile.life -= dt;
        let wave_expired = wave_ended && !pile.persist_waves;

        if pile.life > 0.0 && !wave_expired && pile.pierce_left > 0 {
            for (b_entity, b_side, bloon, mut hp, b_pos) in &mut bloons {
                if *b_side != *p_side
                    || hp.current <= 0
                    || bloon.is_inside_chipper
                    || (bloon.camo && !pile.can_hit_camo)
                    || (bloon.lead && !pile.can_pop_lead)
                {
                    continue;
                }
                let b_rad = bloon_cfg(bloon.layer).size * 0.5;
                if !circles_overlap(p_pos.0, 12.0, b_pos.0, b_rad) {
                    continue;
                }

                let dmg = pile.damage
                    + if bloon.layer == BloonLayer::Ceramic {
                        pile.ceramic_bonus
                    } else {
                        0
                    };
                let mut src_tower = towers.get_mut(pile.source_tower).ok();
                apply_spike_damage(
                    &mut commands,
                    b_entity,
                    *p_side,
                    &bloon,
                    b_pos.0,
                    &mut hp,
                    dmg,
                    src_tower.as_deref_mut(),
                    &mut spawn_writer,
                    &mut pop_writer,
                );
                pile.pierce_left = pile.pierce_left.saturating_sub(1);
                if pile.pierce_left == 0 {
                    break;
                }
            }
        }

        if pile.pierce_left == 0 || pile.life <= 0.0 || wave_expired {
            if pile.explode_on_expire {
                commands.spawn((
                    MatchEntity,
                    SpikeExplosionFx {
                        life: 0.26,
                        max_life: 0.26,
                        target_size: pile.explosion_radius * 2.0,
                    },
                    Sprite::from_color(
                        Color::srgba(0.98, 0.36, 0.08, 0.85),
                        Vec2::splat(16.0),
                    ),
                    Transform::from_xyz(p_pos.0.x, p_pos.0.y, 8.5),
                ));

                let mut src_tower = towers.get_mut(pile.source_tower).ok();
                for (b_entity, b_side, bloon, mut hp, b_pos) in &mut bloons {
                    if *b_side != *p_side || hp.current <= 0 || bloon.is_inside_chipper {
                        continue;
                    }
                    let b_rad = bloon_cfg(bloon.layer).size * 0.5;
                    if circles_overlap(p_pos.0, pile.explosion_radius, b_pos.0, b_rad) {
                        apply_spike_damage(
                            &mut commands,
                            b_entity,
                            *p_side,
                            &bloon,
                            b_pos.0,
                            &mut hp,
                            8,
                            src_tower.as_deref_mut(),
                            &mut spawn_writer,
                            &mut pop_writer,
                        );
                    }
                }
            }
            commands.entity(p_entity).despawn_recursive();
            continue;
        }

        let ratio = (pile.pierce_left as f32 / pile.max_pierce as f32).clamp(0.25, 1.0);
        let scale = 0.55 + 0.45 * ratio;
        sprite.custom_size = Some(Vec2::splat(pile.base_size * scale));
        sprite.color = pile.base_color.with_alpha(0.42 + 0.58 * ratio);
    }
}
