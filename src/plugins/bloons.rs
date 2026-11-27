use std::collections::VecDeque;

use bevy::prelude::*;

use crate::core::events::{BloonLeakedEvent, SpawnBloonEvent};
use crate::core::state::{GameState, MatchSettings};
use crate::core::types::{BloonLayer, Health, MatchEntity, Position, Side, SidesResource};
use crate::data::bloons_cfg::{bloon_cfg, wave_groups, WAVE_DURATION_SECS};
use crate::data::maps_cfg::waypoints_for_side;
use crate::plugins::sprites::SpriteAssets;
use crate::utils::sample_path;

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
    pub is_pulled: bool,
    pub is_inside_chipper: bool,
    pub supa_vac_pulled: bool,
    pub regrab_immune_tower: Option<Entity>,
    pub regrab_cooldown: f32,
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

#[derive(Debug, Clone)]
pub struct QueuedSendPack {
    pub layer: BloonLayer,
    pub camo: bool,
    pub lead: bool,
    pub remaining: u32,
    pub spacing: f32,
    pub timer: f32,
}

#[derive(Resource, Default)]
pub struct SendPackQueue {
    pub player_target: VecDeque<QueuedSendPack>,
    pub bot_target: VecDeque<QueuedSendPack>,
    pub player_gap: f32,
    pub bot_gap: f32,
}

impl SendPackQueue {
    pub fn queue_for_mut(&mut self, target: Side) -> (&mut VecDeque<QueuedSendPack>, &mut f32) {
        match target {
            Side::Player => (&mut self.player_target, &mut self.player_gap),
            Side::Bot => (&mut self.bot_target, &mut self.bot_gap),
        }
    }
    pub fn count_for(&self, target: Side) -> usize {
        match target { Side::Player => self.player_target.len(), Side::Bot => self.bot_target.len() }
    }
}

fn reset_bloon_spawner(mut queue: ResMut<SpawnQueue>, mut send_q: ResMut<SendPackQueue>) {
    queue.items.clear();
    send_q.player_target.clear();
    send_q.bot_target.clear();
    send_q.player_gap = 0.0;
    send_q.bot_gap = 0.0;
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
                regrab_immune_tower: None,
                regrab_cooldown: 0.0,
            });
        } else {
            remaining.push(item);
        }
    }
    queue.items = remaining;
}

fn process_send_pack_queue(
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut send_q: ResMut<SendPackQueue>,
    mut writer: EventWriter<SpawnBloonEvent>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;

    for target in [Side::Player, Side::Bot] {
        let (queue, gap) = send_q.queue_for_mut(target);
        if *gap > 0.0 {
            *gap = (*gap - dt).max(0.0);
            continue;
        }
        let Some(front) = queue.front_mut() else {
            continue;
        };
        front.timer -= dt;
        while front.timer <= 0.0 && front.remaining > 0 {
            writer.send(SpawnBloonEvent {
                side: target,
                layer: front.layer,
                camo: front.camo,
                lead: front.lead,
                path_progress: 0.0,
                from_send: true,
                permafrost: false,
                regrab_immune_tower: None,
                regrab_cooldown: 0.0,
            });
            front.remaining -= 1;
            front.timer += front.spacing;
        }
        if front.remaining == 0 {
            queue.pop_front();
            *gap = 0.25;
        }
    }
}

fn handle_spawn_bloon_events(
    mut commands: Commands,
    settings: Res<MatchSettings>,
    sprites: Res<SpriteAssets>,
    mut reader: EventReader<SpawnBloonEvent>,
) {
    for ev in reader.read() {
        let wps = waypoints_for_side(settings.map_id, ev.side);
        let (pos, _) = sample_path(&wps, ev.path_progress);
        let cfg = bloon_cfg(ev.layer);
        let img = sprites.bloon_handle(ev.layer, ev.lead);
        let tint = if ev.lead
            || matches!(
                ev.layer,
                BloonLayer::Rainbow | BloonLayer::Moab | BloonLayer::Bfb | BloonLayer::Zomg
            ) {
            Color::WHITE
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
                    is_pulled: false,
                    is_inside_chipper: false,
                    supa_vac_pulled: false,
                    regrab_immune_tower: ev.regrab_immune_tower,
                    regrab_cooldown: ev.regrab_cooldown,
                },
                Health {
                    current: cfg.hp,
                    max: cfg.hp,
                },
                Position(pos),
                Sprite {
                    image: img,
                    color: tint,
                    custom_size: Some(Vec2::splat(cfg.size)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, if ev.layer.is_moab_class() { 6.5 } else { 6.0 }),
            ))
            .with_children(|parent| {
                if ev.camo {
                    parent.spawn((
                        Sprite {
                            image: sprites.overlay_camo.clone(),
                            custom_size: Some(Vec2::splat(cfg.size)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, 0.2),
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
        if bloon.regrab_cooldown > 0.0 {
            bloon.regrab_cooldown = (bloon.regrab_cooldown - dt).max(0.0);
            if bloon.regrab_cooldown <= 0.0 {
                bloon.regrab_immune_tower = None;
            }
        }

        if bloon.is_pulled || bloon.is_inside_chipper || bloon.supa_vac_pulled {
            continue;
        }

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
