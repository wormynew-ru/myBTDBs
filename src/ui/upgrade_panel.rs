use bevy::prelude::*;

use crate::core::events::{
    ActivateAbilityEvent, ChangePriorityEvent, SellTowerEvent, UpgradeTowerEvent,
};
use crate::core::state::{GameState, SelectedTower};
use crate::core::types::{Side, TowerType};
use crate::data::towers_cfg::{tower_cfg, SELL_REFUND_RATIO};
use crate::plugins::towers::{ChipperState, Tower, TowerBuff};

pub struct UpgradePanelPlugin;

impl Plugin for UpgradePanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_upgrade_panel)
            .add_systems(OnExit(GameState::Playing), despawn_upgrade_panel)
            .add_systems(
                Update,
                (sync_upgrade_panel, handle_upgrade_panel_buttons)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
struct UpgradePanelRoot;

#[derive(Component)]
struct PanelTitleText;

#[derive(Component)]
struct Path1BtnText;

#[derive(Component)]
struct Path2BtnText;

#[derive(Component)]
struct PriorityBtnText;

#[derive(Component)]
struct AbilityBtn;

#[derive(Component)]
struct AbilityBtnText;

#[derive(Component)]
struct SellBtnText;

#[derive(Component)]
enum UpgradePanelAction {
    UpgradePath(u8),
    CyclePriority,
    ActivateAbility,
    Sell,
    Close,
}

fn spawn_upgrade_panel(mut commands: Commands) {
    commands
        .spawn((
            UpgradePanelRoot,
            Node {
                width: Val::Px(255.0),
                position_type: PositionType::Absolute,
                right: Val::Px(12.0),
                top: Val::Px(68.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(7.0),
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor(Color::srgb(0.32, 0.40, 0.54)),
            BorderRadius::all(Val::Px(12.0)),
            BackgroundColor(Color::srgba(0.06, 0.09, 0.15, 0.95)),
            Visibility::Hidden,
        ))
        .with_children(|panel| {
            panel.spawn((
                PanelTitleText,
                Text::new("Tower"),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.98, 0.80, 0.24)),
            ));

            spawn_panel_btn::<Path1BtnText>(
                panel,
                "Path 1",
                UpgradePanelAction::UpgradePath(1),
                Color::srgb(0.16, 0.22, 0.34),
            );
            spawn_panel_btn::<Path2BtnText>(
                panel,
                "Path 2",
                UpgradePanelAction::UpgradePath(2),
                Color::srgb(0.16, 0.22, 0.34),
            );
            spawn_panel_btn::<PriorityBtnText>(
                panel,
                "Target: First",
                UpgradePanelAction::CyclePriority,
                Color::srgb(0.20, 0.25, 0.36),
            );

            panel
                .spawn((
                    Button,
                    AbilityBtn,
                    UpgradePanelAction::ActivateAbility,
                    Node {
                        width: Val::Percent(100.0),
                        min_height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(8.0)),
                    BackgroundColor(Color::srgb(0.45, 0.16, 0.72)),
                    Visibility::Hidden,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        AbilityBtnText,
                        Text::new("Supa-Vac"),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            spawn_panel_btn::<SellBtnText>(
                panel,
                "Sell",
                UpgradePanelAction::Sell,
                Color::srgb(0.45, 0.14, 0.18),
            );

            panel
                .spawn((
                    Button,
                    UpgradePanelAction::Close,
                    Node {
                        width: Val::Percent(100.0),
                        min_height: Val::Px(48.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderRadius::all(Val::Px(8.0)),
                    BackgroundColor(Color::srgb(0.18, 0.22, 0.30)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Close"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn spawn_panel_btn<M: Component + Default>(
    parent: &mut ChildBuilder,
    initial: &str,
    action: UpgradePanelAction,
    bg: Color,
) {
    parent
        .spawn((
            Button,
            action,
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(bg),
        ))
        .with_children(|btn| {
            btn.spawn((
                M::default(),
                Text::new(initial),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

impl Default for Path1BtnText { fn default() -> Self { Self } }
impl Default for Path2BtnText { fn default() -> Self { Self } }
impl Default for PriorityBtnText { fn default() -> Self { Self } }
impl Default for SellBtnText { fn default() -> Self { Self } }

fn despawn_upgrade_panel(mut commands: Commands, query: Query<Entity, With<UpgradePanelRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
fn sync_upgrade_panel(
    selected: Res<SelectedTower>,
    towers: Query<(&Tower, &TowerBuff, Option<&ChipperState>)>,
    mut panel_q: Query<&mut Visibility, With<UpgradePanelRoot>>,
    mut ability_vis_q: Query<&mut Visibility, (With<AbilityBtn>, Without<UpgradePanelRoot>)>,
    mut title_q: Query<&mut Text, (With<PanelTitleText>, Without<Path1BtnText>, Without<Path2BtnText>, Without<PriorityBtnText>, Without<SellBtnText>, Without<AbilityBtnText>)>,
    mut p1_q: Query<&mut Text, (With<Path1BtnText>, Without<PanelTitleText>, Without<Path2BtnText>, Without<PriorityBtnText>, Without<SellBtnText>, Without<AbilityBtnText>)>,
    mut p2_q: Query<&mut Text, (With<Path2BtnText>, Without<PanelTitleText>, Without<Path1BtnText>, Without<PriorityBtnText>, Without<SellBtnText>, Without<AbilityBtnText>)>,
    mut prio_q: Query<&mut Text, (With<PriorityBtnText>, Without<PanelTitleText>, Without<Path1BtnText>, Without<Path2BtnText>, Without<SellBtnText>, Without<AbilityBtnText>)>,
    mut sell_q: Query<&mut Text, (With<SellBtnText>, Without<PanelTitleText>, Without<Path1BtnText>, Without<Path2BtnText>, Without<PriorityBtnText>, Without<AbilityBtnText>)>,
    mut ab_text_q: Query<&mut Text, (With<AbilityBtnText>, Without<PanelTitleText>, Without<Path1BtnText>, Without<Path2BtnText>, Without<PriorityBtnText>, Without<SellBtnText>)>,
) {
    let Ok(mut vis) = panel_q.get_single_mut() else {
        return;
    };
    let Some(entity) = selected.0 else {
        *vis = Visibility::Hidden;
        return;
    };
    let Ok((tower, buff, chipper)) = towers.get(entity) else {
        *vis = Visibility::Hidden;
        return;
    };

    *vis = Visibility::Visible;
    let cfg = tower_cfg(tower.tower_type);

    if let Ok(mut t) = title_q.get_single_mut() {
        if tower.tower_type == TowerType::Farm {
            **t = format!("{} | Earned: ${}", cfg.name, tower.pops);
        } else {
            let camo_tag = if buff.0.camo { " [CAMO]" } else { "" };
            let lead_tag = if buff.0.lead { " [LEAD]" } else { "" };
            **t = format!("{} | Pops: {}{}{}", cfg.name, tower.pops, camo_tag, lead_tag);
        }
    }
    if let Ok(mut t) = p1_q.get_single_mut() {
        let max_t = cfg.path1.len();
        **t = if (tower.path1 as usize) < max_t {
            let u = cfg.path1[tower.path1 as usize];
            format!("P1 ({}/{}): {} ${}", tower.path1, max_t, u.name, u.cost)
        } else {
            format!("P1 ({}/{}): MAXED", max_t, max_t)
        };
    }
    if let Ok(mut t) = p2_q.get_single_mut() {
        let max_t = cfg.path2.len();
        **t = if (tower.path2 as usize) < max_t {
            let u = cfg.path2[tower.path2 as usize];
            format!("P2 ({}/{}): {} ${}", tower.path2, max_t, u.name, u.cost)
        } else {
            format!("P2 ({}/{}): MAXED", max_t, max_t)
        };
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
        } else {
            *ab_vis = Visibility::Hidden;
        }
    }
    if let Ok(mut t) = sell_q.get_single_mut() {
        let refund = ((tower.total_invested as f32) * SELL_REFUND_RATIO).round() as i32;
        **t = format!("Sell (+${})", refund);
    }
}

fn handle_upgrade_panel_buttons(
    mut selected: ResMut<SelectedTower>,
    towers: Query<&Tower>,
    interactions: Query<(&Interaction, &UpgradePanelAction), Changed<Interaction>>,
    mut upgrade_writer: EventWriter<UpgradeTowerEvent>,
    mut prio_writer: EventWriter<ChangePriorityEvent>,
    mut ability_writer: EventWriter<ActivateAbilityEvent>,
    mut sell_writer: EventWriter<SellTowerEvent>,
) {
    let Some(tower_entity) = selected.0 else {
        return;
    };

    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            UpgradePanelAction::UpgradePath(branch) => {
                upgrade_writer.send(UpgradeTowerEvent {
                    side: Side::Player,
                    tower_entity,
                    branch,
                });
            }
            UpgradePanelAction::CyclePriority => {
                if let Ok(tower) = towers.get(tower_entity) {
                    prio_writer.send(ChangePriorityEvent {
                        tower_entity,
                        priority: tower.priority.next(),
                    });
                }
            }
            UpgradePanelAction::ActivateAbility => {
                ability_writer.send(ActivateAbilityEvent {
                    side: Side::Player,
                    tower_entity,
                });
            }
            UpgradePanelAction::Sell => {
                sell_writer.send(SellTowerEvent {
                    side: Side::Player,
                    tower_entity,
                });
            }
            UpgradePanelAction::Close => {
                selected.0 = None;
            }
        }
    }
}
