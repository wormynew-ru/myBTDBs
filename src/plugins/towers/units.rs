use bevy::prelude::*;

use crate::core::events::{BloonPoppedEvent, SpawnBloonEvent};
use crate::core::state::MatchSettings;
use crate::core::types::{Health, MatchEntity, Position, Side};
use crate::data::bloons_cfg::bloon_cfg;
use crate::data::maps_cfg::waypoints_for_side;
use crate::plugins::bloons::Bloon;
use crate::plugins::sprites::SpriteAssets;
use crate::utils::{circles_overlap, sample_path};

use super::units_fx::{apply_multi_layer_damage, children_layers_total, UnitExplosionFx};
use super::{Projectile, Tower};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    MilitaryVehicle(u8),
    Crook,
    TommyGoon,
    MafiaEnforcer,
}

impl UnitKind {
    pub fn is_crook(self) -> bool {
        !matches!(self, UnitKind::MilitaryVehicle(_))
    }
}

#[derive(Component, Debug, Clone)]
pub struct PathUnit {
    pub source_tower: Entity,
    pub kind: UnitKind,
    pub hp: i32,
    pub speed: f32,
    pub path_progress: f32,
    pub fire_cooldown: f32,
    pub grenade_cooldown: f32,
    pub gun_damage: i32,
    pub gun_pierce: u32,
    pub gun_cd: f32,
    pub gun_range: f32,
    pub can_hit_camo: bool,
    pub can_pop_lead: bool,
    pub ceramic_bonus: i32,
    pub moab_bonus: i32,
    pub explode_damage: i32,
    pub explode_radius: f32,
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn path_units_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    sprites: Res<SpriteAssets>,
    mut units: Query<(Entity, &Side, &mut PathUnit, &mut Position, &mut Transform), Without<Bloon>>,
    mut bloons: Query<(Entity, &Side, &Bloon, &mut Health, &Position), (Without<PathUnit>, Without<Tower>)>,
    mut towers: Query<&mut Tower, (Without<PathUnit>, Without<Bloon>)>,
    mut spawn_writer: EventWriter<SpawnBloonEvent>,
    mut pop_writer: EventWriter<BloonPoppedEvent>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;
    let player_wps = waypoints_for_side(settings.map_id, Side::Player);
    let bot_wps = waypoints_for_side(settings.map_id, Side::Bot);

