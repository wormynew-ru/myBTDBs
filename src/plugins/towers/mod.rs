mod chipper;
mod chipper_fx;
mod combat;
mod combat_proj;
mod commander;
mod crook_boss;
mod military_base;
mod spikes;
mod spikes_fx;
mod units;
mod units_fx;

use bevy::prelude::*;

use crate::core::events::{
    ActivateAbilityEvent, BuildTowerEvent, ChangePriorityEvent, SellTowerEvent, UpgradeTowerEvent,
};
use crate::core::state::{GameState, MatchSettings, SelectedTower};
use crate::core::types::{MatchEntity, Position, Side, SidesResource, TargetPriority, TowerType};
use crate::data::maps_cfg::waypoints_for_side;
use crate::data::tower_stats::VillageBuff;
use crate::data::towers_cfg::{tower_cfg, SELL_REFUND_RATIO, TOWER_RADIUS};
use crate::plugins::sprites::SpriteAssets;
use crate::utils::is_valid_tower_spot;

pub use chipper::{ChipperState, ShredParticle};
use chipper::chipper_fsm_system;
use chipper_fx::{shred_particle_system, supa_vac_system};
use combat::tower_attack_system;
use combat_proj::projectile_update_system;
use commander::{
    bodyguard_system, call_to_arms_fx_system, commander_ability_and_guards_system,
    update_tower_buffs_system, CommanderState,
};
use crook_boss::{crook_boss_spawn_system, CrookBossState};
use military_base::{military_base_spawn_system, MilitaryBaseState};
use spikes::{spike_factory_produce_system, spike_piles_system, SpikeFactoryState};
use spikes_fx::spike_explosion_fx_system;
use units::path_units_system;
use units_fx::unit_explosion_fx_system;

pub struct TowersPlugin;

impl Plugin for TowersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (
                    handle_build_tower,
                    handle_upgrade_tower,
                    handle_sell_tower,
                    handle_priority_change,
                    handle_activate_ability,
                    update_tower_buffs_system,
                    commander_ability_and_guards_system,
                    bodyguard_system,
                    call_to_arms_fx_system,
                    tower_attack_system,
                )
                    .chain(),
                (
                    chipper_fsm_system,
                    supa_vac_system,
                    shred_particle_system,
                    military_base_spawn_system,
                    crook_boss_spawn_system,
                    path_units_system,
                    unit_explosion_fx_system,
                    projectile_update_system,
                    spike_factory_produce_system,
                    spike_piles_system,
                    spike_explosion_fx_system,
                )
                    .chain(),
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

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct TowerBuff(pub VillageBuff);

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
    pub ceramic_bonus: i32,
    pub moab_bonus: i32,
    pub life: f32,
    pub hit_ids: Vec<Entity>,
}

fn handle_build_tower(
    mut commands: Commands,
    settings: Res<MatchSettings>,
    sprites: Res<SpriteAssets>,
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
        let tex = sprites.tower_handle(ev.tower_type, 0, 0);
        let mut entity_cmds = commands.spawn((
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
            TowerBuff::default(),
            Position(ev.pos),
            Sprite {
                image: tex,
                custom_size: Some(Vec2::splat(TOWER_RADIUS * 2.1)),
                ..default()
            },
            Transform::from_xyz(ev.pos.x, ev.pos.y, 7.0),
        ));

        if ev.tower_type == TowerType::Bloonchipper {
            entity_cmds.insert(ChipperState::default());
        } else if ev.tower_type == TowerType::SpikeFactory {
            let seed = (ev.pos.x.to_bits() ^ ev.pos.y.to_bits().rotate_left(16)) | 1;
            entity_cmds.insert(SpikeFactoryState {
                storm_timer: 0.5,
                rng_state: seed,
            });
        } else if ev.tower_type == TowerType::Commander {
            entity_cmds.insert(CommanderState::default());
        } else if ev.tower_type == TowerType::MilitaryBase {
            entity_cmds.insert(MilitaryBaseState::default());
        } else if ev.tower_type == TowerType::CrookBoss {
            entity_cmds.insert(CrookBossState::default());
        }

        let entity = entity_cmds.id();
        if ev.side == Side::Player {
            selected.0 = Some(entity);
        }
    }
}

fn handle_upgrade_tower(
    mut sides: ResMut<SidesResource>,
    sprites: Res<SpriteAssets>,
    mut towers: Query<(&Side, &mut Tower, &mut Sprite)>,
    mut reader: EventReader<UpgradeTowerEvent>,
) {
    for ev in reader.read() {
        let Ok((side, mut tower, mut sprite)) = towers.get_mut(ev.tower_entity) else {
            continue;
        };
        if *side != ev.side {
            continue;
        }
        let cfg = tower_cfg(tower.tower_type);
        let tier = if ev.branch == 1 { tower.path1 } else { tower.path2 };
        let branch_cfgs = if ev.branch == 1 { cfg.path1 } else { cfg.path2 };
        if (tier as usize) >= branch_cfgs.len() {
            continue;
        }
        let upg = branch_cfgs[tier as usize];
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
        sprite.image = sprites.tower_handle(tower.tower_type, tower.path1, tower.path2);
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

fn handle_activate_ability(
    mut query: Query<(&Side, &Tower, &mut ChipperState)>,
    mut reader: EventReader<ActivateAbilityEvent>,
) {
    for ev in reader.read() {
        if let Ok((side, tower, mut chipper)) = query.get_mut(ev.tower_entity) {
            if *side == ev.side
                && tower.tower_type == TowerType::Bloonchipper
                && tower.path2 >= 4
                && chipper.supa_vac_cooldown <= 0.0
            {
                chipper.supa_vac_active = 7.0;
                chipper.supa_vac_cooldown = 28.0;
            }
        }
    }
}
