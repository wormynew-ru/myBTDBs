use bevy::prelude::*;

use crate::core::events::BuildTowerEvent;
use crate::core::input::PointerInput;
use crate::core::state::{BuildSelection, GameState, MatchSettings, SelectedTower};
use crate::core::types::{MatchEntity, Position, Side};
use crate::data::maps_cfg::{
    map_cfg, waypoints_for_side, ARENA_H, ARENA_W, DIVIDER_HALF, ROAD_WIDTH,
};
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats};
use crate::data::towers_cfg::{tower_cfg, TOWER_RADIUS};
use crate::plugins::sprites::SpriteAssets;
use crate::plugins::towers::{Tower, TowerBuff};
use crate::utils::is_valid_tower_spot;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_map_visuals)
            .add_systems(OnEnter(GameState::MainMenu), cleanup_match_entities)
            .add_systems(
                Update,
                (
                    handle_map_pointer,
                    update_placement_preview,
                    update_range_indicator,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
struct PlacementPreview;

#[derive(Component)]
struct SelectedRangeRing;

fn cleanup_match_entities(
    mut commands: Commands,
    query: Query<Entity, With<MatchEntity>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_map_visuals(
    mut commands: Commands,
    settings: Res<MatchSettings>,
    sprites: Res<SpriteAssets>,
    existing: Query<Entity, With<MatchEntity>>,
) {
    for entity in &existing {
        commands.entity(entity).despawn_recursive();
    }

    let cfg = map_cfg(settings.map_id);

    // Arena background
    commands.spawn((
        MatchEntity,
        Sprite::from_color(cfg.grass_color, Vec2::new(ARENA_W, ARENA_H)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Symmetric roads for both Player and Bot
    for side in [Side::Player, Side::Bot] {
        let wps = waypoints_for_side(settings.map_id, side);
        for pair in wps.windows(2) {
            let a = pair[0];
            let b = pair[1];
            let delta = b - a;
            let len = delta.length();
            let mid = (a + b) * 0.5;
            let angle = delta.y.atan2(delta.x);

            commands.spawn((
                MatchEntity,
                Sprite::from_color(
                    cfg.road_border_color,
                    Vec2::new(len + ROAD_WIDTH + 6.0, ROAD_WIDTH + 6.0),
                ),
                Transform::from_xyz(mid.x, mid.y, 1.0)
                    .with_rotation(Quat::from_rotation_z(angle)),
            ));
            commands.spawn((
                MatchEntity,
                Sprite::from_color(
                    cfg.road_color,
                    Vec2::new(len + ROAD_WIDTH, ROAD_WIDTH),
                ),
                Transform::from_xyz(mid.x, mid.y, 1.5)
                    .with_rotation(Quat::from_rotation_z(angle)),
            ));
        }

        if let (Some(&entry), Some(&exit)) = (wps.first(), wps.last()) {
            commands.spawn((
                MatchEntity,
                Sprite {
                    image: sprites.portal_entry.clone(),
                    custom_size: Some(Vec2::splat(26.0)),
                    ..default()
                },
                Transform::from_xyz(entry.x.clamp(-624.0, 624.0), entry.y.clamp(-255.0, 280.0), 2.0),
            ));
            commands.spawn((
                MatchEntity,
                Sprite {
                    image: sprites.portal_exit.clone(),
                    custom_size: Some(Vec2::splat(26.0)),
                    ..default()
                },
                Transform::from_xyz(exit.x.clamp(-624.0, 624.0), exit.y.clamp(-255.0, 280.0), 2.0),
            ));
        }
    }

    // Center divider
    commands.spawn((
        MatchEntity,
        Sprite::from_color(
            Color::srgb(0.06, 0.09, 0.16),
            Vec2::new(DIVIDER_HALF * 2.0, ARENA_H),
        ),
        Transform::from_xyz(0.0, 0.0, 3.0),
    ));

    // Hover placement preview
    commands.spawn((
        MatchEntity,
        PlacementPreview,
        Sprite::from_color(Color::srgba(0.2, 0.9, 0.3, 0.35), Vec2::splat(TOWER_RADIUS * 2.0)),
        Transform::from_xyz(0.0, 0.0, 8.0),
        Visibility::Hidden,
    ));

    // Selected tower range box
    commands.spawn((
        MatchEntity,
        SelectedRangeRing,
        Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.12), Vec2::splat(100.0)),
        Transform::from_xyz(0.0, 0.0, 4.0),
        Visibility::Hidden,
    ));
}

fn handle_map_pointer(
    pointer: Res<PointerInput>,
    mut build_sel: ResMut<BuildSelection>,
    mut selected_tower: ResMut<SelectedTower>,
    towers: Query<(Entity, &Side, &Position), With<Tower>>,
    mut build_writer: EventWriter<BuildTowerEvent>,
) {
    if !pointer.just_pressed {
        return;
    }
    let Some(world_pos) = pointer.world_pos else {
        return;
    };

    if let Some(tower_type) = build_sel.0 {
        if world_pos.x < -DIVIDER_HALF {
            build_writer.send(BuildTowerEvent {
                side: Side::Player,
                tower_type,
                pos: world_pos,
            });
            build_sel.0 = None;
        }
        return;
    }

    let mut found: Option<Entity> = None;
    let mut best_dist = TOWER_RADIUS + 12.0;
    for (entity, side, pos) in &towers {
        if *side != Side::Player {
            continue;
        }
        let d = pos.0.distance(world_pos);
        if d <= best_dist {
            best_dist = d;
            found = Some(entity);
        }
    }
    selected_tower.0 = found;
}

fn update_placement_preview(
    pointer: Res<PointerInput>,
    build_sel: Res<BuildSelection>,
    settings: Res<MatchSettings>,
    towers: Query<(&Side, &Position), With<Tower>>,
    mut preview_q: Query<(&mut Sprite, &mut Transform, &mut Visibility), With<PlacementPreview>>,
) {
    let Ok((mut sprite, mut tf, mut vis)) = preview_q.get_single_mut() else {
        return;
    };

    let (Some(tower_type), Some(world_pos)) = (build_sel.0, pointer.world_pos) else {
        *vis = Visibility::Hidden;
        return;
    };

    let wps = waypoints_for_side(settings.map_id, Side::Player);
    let existing: Vec<Vec2> = towers
        .iter()
        .filter(|(s, _)| **s == Side::Player)
        .map(|(_, p)| p.0)
        .collect();

    let valid = is_valid_tower_spot(world_pos, Side::Player, &wps, &existing);
    let cfg = tower_cfg(tower_type);
    sprite.color = if valid {
        Color::srgba(0.20, 0.85, 0.35, 0.45)
    } else {
        Color::srgba(0.92, 0.20, 0.20, 0.45)
    };
    sprite.custom_size = Some(Vec2::splat(cfg.range * 2.0));
    tf.translation = Vec3::new(world_pos.x, world_pos.y, 4.2);
    *vis = Visibility::Visible;
}

fn update_range_indicator(
    selected: Res<SelectedTower>,
    towers: Query<(&Tower, &TowerBuff, &Position)>,
    mut ring_q: Query<(&mut Sprite, &mut Transform, &mut Visibility), With<SelectedRangeRing>>,
) {
    let Ok((mut sprite, mut tf, mut vis)) = ring_q.get_single_mut() else {
        return;
    };

    if let Some(entity) = selected.0 {
        if let Ok((tower, buff, pos)) = towers.get(entity) {
            let stats = apply_village_buff(
                compute_effective_stats(tower.tower_type, tower.path1, tower.path2),
                buff.0,
            );
            sprite.custom_size = Some(Vec2::splat(stats.range * 2.0));
            tf.translation = Vec3::new(pos.0.x, pos.0.y, 4.0);
            *vis = Visibility::Visible;
            return;
        }
    }
    *vis = Visibility::Hidden;
}
