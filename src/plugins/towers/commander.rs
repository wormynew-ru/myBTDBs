use bevy::prelude::*;

use crate::core::state::MatchSettings;
use crate::core::types::{Health, MatchEntity, Position, Side, TargetPriority, TowerType};
use crate::data::bloons_cfg::bloon_cfg;
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats, VillageBuff};
use crate::plugins::bloons::Bloon;
use crate::plugins::sprites::SpriteAssets;

use super::{Projectile, Tower, TowerBuff};

#[derive(Component, Debug, Clone)]
pub struct CommanderState {
    pub cta_cooldown: f32,
    pub cta_active: f32,
    pub guards_spawned: u8,
}

impl Default for CommanderState {
    fn default() -> Self {
        Self {
            cta_cooldown: 4.0,
            cta_active: 0.0,
            guards_spawned: 0,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct CommanderGuard {
    pub parent_tower: Entity,
    pub offset: Vec2,
    pub cooldown: f32,
    pub burst_left: u8,
    pub burst_timer: f32,
}

#[derive(Component, Debug, Clone)]
pub struct CallToArmsFx {
    pub life: f32,
    pub max_life: f32,
    pub target_size: f32,
}

#[allow(clippy::type_complexity)]
pub fn update_tower_buffs_system(
    mut towers: Query<(
        Entity,
        &Side,
        &Tower,
        &Position,
        &mut TowerBuff,
        Option<&CommanderState>,
    )>,
) {
    let mut villages: Vec<(Entity, Side, Vec2, f32, VillageBuff)> = Vec::new();
    let mut commanders: Vec<(Entity, Side, Vec2, f32, VillageBuff)> = Vec::new();

    for (e, side, t, pos, _, cmd) in towers.iter() {
        let s = compute_effective_stats(t.tower_type, t.path1, t.path2);
        if t.tower_type == TowerType::Village {
            villages.push((
                e,
                *side,
                pos.0,
                s.range,
                VillageBuff {
                    camo: s.aura_camo,
                    lead: s.aura_lead,
                    speed_mult: s.aura_speed_mult,
                    range_mult: s.aura_range_mult,
                    pierce_bonus: s.aura_pierce_bonus,
                    damage_bonus: 0,
                },
            ));
        } else if t.tower_type == TowerType::Commander {
            let cta_mult = if cmd.is_some_and(|c| c.cta_active > 0.0) {
                1.45
            } else {
                1.0
            };
            commanders.push((
                e,
                *side,
                pos.0,
                s.range,
                VillageBuff {
                    camo: false,
                    lead: s.aura_lead,
                    speed_mult: s.aura_speed_mult * cta_mult,
                    range_mult: 1.0,
                    pierce_bonus: 0,
                    damage_bonus: s.aura_damage_bonus,
                },
            ));
        }
    }

    for (entity, side, tower, pos, mut buff, _) in &mut towers {
        let mut v_comb = VillageBuff::default();
        if tower.tower_type != TowerType::Village {
            for (ve, vs, vp, vr, vb) in &villages {
                if *ve == entity || *vs != *side || pos.0.distance(*vp) > *vr {
                    continue;
                }
                v_comb.camo |= vb.camo;
                v_comb.lead |= vb.lead;
                v_comb.speed_mult = v_comb.speed_mult.max(vb.speed_mult);
                v_comb.range_mult = v_comb.range_mult.max(vb.range_mult);
                v_comb.pierce_bonus = v_comb.pierce_bonus.max(vb.pierce_bonus);
            }
        }

        let mut c_speed = 1.0_f32;
        let mut c_lead = false;
        let mut c_dmg = 0_i32;
        for (ce, cs, cp, cr, cb) in &commanders {
            if *ce == entity || *cs != *side || pos.0.distance(*cp) > *cr {
                continue;
            }
            c_speed = c_speed.max(cb.speed_mult);
            c_lead |= cb.lead;
            c_dmg = c_dmg.max(cb.damage_bonus);
        }

        buff.0 = VillageBuff {
            camo: v_comb.camo,
            lead: v_comb.lead || c_lead,
            speed_mult: v_comb.speed_mult * c_speed,
            range_mult: v_comb.range_mult,
            pierce_bonus: v_comb.pierce_bonus,
            damage_bonus: c_dmg,
        };
    }
}

pub fn commander_ability_and_guards_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    sprites: Res<SpriteAssets>,
    mut commanders: Query<(Entity, &Side, &Tower, &Position, &mut CommanderState)>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;

    for (entity, side, tower, pos, mut state) in &mut commanders {
        if tower.tower_type != TowerType::Commander {
            continue;
        }
        let stats = compute_effective_stats(tower.tower_type, tower.path1, tower.path2);

        if stats.has_call_to_arms {
            if state.cta_active > 0.0 {
                state.cta_active = (state.cta_active - dt).max(0.0);
            } else {
                state.cta_cooldown -= dt;
                if state.cta_cooldown <= 0.0 {
                    state.cta_active = 6.0;
                    state.cta_cooldown = 16.0;
                    commands.spawn((
                        MatchEntity,
                        CallToArmsFx {
                            life: 0.55,
                            max_life: 0.55,
                            target_size: stats.range * 2.0,
                        },
                        Sprite::from_color(
                            Color::srgba(0.98, 0.80, 0.24, 0.45),
                            Vec2::splat(24.0),
                        ),
                        Transform::from_xyz(pos.0.x, pos.0.y, 4.3),
                    ));
                }
            }
        }

        if stats.guard_count > state.guards_spawned {
            let offsets = [Vec2::new(-18.0, 15.0), Vec2::new(-18.0, -15.0)];
            for idx in state.guards_spawned..stats.guard_count {
                let off = offsets[(idx as usize) % offsets.len()];
                let gpos = pos.0 + off;
                commands.spawn((
                    MatchEntity,
                    *side,
                    Position(gpos),
                    CommanderGuard {
                        parent_tower: entity,
                        offset: off,
                        cooldown: 0.2,
                        burst_left: 0,
                        burst_timer: 0.0,
                    },
                    Sprite {
                        image: sprites.tower_dart.clone(),
                        color: Color::srgb(0.35, 0.65, 1.0),
                        custom_size: Some(Vec2::splat(22.0)),
                        ..default()
                    },
                    Transform::from_xyz(gpos.x, gpos.y, 7.2),
                ));
            }
            state.guards_spawned = stats.guard_count;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn bodyguard_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    sprites: Res<SpriteAssets>,
    towers: Query<(&Tower, &TowerBuff, &Position), Without<CommanderGuard>>,
    bloons: Query<(&Side, &Bloon, &Health, &Position), (Without<Tower>, Without<CommanderGuard>)>,
    mut guards: Query<(Entity, &Side, &mut CommanderGuard, &mut Position, &mut Transform)>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;

    for (g_e, g_side, mut guard, mut g_pos, mut g_tf) in &mut guards {
        let Ok((parent, buff, p_pos)) = towers.get(guard.parent_tower) else {
            commands.entity(g_e).despawn_recursive();
            continue;
        };

        let anchor = p_pos.0 + guard.offset;
        g_pos.0 = anchor;
        g_tf.translation.x = anchor.x;
        g_tf.translation.y = anchor.y;

        let stats = apply_village_buff(
            compute_effective_stats(parent.tower_type, parent.path1, parent.path2),
            buff.0,
        );
        let elite = stats.guard_elite;
        let guard_range = stats.range * 0.95;

        let mut best: Option<(Vec2, f32)> = None;
        for (b_side, b, hp, bp) in &bloons {
            if *b_side != *g_side || hp.current <= 0 || b.is_inside_chipper {
                continue;
            }
            let d = g_pos.0.distance(bp.0);
            if d > guard_range {
                continue;
            }
            let score = match parent.priority {
                TargetPriority::First => b.path_progress,
                TargetPriority::Last => -b.path_progress,
                TargetPriority::Close => -d,
                TargetPriority::Strong => (bloon_cfg(b.layer).rbe as f32) * 10000.0 + b.path_progress,
            };
            if best.map_or(true, |(_, bs)| score > bs) {
                best = Some((bp.0, score));
            }
        }

        if guard.burst_left == 0 {
            guard.cooldown = (guard.cooldown - dt).max(0.0);
            if guard.cooldown <= 0.0 && best.is_some() {
                let base_cd = if elite { 0.85 } else { 1.25 };
                guard.cooldown = base_cd / buff.0.speed_mult.max(1.0);
                guard.burst_left = 3;
                guard.burst_timer = 0.0;
            }
        }

        if guard.burst_left > 0 {
            guard.burst_timer -= dt;
            if guard.burst_timer <= 0.0 {
                if let Some((target_pos, _)) = best {
                    let dir = (target_pos - g_pos.0).normalize_or(Vec2::X);
                    let ang = dir.y.atan2(dir.x);
                    g_tf.rotation = Quat::from_rotation_z(ang);
                    commands.spawn((
                        MatchEntity,
                        *g_side,
                        Position(g_pos.0 + dir * 10.0),
                        Projectile {
                            source_tower: guard.parent_tower,
                            velocity: dir * 560.0,
                            damage: if elite { 2 } else { 1 } + buff.0.damage_bonus,
                            pierce_left: if elite { 2 } else { 1 },
                            splash_radius: 0.0,
                            can_pop_lead: elite || buff.0.lead,
                            can_hit_camo: true,
                            knockback_chance: 0.0,
                            stun_duration: 0.0,
                            ceramic_bonus: 0,
                            moab_bonus: 0,
                            life: 0.9,
                            hit_ids: Vec::new(),
                        },
                        Sprite {
                            image: sprites.proj_dart.clone(),
                            color: if elite { Color::srgb(0.38, 0.82, 1.0) } else { Color::WHITE },
                            custom_size: Some(Vec2::splat(12.0)),
                            ..default()
                        },
                        Transform::from_xyz(g_pos.0.x, g_pos.0.y, 9.0)
                            .with_rotation(Quat::from_rotation_z(ang)),
                    ));
                }
                guard.burst_left -= 1;
                guard.burst_timer = 0.09;
            }
        }
    }
}

pub fn call_to_arms_fx_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut fx_q: Query<(Entity, &mut CallToArmsFx, &mut Sprite)>,
) {
    if settings.paused {
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
        let size = 24.0 + (fx.target_size - 24.0) * t;
        sprite.custom_size = Some(Vec2::splat(size));
        sprite.color = Color::srgba(0.98, 0.80, 0.24, 0.42 * (1.0 - t));
    }
}
