use bevy::prelude::*;

use crate::core::state::MatchSettings;
use crate::core::types::{MatchEntity, Position, Side, SidesResource, TowerType};
use crate::data::tower_stats::compute_effective_stats;

use super::Tower;

#[derive(Component, Debug, Clone)]
pub struct FarmState {
    pub harvest_timer: f32,
    pub dividend_timer: f32,
}

impl Default for FarmState {
    fn default() -> Self {
        Self {
            harvest_timer: 4.5,
            dividend_timer: 15.0,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct FarmIncomeFx {
    pub life: f32,
    pub max_life: f32,
}

pub fn farm_tick_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut sides: ResMut<SidesResource>,
    mut farms: Query<(&Side, &mut Tower, &Position, &mut FarmState)>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;

    for (side, mut tower, pos, mut state) in &mut farms {
        if tower.tower_type != TowerType::Farm {
            continue;
        }
        let stats = compute_effective_stats(tower.tower_type, tower.path1, tower.path2);

        if state.harvest_timer > stats.farm_interval {
            state.harvest_timer = stats.farm_interval;
        }
        state.harvest_timer -= dt;
        if state.harvest_timer <= 0.0 {
            state.harvest_timer += stats.farm_interval;
            sides.get_mut(*side).cash += stats.farm_cash;
            tower.pops += stats.farm_cash as u32;
            spawn_income_fx(
                &mut commands,
                pos.0 + Vec2::new(0.0, 18.0),
                stats.farm_cash,
                Color::srgb(0.22, 0.96, 0.42),
            );
        }

        if stats.farm_dividend_cash > 0 {
            if state.dividend_timer > stats.farm_dividend_interval {
                state.dividend_timer = stats.farm_dividend_interval;
            }
            state.dividend_timer -= dt;
            if state.dividend_timer <= 0.0 {
                state.dividend_timer += stats.farm_dividend_interval;
                sides.get_mut(*side).cash += stats.farm_dividend_cash;
                tower.pops += stats.farm_dividend_cash as u32;
                spawn_income_fx(
                    &mut commands,
                    pos.0 + Vec2::new(0.0, 26.0),
                    stats.farm_dividend_cash,
                    Color::srgb(0.98, 0.88, 0.24),
                );
            }
        }
    }
}

fn spawn_income_fx(commands: &mut Commands, pos: Vec2, amount: i32, color: Color) {
    commands.spawn((
        MatchEntity,
        FarmIncomeFx {
            life: 1.0,
            max_life: 1.0,
        },
        Text2d::new(format!("+{}$", amount)),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(pos.x, pos.y, 12.0),
    ));
}

pub fn farm_income_fx_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut fx_q: Query<(Entity, &mut FarmIncomeFx, &mut Transform, &mut TextColor)>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;
    for (e, mut fx, mut tf, mut color) in &mut fx_q {
        fx.life -= dt;
        if fx.life <= 0.0 {
            commands.entity(e).despawn_recursive();
            continue;
        }
        tf.translation.y += 24.0 * dt;
        let alpha = (fx.life / fx.max_life).clamp(0.0, 1.0);
        color.0 = color.0.with_alpha(alpha);
    }
}
