use bevy::prelude::*;

use crate::core::state::SelectedTower;
use crate::core::types::TowerType;
use crate::data::tower_stats::{apply_village_buff, compute_effective_stats};
use crate::data::towers_cfg::{tower_cfg, SELL_REFUND_RATIO};
use crate::plugins::towers::{
    ChipperState, GlueState, MirageGeneratorState, NecromancerState, SaboteurState,
    ScrapCollectorState, SoulEaterState, Tower, TowerBuff,
};

use super::{
    AbilityBtn, AbilityBtnText, PanelTitleText, Path1BtnText, Path2BtnText, PriorityBtnText,
    SellBtnText, UpgradePanelRoot,
};

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn sync_upgrade_panel(
    selected: Res<SelectedTower>,
    towers: Query<(
        &Tower,
        &TowerBuff,
        Option<&ChipperState>,
        Option<&GlueState>,
        Option<&SoulEaterState>,
        Option<&NecromancerState>,
        Option<&SaboteurState>,
        Option<&ScrapCollectorState>,
        Option<&MirageGeneratorState>,
    )>,
    mut panel_q: Query<&mut Visibility, With<UpgradePanelRoot>>,
    mut ability_vis_q: Query<&mut Visibility, (With<AbilityBtn>, Without<UpgradePanelRoot>)>,
    mut title_q: Query<
        &mut Text,
        (
            With<PanelTitleText>,
            Without<Path1BtnText>,
            Without<Path2BtnText>,
            Without<PriorityBtnText>,
            Without<SellBtnText>,
            Without<AbilityBtnText>,
        ),
    >,
    mut p1_q: Query<
        &mut Text,
        (
            With<Path1BtnText>,
            Without<PanelTitleText>,
            Without<Path2BtnText>,
            Without<PriorityBtnText>,
            Without<SellBtnText>,
            Without<AbilityBtnText>,
        ),
    >,
    mut p2_q: Query<
        &mut Text,
        (
            With<Path2BtnText>,
            Without<PanelTitleText>,
            Without<Path1BtnText>,
            Without<PriorityBtnText>,
            Without<SellBtnText>,
            Without<AbilityBtnText>,
        ),
    >,
    mut prio_q: Query<
        &mut Text,
        (
            With<PriorityBtnText>,
            Without<PanelTitleText>,
            Without<Path1BtnText>,
            Without<Path2BtnText>,
            Without<SellBtnText>,
            Without<AbilityBtnText>,
        ),
    >,
    mut sell_q: Query<
        &mut Text,
        (
            With<SellBtnText>,
            Without<PanelTitleText>,
            Without<Path1BtnText>,
            Without<Path2BtnText>,
            Without<PriorityBtnText>,
            Without<AbilityBtnText>,
        ),
    >,
    mut ab_text_q: Query<
        &mut Text,
        (
            With<AbilityBtnText>,
            Without<PanelTitleText>,
            Without<Path1BtnText>,
            Without<Path2BtnText>,
            Without<PriorityBtnText>,
            Without<SellBtnText>,
        ),
    >,
) {
    let Ok(mut vis) = panel_q.get_single_mut() else {
        return;
    };
    let Some(entity) = selected.0 else {
        *vis = Visibility::Hidden;
        return;
    };
    let Ok((tower, buff, chipper, glue, soul_eater, necro, saboteur, scrap, mirage)) =
        towers.get(entity)
    else {
        *vis = Visibility::Hidden;
        return;
    };

    *vis = Visibility::Visible;
    let cfg = tower_cfg(tower.tower_type);
    let jam_tag = if tower.jammed_timer > 0.0 {
        format!(" [JAM {:.1}s]", tower.jammed_timer)
    } else {
        String::new()
    };

    if let Ok(mut t) = title_q.get_single_mut() {
        if tower.tower_type == TowerType::Farm {
            **t = format!("{} | Earned: ${}{}", cfg.name, tower.pops, jam_tag);
        } else if let Some(se) = soul_eater {
            **t = format!("{} | Souls: {}/{}{}", cfg.name, se.souls, se.target, jam_tag);
        } else if let Some(n) = necro {
            let cap = compute_effective_stats(tower.tower_type, tower.path1, tower.path2).soul_capacity;
            **t = format!(
                "{} | Souls: {}/{} | Pops: {}{}",
                cfg.name, n.souls, cap, tower.pops, jam_tag
            );
        } else if let Some(sc) = scrap {
            let stats = apply_village_buff(
                compute_effective_stats(tower.tower_type, tower.path1, tower.path2),
                buff.0,
            );
            let aura_pct = ((sc.heat / 12.0).clamp(0.0, 1.0)
                * stats.scrap_max_speed_bonus
                * 100.0)
                .round() as i32;
            **t = format!(
                "{} | Salvage: ${} (+{}% Spd){}",
                cfg.name, tower.pops, aura_pct, jam_tag
            );
        } else if let Some(mg) = mirage {
            **t = format!(
                "{} | Mirage: {:.1}s{}",
                cfg.name,
                mg.spawn_timer.max(0.0),
                jam_tag
            );
        } else {
            let camo_tag = if buff.0.camo { " [CAMO]" } else { "" };
            let lead_tag = if buff.0.lead { " [LEAD]" } else { "" };
            **t = format!(
                "{} | Pops: {}{}{}{}",
                cfg.name, tower.pops, camo_tag, lead_tag, jam_tag
            );
        }
    }

    if let Ok(mut t) = p1_q.get_single_mut() {
        if let Some(se) = soul_eater {
            **t = format!("Absorbed: {} / {}", se.souls, se.target);
        } else if tower.jammed_timer > 0.0 {
            **t = format!("JAMMED ({:.1}s)", tower.jammed_timer);
        } else {
            let max_t = cfg.path1.len();
            **t = if (tower.path1 as usize) < max_t {
                let u = cfg.path1[tower.path1 as usize];
                format!("P1 ({}/{}): {} ${}", tower.path1, max_t, u.name, u.cost)
            } else {
                format!("P1 ({}/{}): MAXED", max_t, max_t)
            };
        }
    }
    if let Ok(mut t) = p2_q.get_single_mut() {
        if soul_eater.is_some() {
            **t = "Evolves to MAX Tower".to_string();
        } else if tower.jammed_timer > 0.0 {
            **t = format!("JAMMED ({:.1}s)", tower.jammed_timer);
        } else {
            let max_t = cfg.path2.len();
            **t = if (tower.path2 as usize) < max_t {
                let u = cfg.path2[tower.path2 as usize];
                format!("P2 ({}/{}): {} ${}", tower.path2, max_t, u.name, u.cost)
            } else {
                format!("P2 ({}/{}): MAXED", max_t, max_t)
            };
        }
    }
    if let Ok(mut t) = prio_q.get_single_mut() {
        **t = format!("Target: {}", tower.priority.label());
    }

    if let Ok(mut ab_vis) = ability_vis_q.get_single_mut() {
        if tower.tower_type == TowerType::Bloonchipper && tower.path2 >= 4 {
            *ab_vis = Visibility::Inherited;
            if let (Ok(mut ab_t), Some(c)) = (ab_text_q.get_single_mut(), chipper) {
                **ab_t = if c.supa_vac_active > 0.0 {
                    format!("Supa-Vac ACTIVE ({:.1}s)", c.supa_vac_active)
                } else if c.supa_vac_cooldown > 0.0 {
                    format!("Supa-Vac CD: {:.1}s", c.supa_vac_cooldown)
                } else {
                    "ACTIVATE SUPA-VAC".to_string()
                };
            }
        } else if tower.tower_type == TowerType::GlueGunner && tower.path2 >= 4 {
            *ab_vis = Visibility::Inherited;
            if let (Ok(mut ab_t), Some(g)) = (ab_text_q.get_single_mut(), glue) {
                **ab_t = if g.striker_cooldown > 0.0 {
                    format!("Glue Striker CD: {:.1}s", g.striker_cooldown)
                } else {
                    "ACTIVATE GLUE STRIKER".to_string()
                };
            }
        } else if tower.tower_type == TowerType::Saboteur {
            *ab_vis = Visibility::Inherited;
            if let (Ok(mut ab_t), Some(s)) = (ab_text_q.get_single_mut(), saboteur) {
                **ab_t = if s.ability_cooldown > 0.0 {
                    format!("Jammer CD: {:.1}s", s.ability_cooldown)
                } else {
                    "ACTIVATE JAMMER".to_string()
                };
            }
        } else {
            *ab_vis = Visibility::Hidden;
        }
    }

    if let Ok(mut t) = sell_q.get_single_mut() {
        let refund = ((tower.total_invested as f32) * SELL_REFUND_RATIO).round() as i32;
        **t = format!("Sell (+${})", refund);
    }
}
