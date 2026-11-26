mod combat;

use bevy::prelude::*;

use crate::core::events::{
    BuildTowerEvent, ChangePriorityEvent, SellTowerEvent, UpgradeTowerEvent,
};
use crate::core::state::{GameState, MatchSettings, SelectedTower};
use crate::core::types::{MatchEntity, Position, Side, SidesResource, TargetPriority, TowerType};
use crate::data::maps_cfg::waypoints_for_side;
use crate::data::towers_cfg::{tower_cfg, SELL_REFUND_RATIO, TOWER_RADIUS};
use crate::utils::is_valid_tower_spot;

use combat::{projectile_update_system, tower_attack_system};

pub struct TowersPlugin;

impl Plugin for TowersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_build_tower,
                handle_upgrade_tower,
                handle_sell_tower,
                handle_priority_change,
                tower_attack_system,
                projectile_update_system,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component, Debug, Clone)]
pub struct Tower {
    pub tower_type: TowerType,
    pub path1: u8,
    pub path2: u8,
    pub priority: TargetPriority,
    pub cooldown: f32,
    pub total_invested: i32,
    pub pops: u32,
    pub shot_count: u32,
}

#[derive(Component, Debug, Clone)]
pub struct Projectile {
    pub source_tower: Entity,
    pub velocity: Vec2,
    pub damage: i32,
    pub pierce_left: u32,
    pub splash_radius: f32,
    pub can_pop_lead: bool,
    pub can_hit_camo: bool,
    pub knockback_chance: f32,
    pub stun_duration: f32,
    pub life: f32,
    pub hit_ids: Vec<Entity>,
}

fn handle_build_tower(
    mut commands: Commands,
    settings: Res<MatchSettings>,
    mut sides: ResMut<SidesResource>,
    mut selected: ResMut<SelectedTower>,
    existing_towers: Query<(&Side, &Position), With<Tower>>,
    mut reader: EventReader<BuildTowerEvent>,
) {
    for ev in reader.read() {
        let cfg = tower_cfg(ev.tower_type);
        if sides.get(ev.side).cash < cfg.cost {
            continue;
        }
        let wps = waypoints_for_side(settings.map_id, ev.side);
        let side_positions: Vec<Vec2> = existing_towers
            .iter()
            .filter(|(s, _)| **s == ev.side)
            .map(|(_, p)| p.0)
            .collect();

        if !is_valid_tower_spot(ev.pos, ev.side, &wps, &side_positions) {
            continue;
        }

        sides.get_mut(ev.side).cash -= cfg.cost;
        let entity = commands
            .spawn((
                MatchEntity,
                ev.side,
                Tower {
                    tower_type: ev.tower_type,
                    path1: 0,
                    path2: 0,
                    priority: TargetPriority::First,
                    cooldown: 0.1,
                    total_invested: cfg.cost,
                    pops: 0,
                    shot_count: 0,
                },
                Position(ev.pos),
                Sprite::from_color(cfg.color, Vec2::splat(TOWER_RADIUS * 2.0)),
                Transform::from_xyz(ev.pos.x, ev.pos.y, 7.0),
            ))
            .id();

        if ev.side == Side::Player {
            selected.0 = Some(entity);
        }
    }
}

fn handle_upgrade_tower(
    mut sides: ResMut<SidesResource>,
    mut towers: Query<(&Side, &mut Tower)>,
    mut reader: EventReader<UpgradeTowerEvent>,
) {
    for ev in reader.read() {
        let Ok((side, mut tower)) = towers.get_mut(ev.tower_entity) else {
            continue;
        };
        if *side != ev.side {
            continue;
        }
        let cfg = tower_cfg(tower.tower_type);
        let tier = if ev.branch == 1 { tower.path1 } else { tower.path2 };
        if tier >= 3 {
            continue;
        }
        let upg = if ev.branch == 1 {
            cfg.path1[tier as usize]
        } else {
            cfg.path2[tier as usize]
        };
        if sides.get(*side).cash < upg.cost {
            continue;
        }
        sides.get_mut(*side).cash -= upg.cost;
        tower.total_invested += upg.cost;
        if ev.branch == 1 {
            tower.path1 += 1;
        } else {
            tower.path2 += 1;
        }
    }
}

fn handle_sell_tower(
    mut commands: Commands,
    mut sides: ResMut<SidesResource>,
    mut selected: ResMut<SelectedTower>,
    towers: Query<(&Side, &Tower)>,
    mut reader: EventReader<SellTowerEvent>,
) {
    for ev in reader.read() {
        let Ok((side, tower)) = towers.get(ev.tower_entity) else {
            continue;
        };
        if *side != ev.side {
            continue;
        }
        let refund = ((tower.total_invested as f32) * SELL_REFUND_RATIO).round() as i32;
        sides.get_mut(*side).cash += refund;
        if selected.0 == Some(ev.tower_entity) {
            selected.0 = None;
        }
        commands.entity(ev.tower_entity).despawn_recursive();
    }
}

fn handle_priority_change(
    mut towers: Query<&mut Tower>,
    mut reader: EventReader<ChangePriorityEvent>,
) {
    for ev in reader.read() {
        if let Ok(mut tower) = towers.get_mut(ev.tower_entity) {
            tower.priority = ev.priority;
        }
    }
}
