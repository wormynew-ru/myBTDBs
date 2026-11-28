use bevy::prelude::*;

use crate::core::events::BloonPoppedEvent;
use crate::core::state::MatchSettings;
use crate::core::types::{MatchEntity, Position, Side, SidesResource, TowerType};
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats};

use super::{Tower, TowerBuff};

#[derive(Component, Debug, Clone, Default)]
pub struct ScrapCollectorState {
    pub heat: f32,
}

#[derive(Component, Debug, Clone)]
pub struct ScrapIncomeFx {
    pub life: f32,
    pub max_life: f32,
}

#[allow(clippy::type_complexity)]
pub fn scrap_collector_system(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut sides: ResMut<SidesResource>,
    mut collectors: Query<(
        Entity,
        &Side,
        &mut Tower,
        &TowerBuff,
        &Position,
        &mut ScrapCollectorState,
    )>,
    mut fx_q: Query<(Entity, &mut ScrapIncomeFx, &mut Transform, &mut TextColor)>,
    mut pop_reader: EventReader<BloonPoppedEvent>,
) {
    if settings.paused {
        pop_reader.clear();
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;

    for (e, mut fx, mut tf, mut color) in &mut fx_q {
        fx.life -= dt;
        if fx.life <= 0.0 {
            commands.entity(e).despawn_recursive();
            continue;
        }
        tf.translation.y += 22.0 * dt;
        let alpha = (fx.life / fx.max_life).clamp(0.0, 1.0);
        color.0 = color.0.with_alpha(alpha);
    }

    for (_, _, tower, buff, _, mut state) in &mut collectors {
        if tower.tower_type != TowerType::ScrapCollector {
            continue;
        }
        let stats = apply_village_buff(
            compute_effective_stats(tower.tower_type, tower.path1, tower.path2),
            buff.0,
        );
        if state.heat > 0.0 {
            state.heat = (state.heat - stats.scrap_heat_decay * dt).max(0.0);
        }
    }

    for ev in pop_reader.read() {
        if !ev.from_send || ev.pops == 0 {
            continue;
        }
        let mut best: Option<(Entity, f32)> = None;
        for (e, side, tower, buff, pos, _) in collectors.iter() {
            if *side != ev.side || tower.tower_type != TowerType::ScrapCollector {
                continue;
            }
            let stats = apply_village_buff(
                compute_effective_stats(tower.tower_type, tower.path1, tower.path2),
                buff.0,
            );
            let d = pos.0.distance(ev.pos);
            if d <= stats.range && best.map_or(true, |(_, bd)| d < bd) {
                best = Some((e, d));
            }
        }
        let Some((e, _)) = best else {
            continue;
        };
        if let Ok((_, side, mut tower, buff, pos, mut state)) = collectors.get_mut(e) {
            let stats = apply_village_buff(
                compute_effective_stats(tower.tower_type, tower.path1, tower.path2),
                buff.0,
            );
            state.heat = (state.heat + ev.pops as f32).min(12.0);
            if ev.killed {
                let cash = if ev.is_moab {
                    stats.scrap_moab_cash
                } else {
                    stats.scrap_cash
                };
                sides.get_mut(*side).cash += cash;
                tower.pops += cash as u32;
                if ev.is_moab || cash >= 5 {
                    commands.spawn((
                        MatchEntity,
                        ScrapIncomeFx {
                            life: 0.75,
                            max_life: 0.75,
                        },
                        Text2d::new(format!("+${}", cash)),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.98, 0.72, 0.22)),
                        Transform::from_xyz(pos.0.x, pos.0.y + 18.0, 11.8),
                    ));
                }
            }
        }
    }
}
