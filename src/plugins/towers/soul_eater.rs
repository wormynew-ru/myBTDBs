use bevy::prelude::*;

use crate::core::events::BloonPoppedEvent;
use crate::core::state::MatchSettings;
use crate::core::types::{MatchEntity, Position, Side, TowerType};
use crate::data::towers_cfg::tower_cfg;
use crate::plugins::sprites::SpriteAssets;

use super::chipper::ChipperState;
use super::commander::CommanderState;
use super::crook_boss::CrookBossState;
use super::farm::FarmState;
use super::glue::GlueState;
use super::military_base::MilitaryBaseState;
use super::mirage::MirageGeneratorState;
use super::necromancer::NecromancerState;
use super::saboteur::SaboteurState;
use super::scrap_collector::ScrapCollectorState;
use super::spikes::SpikeFactoryState;
use super::Tower;

pub const SOUL_EATER_TARGET: u32 = 350;

#[derive(Component, Debug, Clone)]
pub struct SoulEaterState {
    pub souls: u32,
    pub target: u32,
}

impl Default for SoulEaterState {
    fn default() -> Self {
        Self {
            souls: 0,
            target: SOUL_EATER_TARGET,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct SoulEvolveFx {
    pub life: f32,
    pub max_life: f32,
}

fn next_rand(seed: u32) -> u32 {
    let mut x = seed | 1;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}

#[allow(clippy::too_many_arguments)]
pub fn soul_eater_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    sprites: Res<SpriteAssets>,
    mut eaters: Query<(
        Entity,
        &Side,
        &mut Tower,
        &Position,
        &mut Sprite,
        &mut Transform,
        &mut SoulEaterState,
    )>,
    mut fx_q: Query<(Entity, &mut SoulEvolveFx, &mut Sprite), (Without<SoulEaterState>, Without<Tower>)>,
    mut pop_reader: EventReader<BloonPoppedEvent>,
) {
    if settings.paused {
        pop_reader.clear();
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
        sprite.custom_size = Some(Vec2::splat(24.0 + 110.0 * t));
        sprite.color = Color::srgba(0.95, 0.55, 0.98, 0.85 * (1.0 - t));
    }

    for ev in pop_reader.read() {
        if ev.pops == 0 {
            continue;
        }
        let mut best: Option<(Entity, f32)> = None;
        for (e, side, tower, pos, _, _, state) in eaters.iter() {
            if *side != ev.side
                || tower.tower_type != TowerType::SoulEater
                || state.souls >= state.target
            {
                continue;
            }
            let d = pos.0.distance(ev.pos);
            if d <= 110.0 && best.map_or(true, |(_, bd)| d < bd) {
                best = Some((e, d));
            }
        }
        if let Some((e, _)) = best {
            if let Ok((_, _, mut tower, _, _, _, mut state)) = eaters.get_mut(e) {
                state.souls = (state.souls + ev.pops).min(state.target);
                tower.pops = state.souls;
            }
        }
    }

    let time_bits = time.elapsed_secs().to_bits();
    for (e, side, mut tower, pos, mut sprite, mut tf, state) in &mut eaters {
        if tower.tower_type != TowerType::SoulEater || state.souls < state.target {
            continue;
        }

        let loadout = match side {
            Side::Player => &settings.player_loadout,
            Side::Bot => &settings.bot_loadout,
        };
        let candidates: Vec<TowerType> = loadout
            .iter()
            .copied()
            .filter(|&t| t != TowerType::SoulEater)
            .collect();
        let pool = if candidates.is_empty() {
            vec![TowerType::DartMonkey]
        } else {
            candidates
        };

        let seed = time_bits
            ^ pos.0.x.to_bits().rotate_left(7)
            ^ pos.0.y.to_bits().rotate_left(19)
            ^ (e.index().wrapping_mul(2654435761));
        let r1 = next_rand(seed);
        let r2 = next_rand(r1);
        let new_type = pool[(r1 as usize) % pool.len()];
        let cfg = tower_cfg(new_type);
        let (p1, p2) = if (r2 & 1) == 0 && !cfg.path1.is_empty() {
            (cfg.path1.len() as u8, 0)
        } else if !cfg.path2.is_empty() {
            (0, cfg.path2.len() as u8)
        } else {
            (cfg.path1.len() as u8, 0)
        };

        tower.tower_type = new_type;
        tower.path1 = p1;
        tower.path2 = p2;
        tower.cooldown = 0.1;
        tower.pops = 0;
        sprite.image = sprites.tower_handle(new_type, p1, p2);
        tf.rotation = Quat::IDENTITY;

        let mut cmd = commands.entity(e);
        cmd.remove::<SoulEaterState>();
        match new_type {
            TowerType::Bloonchipper => { cmd.insert(ChipperState::default()); }
            TowerType::SpikeFactory => {
                cmd.insert(SpikeFactoryState { storm_timer: 0.5, rng_state: seed | 1 });
            }
            TowerType::Commander => { cmd.insert(CommanderState::default()); }
            TowerType::MilitaryBase => { cmd.insert(MilitaryBaseState::default()); }
            TowerType::CrookBoss => { cmd.insert(CrookBossState::default()); }
            TowerType::Farm => { cmd.insert(FarmState::default()); }
            TowerType::GlueGunner => { cmd.insert(GlueState::default()); }
            TowerType::Necromancer => { cmd.insert(NecromancerState::default()); }
            TowerType::Saboteur => { cmd.insert(SaboteurState { rng_state: seed | 1, ..default() }); }
            TowerType::ScrapCollector => { cmd.insert(ScrapCollectorState::default()); }
            TowerType::MirageGenerator => { cmd.insert(MirageGeneratorState::default()); }
            _ => {}
        }

        commands.spawn((
            MatchEntity,
            SoulEvolveFx { life: 0.45, max_life: 0.45 },
            Sprite::from_color(Color::srgba(0.95, 0.55, 0.98, 0.85), Vec2::splat(24.0)),
            Transform::from_xyz(pos.0.x, pos.0.y, 9.6),
        ));
    }
}
