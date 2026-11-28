use bevy::prelude::*;

use crate::core::events::BloonPoppedEvent;
use crate::core::state::MatchSettings;
use crate::core::types::{BloonLayer, GlueStatus, Health, MatchEntity, Position, Side, TowerType};
use crate::data::maps_cfg::waypoints_for_side;
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats};
use crate::plugins::bloons::{Bloon, GlueOverlay};
use crate::plugins::sprites::SpriteAssets;
use crate::utils::sample_path;

use super::{Tower, TowerBuff};

#[derive(Component, Debug, Clone)]
pub struct NecromancerState {
    pub souls: u32,
    pub rift_timer: f32,
}

impl Default for NecromancerState {
    fn default() -> Self {
        Self {
            souls: 0,
            rift_timer: 10.0,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct RiftPortalFx {
    pub life: f32,
    pub max_life: f32,
}

#[allow(clippy::too_many_arguments)]
pub fn necromancer_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    sprites: Res<SpriteAssets>,
    mut necros: Query<(Entity, &Side, &Tower, &TowerBuff, &Position, &mut NecromancerState)>,
    mut fx_q: Query<(Entity, &mut RiftPortalFx, &mut Sprite, &mut Transform)>,
    mut pop_reader: EventReader<BloonPoppedEvent>,
) {
    if settings.paused {
        pop_reader.clear();
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;

    for (e, mut fx, mut sprite, mut tf) in &mut fx_q {
        fx.life -= dt;
        if fx.life <= 0.0 {
            commands.entity(e).despawn_recursive();
            continue;
        }
        tf.rotate_z(3.2 * dt);
        let alpha = (fx.life / fx.max_life).clamp(0.0, 1.0);
        sprite.color = Color::WHITE.with_alpha(alpha);
    }

    for ev in pop_reader.read() {
        if !ev.killed {
            continue;
        }
        let mut best: Option<(Entity, f32)> = None;
        for (e, side, tower, buff, pos, state) in necros.iter() {
            if *side != ev.side || tower.tower_type != TowerType::Necromancer {
                continue;
            }
            let stats = apply_village_buff(
                compute_effective_stats(tower.tower_type, tower.path1, tower.path2),
                buff.0,
            );
            if state.souls >= stats.soul_capacity {
                continue;
            }
            let d = pos.0.distance(ev.pos);
            if d <= stats.range && best.map_or(true, |(_, bd)| d < bd) {
                best = Some((e, d));
            }
        }
        if let Some((e, _)) = best {
            if let Ok((_, _, tower, buff, _, mut state)) = necros.get_mut(e) {
                let stats = apply_village_buff(
                    compute_effective_stats(tower.tower_type, tower.path1, tower.path2),
                    buff.0,
                );
                state.souls = (state.souls + 1).min(stats.soul_capacity);
            }
        }
    }

    let player_wps = waypoints_for_side(settings.map_id, Side::Player);
    let bot_wps = waypoints_for_side(settings.map_id, Side::Bot);

    for (_, side, tower, buff, _, mut state) in &mut necros {
        if tower.tower_type != TowerType::Necromancer {
            continue;
        }
        let stats = apply_village_buff(
            compute_effective_stats(tower.tower_type, tower.path1, tower.path2),
            buff.0,
        );
        if state.rift_timer > stats.rift_interval {
            state.rift_timer = stats.rift_interval;
        }
        state.rift_timer -= dt;

        let ready = state.souls > 0 && (state.rift_timer <= 0.0 || state.souls >= stats.soul_capacity);
        if !ready {
            continue;
        }
        state.rift_timer = stats.rift_interval;
        let count = state.souls;
        state.souls = 0;

        let target_side = side.opponent();
        let wps = if target_side == Side::Player { &player_wps } else { &bot_wps };
        let rift_prog = 32.0;
        let (rift_pos, _) = sample_path(wps, rift_prog);

        commands.spawn((
            MatchEntity,
            RiftPortalFx { life: 1.4, max_life: 1.4 },
            Sprite {
                image: sprites.rift_portal.clone(),
                custom_size: Some(Vec2::splat(38.0)),
                ..default()
            },
            Transform::from_xyz(rift_pos.x, rift_pos.y, 7.5),
        ));

        let ghost_speed = 135.0 * stats.ghost_speed_mult;
        for i in 0..count {
            let prog = (rift_prog + (i as f32) * 3.5).max(0.0);
            let (g_pos, _) = sample_path(wps, prog);
            spawn_ghost_unit(
                &mut commands,
                &sprites,
                target_side,
                g_pos,
                prog,
                stats.ghost_hp,
                ghost_speed,
                1,
                stats.ghost_camo,
                stats.ghost_lead,
                false,
            );
        }

        if stats.summon_ghost_knight {
            spawn_ghost_unit(
                &mut commands,
                &sprites,
                target_side,
                rift_pos,
                rift_prog,
                140,
                105.0 * stats.ghost_speed_mult,
                15,
                stats.ghost_camo,
                true,
                true,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_ghost_unit(
    commands: &mut Commands,
    sprites: &SpriteAssets,
    side: Side,
    pos: Vec2,
    path_progress: f32,
    hp: i32,
    speed: f32,
    life_loss: i32,
    camo: bool,
    lead: bool,
    is_knight: bool,
) {
    let size = if is_knight { 34.0 } else { 24.0 };
    let img = if is_knight {
        sprites.ghost_knight.clone()
    } else {
        sprites.ghost_bloon.clone()
    };
    commands
        .spawn((
            MatchEntity,
            side,
            Bloon {
                layer: if is_knight { BloonLayer::Ceramic } else { BloonLayer::Pink },
                camo,
                lead,
                path_progress,
                freeze_timer: 0.0,
                slow_factor: 1.0,
                speed_mult: 1.0,
                permafrost: false,
                glue: GlueStatus::default(),
                from_send: false,
                is_ghost: true,
                ghost_life_loss: life_loss,
                is_mirage: false,
                mirage_flash_radius: 0.0,
                mirage_flash_duration: 0.0,
                base_speed_override: Some(speed),
                is_pulled: false,
                is_inside_chipper: false,
                supa_vac_pulled: false,
                regrab_immune_tower: None,
                regrab_cooldown: 0.0,
            },
            Health { current: hp, max: hp },
            Position(pos),
            Sprite {
                image: img,
                color: Color::WHITE,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, if is_knight { 6.4 } else { 6.1 }),
        ))
        .with_children(|parent| {
            if camo {
                parent.spawn((
                    Sprite {
                        image: sprites.overlay_camo.clone(),
                        custom_size: Some(Vec2::splat(size)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 0.2),
                ));
            }
            parent.spawn((
                GlueOverlay,
                Sprite {
                    image: sprites.overlay_glue.clone(),
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.3),
                Visibility::Hidden,
            ));
        });
}
