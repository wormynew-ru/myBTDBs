use bevy::prelude::*;

use crate::core::events::{BloonPoppedEvent, SpawnBloonEvent};
use crate::core::state::MatchSettings;
use crate::core::types::{BloonLayer, GlueStatus, Health, MatchEntity, Position, Side, TowerType};
use crate::data::maps_cfg::{ARENA_H, DIVIDER_HALF, HALF_W};
use crate::plugins::bloons::Bloon;

use super::combat_proj::apply_damage;
use super::Tower;

#[derive(Component, Debug, Clone, Default)]
pub struct GlueState {
    pub striker_cooldown: f32,
}

#[derive(Component, Debug, Clone)]
pub struct GlueStrikerFx {
    pub life: f32,
    pub max_life: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct GluePayload {
    pub duration: f32,
    pub soak: bool,
    pub corrosive_dmg: i32,
    pub corrosive_interval: f32,
    pub moab_capable: bool,
}

pub fn apply_glue_to_bloon(bloon: &mut Bloon, source_tower: Entity, g: GluePayload) {
    let stronger_acid = g.corrosive_dmg >= bloon.glue.corrosive_dmg;
    let corrosive_interval = if bloon.glue.corrosive_interval > 0.0 && g.corrosive_interval > 0.0 {
        bloon.glue.corrosive_interval.min(g.corrosive_interval)
    } else if g.corrosive_interval > 0.0 {
        g.corrosive_interval
    } else {
        bloon.glue.corrosive_interval
    };
    let tick = if bloon.glue.tick > 0.0 {
        if g.corrosive_interval > 0.0 {
            bloon.glue.tick.min(g.corrosive_interval)
        } else {
            bloon.glue.tick
        }
    } else {
        corrosive_interval
    };
    bloon.glue = GlueStatus {
        timer: bloon.glue.timer.max(g.duration),
        soak: bloon.glue.soak || g.soak,
        corrosive_dmg: bloon.glue.corrosive_dmg.max(g.corrosive_dmg),
        corrosive_interval,
        tick,
        source_tower: if stronger_acid || bloon.glue.source_tower.is_none() {
            Some(source_tower)
        } else {
            bloon.glue.source_tower
        },
    };
}

pub fn spawn_glue_striker_fx(commands: &mut Commands, side: Side) {
    let center_x = match side {
        Side::Player => -(HALF_W + DIVIDER_HALF) * 0.5,
        Side::Bot => (HALF_W + DIVIDER_HALF) * 0.5,
    };
    let width = HALF_W - DIVIDER_HALF;
    commands.spawn((
        MatchEntity,
        GlueStrikerFx {
            life: 0.45,
            max_life: 0.45,
        },
        Sprite::from_color(
            Color::srgba(0.65, 0.92, 0.18, 0.32),
            Vec2::new(width, ARENA_H),
        ),
        Transform::from_xyz(center_x, 0.0, 9.4),
    ));
}

#[allow(clippy::too_many_arguments)]
pub fn glue_corrosive_and_cooldown_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut glue_towers: Query<(&Tower, &mut GlueState)>,
    mut bloons: Query<
        (Entity, &Side, &mut Bloon, &mut Health, &Position),
        (Without<Tower>, Without<GlueState>),
    >,
    mut towers_mut: Query<&mut Tower, (Without<Bloon>, Without<GlueState>)>,
    mut fx_q: Query<(Entity, &mut GlueStrikerFx, &mut Sprite)>,
    mut spawn_writer: EventWriter<SpawnBloonEvent>,
    mut pop_writer: EventWriter<BloonPoppedEvent>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;

    for (tower, mut state) in &mut glue_towers {
        if tower.tower_type == TowerType::GlueGunner && state.striker_cooldown > 0.0 {
            state.striker_cooldown = (state.striker_cooldown - dt).max(0.0);
        }
    }

    for (e, mut fx, mut sprite) in &mut fx_q {
        fx.life -= dt;
        if fx.life <= 0.0 {
            commands.entity(e).despawn_recursive();
            continue;
        }
        let alpha = 0.35 * (fx.life / fx.max_life).clamp(0.0, 1.0);
        sprite.color = Color::srgba(0.65, 0.92, 0.18, alpha);
    }

    for (b_e, b_side, mut bloon, mut hp, b_pos) in &mut bloons {
        if hp.current <= 0
            || bloon.is_inside_chipper
            || bloon.glue.timer <= 0.0
            || bloon.glue.corrosive_dmg <= 0
            || bloon.glue.corrosive_interval <= 0.0
        {
            continue;
        }
        bloon.glue.tick -= dt;
        if bloon.glue.tick > 0.0 {
            continue;
        }
        bloon.glue.tick += bloon.glue.corrosive_interval;
        let bonus = if bloon.glue.corrosive_interval <= 0.5 && bloon.layer == BloonLayer::Ceramic {
            1
        } else {
            0
        };
        let dmg = bloon.glue.corrosive_dmg + bonus;
        let mut src = bloon
            .glue
            .source_tower
            .and_then(|e| towers_mut.get_mut(e).ok());
        apply_damage(
            &mut commands,
            b_e,
            *b_side,
            &bloon,
            b_pos.0,
            &mut hp,
            dmg,
            src.as_deref_mut(),
            &mut spawn_writer,
            &mut pop_writer,
        );
    }
}
