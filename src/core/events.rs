use bevy::prelude::*;

use super::types::{BloonLayer, Side, TargetPriority, TowerType};

#[derive(Event, Debug, Clone, Copy)]
pub struct SpawnBloonEvent {
    pub side: Side,
    pub layer: BloonLayer,
    pub camo: bool,
    pub lead: bool,
    pub path_progress: f32,
    pub from_send: bool,
    pub permafrost: bool,
    pub regrab_immune_tower: Option<Entity>,
    pub regrab_cooldown: f32,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct BuildTowerEvent {
    pub side: Side,
    pub tower_type: TowerType,
    pub pos: Vec2,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct UpgradeTowerEvent {
    pub side: Side,
    pub tower_entity: Entity,
    pub branch: u8,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SellTowerEvent {
    pub side: Side,
    pub tower_entity: Entity,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct ChangePriorityEvent {
    pub tower_entity: Entity,
    pub priority: TargetPriority,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct ActivateAbilityEvent {
    pub side: Side,
    pub tower_entity: Entity,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SendBloonPackEvent {
    pub sender: Side,
    pub pack_index: usize,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct BloonLeakedEvent {
    pub side: Side,
    pub life_loss: i32,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct BloonPoppedEvent {
    pub side: Side,
    pub cash_gain: i32,
    pub pops: u32,
}
