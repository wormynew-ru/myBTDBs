use std::collections::VecDeque;

use bevy::prelude::*;

use crate::core::events::SpawnBloonEvent;
use crate::core::state::MatchSettings;
use crate::core::types::{BloonLayer, GlueStatus, Health, MatchEntity, Position, Side, SidesResource};
use crate::data::bloons_cfg::{bloon_cfg, wave_groups, wave_ramp_multipliers};
use crate::data::maps_cfg::waypoints_for_side;
use crate::plugins::sprites::SpriteAssets;
use crate::utils::sample_path;

use super::{Bloon, GlueOverlay};

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
        match target {
            Side::Player => self.player_target.len(),
            Side::Bot => self.bot_target.len(),
        }
    }
}

pub fn reset_bloon_spawner(mut queue: ResMut<SpawnQueue>, mut send_q: ResMut<SendPackQueue>) {
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

pub fn wave_timer_system(
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
        sides.wave_timer += settings.game_mode.wave_duration();
        enqueue_wave(&mut queue, sides.wave);
    }
}

pub fn process_spawn_queue(
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
    for mut item in std::mem::take(&mut queue.items) {
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
                glue: GlueStatus::default(),
                regrab_immune_tower: None,
                regrab_cooldown: 0.0,
            });
        } else {
            remaining.push(item);
        }
    }
    queue.items = remaining;
}

pub fn process_send_pack_queue(
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
                glue: GlueStatus::default(),
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

pub fn handle_spawn_bloon_events(
    mut commands: Commands,
    settings: Res<MatchSettings>,
    sides: Res<SidesResource>,
    sprites: Res<SpriteAssets>,
    mut reader: EventReader<SpawnBloonEvent>,
) {
    let (base_hp_mult, spd_mult) = wave_ramp_multipliers(sides.wave);
    for ev in reader.read() {
        let wps = waypoints_for_side(settings.map_id, ev.side);
        let (pos, _) = sample_path(&wps, ev.path_progress);
        let cfg = bloon_cfg(ev.layer);

        let mut hp_mult = base_hp_mult;
        if sides.wave >= 35 && ev.from_send && ev.layer.is_moab_class() {
            hp_mult *= 1.8;
        }
        let scaled_hp = ((cfg.hp as f32) * hp_mult).round().max(1.0) as i32;
        let img = sprites.bloon_handle(ev.layer, ev.lead);
        let tint = if ev.lead
            || matches!(
                ev.layer,
                BloonLayer::Rainbow | BloonLayer::Moab | BloonLayer::Bfb | BloonLayer::Zomg
            )
        {
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
                    speed_mult: spd_mult,
                    permafrost: ev.permafrost,
                    glue: ev.glue,
                    from_send: ev.from_send,
                    is_ghost: false,
                    ghost_life_loss: 0,
                    is_mirage: false,
                    mirage_flash_radius: 0.0,
                    mirage_flash_duration: 0.0,
                    base_speed_override: None,
                    is_pulled: false,
                    is_inside_chipper: false,
                    supa_vac_pulled: false,
                    regrab_immune_tower: ev.regrab_immune_tower,
                    regrab_cooldown: ev.regrab_cooldown,
                },
                Health {
                    current: scaled_hp,
                    max: scaled_hp,
                },
                Position(pos),
                Sprite {
                    image: img,
                    color: tint,
                    custom_size: Some(Vec2::splat(cfg.size)),
                    ..default()
                },
                Transform::from_xyz(
                    pos.x,
                    pos.y,
                    if ev.layer.is_moab_class() { 6.5 } else { 6.0 },
                ),
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
                parent.spawn((
                    GlueOverlay,
                    Sprite {
                        image: sprites.overlay_glue.clone(),
                        custom_size: Some(Vec2::splat(cfg.size)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 0.3),
                    if ev.glue.timer > 0.0 {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    },
                ));
            });
    }
}