    for (u_e, u_side, mut unit, mut u_pos, mut u_tf) in &mut units {
        if unit.path_progress <= 0.0 || unit.hp <= 0 {
            commands.entity(u_e).despawn_recursive();
            continue;
        }

        let mut target: Option<(Vec2, f32)> = None;
        if unit.gun_range > 0.0 {
            for (_, bs, b, hp, bp) in bloons.iter() {
                if *bs != *u_side || hp.current <= 0 || b.is_inside_chipper || (b.camo && !unit.can_hit_camo) {
                    continue;
                }
                let d = u_pos.0.distance(bp.0);
                if d <= unit.gun_range && target.map_or(true, |(_, bd)| d < bd) {
                    target = Some((bp.0, d));
                }
            }
        }

        let move_mult = if unit.kind.is_crook() {
            match target {
                Some((_, d)) if d <= unit.gun_range * 0.65 => 0.10,
                Some(_) => 0.40,
                None => 1.0,
            }
        } else {
            1.0
        };

        unit.path_progress -= unit.speed * move_mult * dt;
        if unit.path_progress <= 0.0 {
            commands.entity(u_e).despawn_recursive();
            continue;
        }

        let wps = if *u_side == Side::Player { &player_wps } else { &bot_wps };
        let (next_pos, _) = sample_path(wps, unit.path_progress);
        let move_dir = next_pos - u_pos.0;
        if let Some((tp, _)) = target.filter(|_| unit.kind.is_crook()) {
            let aim = (tp - next_pos).normalize_or(Vec2::X);
            u_tf.rotation = Quat::from_rotation_z(aim.y.atan2(aim.x));
        } else if move_dir.length_squared() > 0.01 {
            u_tf.rotation = Quat::from_rotation_z(move_dir.y.atan2(move_dir.x));
        }
        u_pos.0 = next_pos;
        u_tf.translation.x = next_pos.x;
        u_tf.translation.y = next_pos.y;

        if unit.gun_cd > 0.0 {
            unit.fire_cooldown -= dt;
            if unit.fire_cooldown <= 0.0 {
                if let Some((tp, _)) = target {
                    let (splash, spd, img, tint) = match unit.kind {
                        UnitKind::MilitaryVehicle(2) => (0.0, 520.0, sprites.proj_dart.clone(), Color::srgb(0.95, 0.85, 0.3)),
                        UnitKind::MilitaryVehicle(3) => (56.0, 410.0, sprites.proj_bomb.clone(), Color::WHITE),
                        UnitKind::MilitaryVehicle(4) => (0.0, 780.0, sprites.proj_dart.clone(), Color::srgb(0.22, 0.88, 0.98)),
                        UnitKind::MafiaEnforcer => (0.0, 580.0, sprites.proj_dart.clone(), Color::srgb(0.98, 0.84, 0.22)),
                        _ => (0.0, 510.0, sprites.proj_dart.clone(), Color::WHITE),
                    };
                    unit.fire_cooldown = unit.gun_cd;
                    spawn_unit_proj(
                        &mut commands, *u_side, unit.source_tower, u_pos.0, tp, spd,
                        unit.gun_damage, unit.gun_pierce, splash, unit.can_pop_lead,
                        unit.can_hit_camo, unit.ceramic_bonus, unit.moab_bonus, 0.0, img, tint,
                    );
                }
            }
        }

        if unit.kind == UnitKind::MafiaEnforcer {
            unit.grenade_cooldown -= dt;
            if unit.grenade_cooldown <= 0.0 {
                if let Some((tp, _)) = target {
                    unit.grenade_cooldown = 2.2;
                    spawn_unit_proj(
                        &mut commands, *u_side, unit.source_tower, u_pos.0, tp, 380.0,
                        6, 1, 64.0, true, unit.can_hit_camo, unit.ceramic_bonus, unit.moab_bonus,
                        0.35, sprites.proj_bomb.clone(), Color::srgb(0.98, 0.72, 0.18),
                    );
                }
            }
        }

        let mut exploded = false;
        let mut src_tower = towers.get_mut(unit.source_tower).ok();
        for (b_e, bs, b, mut hp, bp) in &mut bloons {
            if *bs != *u_side || hp.current <= 0 || b.is_inside_chipper {
                continue;
            }
            if unit.kind.is_crook() && ((b.camo && !unit.can_hit_camo) || (b.lead && !unit.can_pop_lead)) {
                continue;
            }
            if !circles_overlap(u_pos.0, 15.0, bp.0, bloon_cfg(b.layer).size * 0.5) {
                continue;
            }
            let bloon_total = hp.current + children_layers_total(b.layer);
            let deal = unit.hp.max(1);
            unit.hp -= bloon_total;
            apply_multi_layer_damage(
                &mut commands, b_e, *u_side, b, &mut hp, deal,
                src_tower.as_deref_mut(), &mut spawn_writer, &mut pop_writer,
            );
            if unit.hp <= 0 {
                exploded = true;
                break;
            }
        }

        if exploded {
            if unit.explode_radius > 0.0 && unit.explode_damage > 0 {
                commands.spawn((
                    MatchEntity,
                    UnitExplosionFx {
                        life: 0.25, max_life: 0.25,
                        target_size: unit.explode_radius * 2.0,
                        color: Color::srgba(0.98, 0.55, 0.12, 0.85),
                    },
                    Sprite::from_color(Color::srgba(0.98, 0.55, 0.12, 0.85), Vec2::splat(16.0)),
                    Transform::from_xyz(u_pos.0.x, u_pos.0.y, 8.8),
                ));
                for (b_e, bs, b, mut hp, bp) in &mut bloons {
                    if *bs != *u_side || hp.current <= 0 || b.is_inside_chipper {
                        continue;
                    }
                    if unit.kind.is_crook() && b.camo && !unit.can_hit_camo {
                        continue;
                    }
                    if circles_overlap(u_pos.0, unit.explode_radius, bp.0, bloon_cfg(b.layer).size * 0.5) {
                        apply_multi_layer_damage(
                            &mut commands, b_e, *u_side, b, &mut hp, unit.explode_damage,
                            src_tower.as_deref_mut(), &mut spawn_writer, &mut pop_writer,
                        );
                    }
                }
            }
            commands.entity(u_e).despawn_recursive();
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_unit_proj(
    commands: &mut Commands, side: Side, source_tower: Entity, origin: Vec2, target: Vec2,
    spd: f32, dmg: i32, pierce: u32, splash: f32, lead: bool, camo: bool,
    ceramic_bonus: i32, moab_bonus: i32, stun_duration: f32, image: Handle<Image>, color: Color,
) {
    let dir = (target - origin).normalize_or(Vec2::X);
    let ang = dir.y.atan2(dir.x);
    commands.spawn((
        MatchEntity,
        side,
        Position(origin + dir * 12.0),
        Projectile {
            source_tower,
            velocity: dir * spd,
            damage: dmg,
            pierce_left: pierce,
            splash_radius: splash,
            can_pop_lead: lead,
            can_hit_camo: camo,
            knockback_chance: 0.0,
            stun_duration,
            ceramic_bonus,
            moab_bonus,
            life: 0.85,
            hit_ids: Vec::new(),
        },
        Sprite { image, color, custom_size: Some(Vec2::splat(14.0)), ..default() },
        Transform::from_xyz(origin.x, origin.y, 9.0).with_rotation(Quat::from_rotation_z(ang)),
    ));
}
