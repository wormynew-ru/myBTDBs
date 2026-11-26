use bevy::prelude::*;

use crate::core::events::SendBloonPackEvent;
use crate::core::state::{BuildSelection, GameState, SelectedTower};
use crate::core::types::{Side, SidesResource, TowerType};
use crate::data::bloons_cfg::SEND_PACKS;
use crate::data::towers_cfg::{tower_cfg, ALL_TOWERS};

pub struct BuildBarPlugin;

impl Plugin for BuildBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_build_bar)
            .add_systems(OnExit(GameState::Playing), despawn_build_bar)
            .add_systems(
                Update,
                (handle_build_bar_buttons, update_build_bar_styles)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
struct BuildBarRoot;

#[derive(Component)]
enum BuildBarAction {
    SelectTower(TowerType),
    SendPack(usize),
}

fn spawn_build_bar(mut commands: Commands) {
    commands
        .spawn((
            BuildBarRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(74.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                column_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.08, 0.14, 0.94)),
        ))
        .with_children(|bar| {
            // Left: Tower buy buttons
            bar.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|row| {
                for tower_type in ALL_TOWERS {
                    let cfg = tower_cfg(tower_type);
                    let label = format!("{}\n${}", cfg.short_name, cfg.cost);
                    row.spawn((
                        Button,
                        BuildBarAction::SelectTower(tower_type),
                        Node {
                            min_width: Val::Px(78.0),
                            min_height: Val::Px(56.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor(Color::srgb(0.30, 0.36, 0.48)),
                        BorderRadius::all(Val::Px(8.0)),
                        BackgroundColor(cfg.color),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                }
            });

            // Right: Send Bloon packs
            bar.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(5.0),
                ..default()
            })
            .with_children(|row| {
                for (idx, pack) in SEND_PACKS.iter().enumerate() {
                    let label = format!("{}\n${} +{}", pack.short_label, pack.cost, pack.eco_gain);
                    row.spawn((
                        Button,
                        BuildBarAction::SendPack(idx),
                        Node {
                            min_width: Val::Px(82.0),
                            min_height: Val::Px(56.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor(pack.badge_color),
                        BorderRadius::all(Val::Px(8.0)),
                        BackgroundColor(Color::srgb(0.11, 0.15, 0.24)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                }
            });
        });
}

fn despawn_build_bar(mut commands: Commands, query: Query<Entity, With<BuildBarRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn handle_build_bar_buttons(
    mut build_sel: ResMut<BuildSelection>,
    mut selected_tower: ResMut<SelectedTower>,
    mut send_writer: EventWriter<SendBloonPackEvent>,
    interactions: Query<(&Interaction, &BuildBarAction), Changed<Interaction>>,
) {
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            BuildBarAction::SelectTower(t) => {
                if build_sel.0 == Some(t) {
                    build_sel.0 = None;
                } else {
                    build_sel.0 = Some(t);
                    selected_tower.0 = None;
                }
            }
            BuildBarAction::SendPack(idx) => {
                send_writer.send(SendBloonPackEvent {
                    sender: Side::Player,
                    pack_index: idx,
                });
            }
        }
    }
}

fn update_build_bar_styles(
    build_sel: Res<BuildSelection>,
    sides: Res<SidesResource>,
    mut buttons: Query<(&BuildBarAction, &mut BorderColor, &mut BackgroundColor)>,
) {
    for (action, mut border, mut bg) in &mut buttons {
        match *action {
            BuildBarAction::SelectTower(t) => {
                let cfg = tower_cfg(t);
                let active = build_sel.0 == Some(t);
                let affordable = sides.player.cash >= cfg.cost;
                border.0 = if active {
                    Color::srgb(0.98, 0.84, 0.22)
                } else if affordable {
                    Color::srgb(0.45, 0.55, 0.68)
                } else {
                    Color::srgb(0.22, 0.22, 0.26)
                };
                bg.0 = if affordable || active {
                    cfg.color
                } else {
                    Color::srgb(0.14, 0.16, 0.20)
                };
            }
            BuildBarAction::SendPack(idx) => {
                if let Some(pack) = SEND_PACKS.get(idx) {
                    let unlocked = sides.wave >= pack.unlock_wave;
                    let affordable = sides.player.cash >= pack.cost;
                    bg.0 = if unlocked && affordable {
                        Color::srgb(0.12, 0.18, 0.28)
                    } else {
                        Color::srgb(0.07, 0.09, 0.14)
                    };
                }
            }
        }
    }
}
