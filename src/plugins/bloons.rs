#[path = "bloons_spawn.rs"]
mod bloons_spawn;

use bevy::prelude::*;

use crate::core::events::BloonLeakedEvent;
use crate::core::state::{GameState, MatchSettings};
use crate::core::types::{BloonLayer, GlueStatus, Position, Side};
use crate::data::bloons_cfg::bloon_cfg;
use crate::data::maps_cfg::waypoints_for_side;
use crate::utils::sample_path;

pub use bloons_spawn::{QueuedSendPack, SendPackQueue, SpawnQueue};
use bloons_spawn::{
    handle_spawn_bloon_events, process_send_pack_queue, process_spawn_queue, reset_bloon_spawner,
    wave_timer_system,
};

pub struct BloonsPlugin;

impl Plugin for BloonsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnQueue>()
            .init_resource::<SendPackQueue>()
            .add_systems(OnEnter(GameState::Playing), reset_bloon_spawner)
            .add_systems(
                Update,
                (
                    wave_timer_system,
                    process_spawn_queue,
                    process_send_pack_queue,
                    handle_spawn_bloon_events,
                    move_bloons_system,
                    sync_glue_overlay_system,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct GlueOverlay;

#[derive(Component, Debug, Clone)]
pub struct Bloon {
    pub layer: BloonLayer,
    pub camo: bool,
    pub lead: bool,
    pub path_progress: f32,
    pub freeze_timer: f32,
    pub slow_factor: f32,
    pub speed_mult: f32,
    pub permafrost: bool,
    pub glue: GlueStatus,
    pub from_send: bool,
    pub is_ghost: bool,
    pub ghost_life_loss: i32,
    pub is_mirage: bool,
    pub mirage_flash_radius: f32,
    pub mirage_flash_duration: f32,
    pub base_speed_override: Option<f32>,
    pub is_pulled: bool,
    pub is_inside_chipper: bool,
    pub supa_vac_pulled: bool,
    pub regrab_immune_tower: Option<Entity>,
    pub regrab_cooldown: f32,
}

fn move_bloons_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut bloons: Query<(Entity, &Side, &mut Bloon, &mut Position, &mut Transform)>,
    mut leak_writer: EventWriter<BloonLeakedEvent>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;
    let player_wps = waypoints_for_side(settings.map_id, Side::Player);
    let bot_wps = waypoints_for_side(settings.map_id, Side::Bot);

    for (entity, side, mut bloon, mut pos, mut tf) in &mut bloons {
        if bloon.regrab_cooldown > 0.0 {
            bloon.regrab_cooldown = (bloon.regrab_cooldown - dt).max(0.0);
            if bloon.regrab_cooldown <= 0.0 {
                bloon.regrab_immune_tower = None;
            }
        }

        if bloon.is_pulled || bloon.is_inside_chipper || bloon.supa_vac_pulled {
            continue;
        }

        if bloon.glue.timer > 0.0 {
            bloon.glue.timer = (bloon.glue.timer - dt).max(0.0);
        }

        let cfg = bloon_cfg(bloon.layer);
        let freeze_mult = if bloon.freeze_timer > 0.0 {
            bloon.freeze_timer = (bloon.freeze_timer - dt).max(0.0);
            bloon.slow_factor
        } else if bloon.permafrost {
            0.72
        } else {
            1.0
        };
        let glue_mult = if bloon.glue.timer > 0.0 { 0.5 } else { 1.0 };
        let speed_mult = freeze_mult.min(glue_mult);
        let base_speed = bloon.base_speed_override.unwrap_or(cfg.speed);

        bloon.path_progress += base_speed * bloon.speed_mult * speed_mult * dt;
        let wps = match side {
            Side::Player => &player_wps,
            Side::Bot => &bot_wps,
        };
        let (next_pos, finished) = sample_path(wps, bloon.path_progress);

        if finished {
            let life_loss = if bloon.is_mirage {
                0
            } else if bloon.is_ghost {
                bloon.ghost_life_loss
            } else {
                cfg.rbe
            };
            if life_loss > 0 {
                leak_writer.send(BloonLeakedEvent {
                    side: *side,
                    life_loss,
                });
            }
            commands.entity(entity).despawn_recursive();
        } else {
            if bloon.layer.is_moab_class() {
                let dir = next_pos - pos.0;
                if dir.length_squared() > 0.01 {
                    tf.rotation = Quat::from_rotation_z(dir.y.atan2(dir.x));
                }
            }
            pos.0 = next_pos;
            tf.translation.x = next_pos.x;
            tf.translation.y = next_pos.y;
        }
    }
}

fn sync_glue_overlay_system(
    bloons: Query<(&Bloon, &Children)>,
    mut overlays: Query<&mut Visibility, With<GlueOverlay>>,
) {
    for (bloon, children) in &bloons {
        let vis = if bloon.glue.timer > 0.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        for &child in children.iter() {
            if let Ok(mut v) = overlays.get_mut(child) {
                *v = vis;
            }
        }
    }
}
