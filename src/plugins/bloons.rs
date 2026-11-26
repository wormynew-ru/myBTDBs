use bevy::prelude::*;

use crate::core::events::{BloonLeakedEvent, SpawnBloonEvent};
use crate::core::state::{GameState, MatchSettings};
use crate::core::types::{BloonLayer, Health, MatchEntity, Position, Side, SidesResource};
use crate::data::bloons_cfg::{bloon_cfg, wave_groups, WAVE_DURATION_SECS};
use crate::data::maps_cfg::waypoints_for_side;
use crate::utils::sample_path;

pub struct BloonsPlugin;

impl Plugin for BloonsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnQueue>()
            .add_systems(OnEnter(GameState::Playing), reset_bloon_spawner)
            .add_systems(
                Update,
                (
                    wave_timer_system,
                    process_spawn_queue,
                    handle_spawn_bloon_events,
                    move_bloons_system,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component, Debug, Clone)]
pub struct Bloon {
    pub layer: BloonLayer,
    pub camo: bool,
    pub lead: bool,
    pub path_progress: f32,
    pub freeze_timer: f32,
    pub slow_factor: f32,
    pub permafrost: bool,
    pub from_send: bool,
}

#[derive(Debug, Clone)]
pub struct QueuedSpawn {
    pub side: Side,
    pub layer: BloonLayer,
    pub camo: bool,
    pub lead: bool,
    pub delay: f32,
    pub from_send: bool,
}

#[derive(Resource, Default)]
pub struct SpawnQueue {
    pub items: Vec<QueuedSpawn>,
}

fn reset_bloon_spawner(mut queue: ResMut<SpawnQueue>) {
    queue.items.clear();
    enqueue_wave(&mut queue, 1);
}

pub fn enqueue_wave(queue: &mut SpawnQueue, wave: u32) {
    let mut cursor = 0.25;
    for group in wave_groups(wave) {
        for i in 0..group.count {
            let delay = cursor + (i as f32) * group.spacing;
            for side in [Side::Player, Side::Bot] {
                queue.items.push(QueuedSpawn {
                    side,
                    layer: group.layer,
                    camo: group.camo,
                    lead: group.lead,
                    delay,
                    from_send: false,
                });
            }
        }
        cursor += (group.count as f32) * group.spacing + 0.45;
    }
}

fn wave_timer_system(
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut sides: ResMut<SidesResource>,
    mut queue: ResMut<SpawnQueue>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;
    sides.match_time += dt;
    sides.wave_timer -= dt;
    if sides.wave_timer <= 0.0 {
        sides.wave += 1;
        sides.wave_timer += WAVE_DURATION_SECS;
        enqueue_wave(&mut queue, sides.wave);
    }
}

fn process_spawn_queue(
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut queue: ResMut<SpawnQueue>,
    mut writer: EventWriter<SpawnBloonEvent>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;
    let mut remaining = Vec::with_capacity(queue.items.len());
    for mut item in queue.items.drain(..) {
        item.delay -= dt;
        if item.delay <= 0.0 {
            writer.send(SpawnBloonEvent {
                side: item.side,
                layer: item.layer,
                camo: item.camo,
                lead: item.lead,
                path_progress: 0.0,
                from_send: item.from_send,
                permafrost: false,
            });
        } else {
            remaining.push(item);
        }
    }
    queue.items = remaining;
}

fn handle_spawn_bloon_events(
    mut commands: Commands,
    settings: Res<MatchSettings>,
    mut reader: EventReader<SpawnBloonEvent>,
) {
    for ev in reader.read() {
        let wps = waypoints_for_side(settings.map_id, ev.side);
        let (pos, _) = sample_path(&wps, ev.path_progress);
        let cfg = bloon_cfg(ev.layer);
        let tint = if ev.lead {
            Color::srgb(0.42, 0.47, 0.55)
        } else {
            cfg.color
        };

        commands
            .spawn((
                MatchEntity,
                ev.side,
                Bloon {
                    layer: ev.layer,
                    camo: ev.camo,
                    lead: ev.lead,
                    path_progress: ev.path_progress,
                    freeze_timer: 0.0,
                    slow_factor: 1.0,
                    permafrost: ev.permafrost,
                    from_send: ev.from_send,
                },
                Health {
                    current: cfg.hp,
                    max: cfg.hp,
                },
                Position(pos),
                Sprite::from_color(tint, Vec2::splat(cfg.size)),
                Transform::from_xyz(pos.x, pos.y, 6.0),
            ))
            .with_children(|parent| {
                if ev.camo {
                    parent.spawn((
                        Sprite::from_color(Color::srgb(0.10, 0.35, 0.14), Vec2::splat(10.0)),
                        Transform::from_xyz(0.0, 0.0, 0.2),
                    ));
                }
                if ev.lead {
                    parent.spawn((
                        Sprite::from_color(Color::srgb(0.82, 0.87, 0.92), Vec2::splat(8.0)),
                        Transform::from_xyz(0.0, 0.0, 0.3),
                    ));
                }
            });
    }
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
        let cfg = bloon_cfg(bloon.layer);
        let speed_mult = if bloon.freeze_timer > 0.0 {
            bloon.freeze_timer = (bloon.freeze_timer - dt).max(0.0);
            bloon.slow_factor
        } else if bloon.permafrost {
            0.72
        } else {
            1.0
        };

        bloon.path_progress += cfg.speed * speed_mult * dt;
        let wps = match side {
            Side::Player => &player_wps,
            Side::Bot => &bot_wps,
        };
        let (next_pos, finished) = sample_path(wps, bloon.path_progress);

        if finished {
            leak_writer.send(BloonLeakedEvent {
                side: *side,
                life_loss: cfg.rbe,
            });
            commands.entity(entity).despawn_recursive();
        } else {
            pos.0 = next_pos;
            tf.translation.x = next_pos.x;
            tf.translation.y = next_pos.y;
        }
    }
}
