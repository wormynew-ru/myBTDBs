use bevy::prelude::*;

use crate::core::types::{Position, Side, TowerType};
use crate::data::tower_stats::{compute_effective_stats, VillageBuff};

use super::commander::CommanderState;
use super::scrap_collector::ScrapCollectorState;
use super::{Tower, TowerBuff};

#[allow(clippy::type_complexity)]
pub fn update_tower_buffs_system(
    mut towers: Query<(
        Entity,
        &Side,
        &Tower,
        &Position,
        &mut TowerBuff,
        Option<&CommanderState>,
        Option<&ScrapCollectorState>,
    )>,
) {
    let mut villages: Vec<(Entity, Side, Vec2, f32, VillageBuff)> = Vec::new();
    let mut commanders: Vec<(Entity, Side, Vec2, f32, VillageBuff)> = Vec::new();
    let mut scraps: Vec<(Entity, Side, Vec2, f32, f32, u32)> = Vec::new();

    for (e, side, t, pos, _, cmd, scrap) in towers.iter() {
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
        } else if t.tower_type == TowerType::ScrapCollector {
            let ratio = scrap.map_or(0.0, |sc| (sc.heat / 10.0).clamp(0.0, 1.0));
            let speed_bonus = ratio * s.scrap_max_speed_bonus;
            let pierce_b = if ratio >= 0.5 { s.scrap_pierce_bonus } else { 0 };
            if speed_bonus > 0.001 || pierce_b > 0 {
                scraps.push((e, *side, pos.0, s.range, 1.0 + speed_bonus, pierce_b));
            }
        }
    }

    for (entity, side, tower, pos, mut buff, _, _) in &mut towers {
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

        let mut s_speed = 1.0_f32;
        let mut s_pierce = 0_u32;
        for (se, ss, sp, sr, spd, prc) in &scraps {
            if *se == entity || *ss != *side || pos.0.distance(*sp) > *sr {
                continue;
            }
            s_speed = s_speed.max(*spd);
            s_pierce = s_pierce.max(*prc);
        }

        let jam_range_mult = if tower.jammed_timer > 0.0 {
            1.0 / 3.0
        } else {
            1.0
        };

        buff.0 = VillageBuff {
            camo: v_comb.camo,
            lead: v_comb.lead || c_lead,
            speed_mult: v_comb.speed_mult * c_speed * s_speed,
            range_mult: v_comb.range_mult * jam_range_mult,
            pierce_bonus: v_comb.pierce_bonus + s_pierce,
            damage_bonus: c_dmg,
        };
    }
}
