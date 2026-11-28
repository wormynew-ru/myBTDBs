use bevy::prelude::*;

use crate::core::events::BloonPoppedEvent;
use crate::core::state::MatchSettings;
use crate::core::types::{
    BloonLayer, GlueStatus, Health, MatchEntity, Position, Side, SidesResource, TowerType,
};
use crate::data::bloons_cfg::bloon_cfg;
use crate::data::maps_cfg::waypoints_for_side;
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats};
use crate::plugins::bloons::{Bloon, GlueOverlay};
use crate::plugins::sprites::SpriteAssets;
use crate::utils::sample_path;

use super::{Tower, TowerBuff};

#[derive(Component, Debug, Clone)]
pub struct MirageGeneratorState {
    pub spawn_timer: f32,
}

impl Default for MirageGeneratorState {
    fn default() -> Self {
        Self { spawn_timer: 4.0 }
    }
}

#[derive(Component, Debug, Clone)]
pub struct MirageFlashFx {
    pub life: f32,
    pub max_life: f32,
    pub target_size: f32,
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn mirage_generator_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    sides: Res<SidesResource>,
    sprites: Res<SpriteAssets>,
    mut generators: Query<(&Side, &Tower, &TowerBuff, &mut MirageGeneratorState)>,
    mut all_towers: Query<(&Side, &mut Tower, &Position), Without<MirageGeneratorState>>,
    mut fx_q: Query<(Entity, &mut MirageFlashFx, &mut Sprite)>,
    mut pop_reader: EventReader<BloonPoppedEvent>,
) {
    if settings.paused {
        pop_reader.clear();
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
        sprite.custom_size = Some(Vec2::splat(20.0 + (fx.target_size - 20.0) * t));
        sprite.color = Color::srgba(0.60, 0.96, 1.0, 0.72 * (1.0 - t));
    }

    for ev in pop_reader.read() {
        let Some((flash_rad, flash_dur)) = ev.mirage_flash else {
            continue;
        };
        commands.spawn((
            MatchEntity,
            MirageFlashFx {
                life: 0.40,
                max_life: 0.40,
                target_size: flash_rad * 2.0,
            },
            Sprite::from_color(Color::srgba(0.60, 0.96, 1.0, 0.72), Vec2::splat(20.0)),
            Transform::from_xyz(ev.pos.x, ev.pos.y, 9.5),
        ));
        for (t_side, mut tower, t_pos) in &mut all_towers {
            if *t_side == ev.side && t_pos.0.distance(ev.pos) <= flash_rad {
                tower.jammed_timer = tower.jammed_timer.max(flash_dur);
            }
        }
    }

    let player_wps = waypoints_for_side(settings.map_id, Side::Player);
    let bot_wps = waypoints_for_side(settings.map_id, Side::Bot);

    for (side, tower, buff, mut state) in &mut generators {
        if tower.tower_type != TowerType::MirageGenerator {
            continue;
        }
        let stats = apply_village_buff(
            compute_effective_stats(tower.tower_type, tower.path1, tower.path2),
            buff.0,
        );
        if state.spawn_timer > stats.mirage_interval {
            state.spawn_timer = stats.mirage_interval;
        }
        state.spawn_timer -= dt;
        if state.spawn_timer > 0.0 {
            continue;
        }
        state.spawn_timer += stats.mirage_interval;

        let target_side = side.opponent();
        let wps = if target_side == Side::Player {
            &player_wps
        } else {
            &bot_wps
        };
        let layer = if stats.mirage_force_zomg || sides.wave >= 21 {
            BloonLayer::Zomg
        } else if sides.wave >= 17 {
            BloonLayer::Bfb
        } else {
            BloonLayer::Moab
        };
        let cfg = bloon_cfg(layer);

        for i in 0..stats.mirage_count {
            let prog = (i as f32) * 28.0;
            let (pos, _) = sample_path(wps, prog);
            commands
                .spawn((
                    MatchEntity,
                    target_side,
                    Bloon {
                        layer,
                        camo: false,
                        lead: false,
                        path_progress: prog,
                        freeze_timer: 0.0,
                        slow_factor: 1.0,
                        speed_mult: 1.0,
                        permafrost: false,
                        glue: GlueStatus::default(),
                        from_send: false,
                        is_ghost: false,
                        ghost_life_loss: 0,
                        is_mirage: true,
                        mirage_flash_radius: stats.mirage_flash_radius,
                        mirage_flash_duration: stats.mirage_flash_duration,
                        base_speed_override: Some(cfg.speed * stats.mirage_speed_mult),
                        is_pulled: false,
                        is_inside_chipper: false,
                        supa_vac_pulled: false,
                        regrab_immune_tower: None,
                        regrab_cooldown: 0.0,
                    },
                    Health { current: 1, max: 1 },
                    Position(pos),
                    Sprite {
                        image: sprites.bloon_handle(layer, false),
                        color: Color::srgba(0.50, 0.94, 1.0, 0.56),
                        custom_size: Some(Vec2::splat(cfg.size)),
                        ..default()
                    },
                    Transform::from_xyz(pos.x, pos.y, 6.6),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        GlueOverlay,
                        Sprite {
                            image: sprites.overlay_glue.clone(),
                            custom_size: Some(Vec2::splat(cfg.size)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, 0.3),
                        Visibility::Hidden,
                    ));
                });
        }
    }
}
