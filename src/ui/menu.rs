#[path = "menu_loadout.rs"]
mod menu_loadout;

use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;

use crate::core::state::{Difficulty, GameMode, GameState, MapId, MatchSettings};
use crate::data::towers_cfg::tower_cfg;

use menu_loadout::{
    randomize_player_loadout, spawn_loadout_section, LoadoutHeaderText, MenuAction,
    StartMatchBtnText,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(GameState::MainMenu), despawn_main_menu)
            .add_systems(
                Update,
                handle_menu_buttons.run_if(in_state(GameState::MainMenu)),
            );
    }
}

#[derive(Component)]
struct MenuRoot;

fn spawn_main_menu(mut commands: Commands, settings: Res<MatchSettings>) {
    let required = settings.game_mode.loadout_size();
    let ready = settings.player_loadout.len() == required;

    commands
        .spawn((
            MenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.09, 0.15)),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("myBTDBs — Bloons TD Battles PvE"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.98, 0.80, 0.24)),
            ));

            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                for diff in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
                    spawn_menu_button(
                        row,
                        diff.label(),
                        MenuAction::SetDifficulty(diff),
                        settings.difficulty == diff,
                        108.0,
                    );
                }
                row.spawn(Node {
                    width: Val::Px(14.0),
                    ..default()
                });
                for mode in [GameMode::Standard, GameMode::Turbo] {
                    spawn_menu_button(
                        row,
                        mode.label(),
                        MenuAction::SetMode(mode),
                        settings.game_mode == mode,
                        144.0,
                    );
                }
            });

            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|row| {
                for map_id in [MapId::Meadow, MapId::Zigzag, MapId::Canyon] {
                    spawn_menu_button(
                        row,
                        map_id.label(),
                        MenuAction::SetMap(map_id),
                        settings.map_id == map_id,
                        164.0,
                    );
                }
            });

            spawn_loadout_section(root, &settings);

            root.spawn((
                Button,
                MenuAction::StartMatch,
                Node {
                    min_width: Val::Px(240.0),
                    min_height: Val::Px(54.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(6.0)),
                    padding: UiRect::axes(Val::Px(22.0), Val::Px(10.0)),
                    ..default()
                },
                BorderRadius::all(Val::Px(10.0)),
                BackgroundColor(if ready {
                    Color::srgb(0.95, 0.65, 0.12)
                } else {
                    Color::srgb(0.24, 0.28, 0.36)
                }),
            ))
            .with_children(|btn| {
                let label = if ready {
                    "START 1v1 MATCH".to_string()
                } else {
                    format!("PICK {} MORE", required - settings.player_loadout.len())
                };
                btn.spawn((
                    StartMatchBtnText,
                    Text::new(label),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.06, 0.09, 0.14)),
                ));
            });
        });
}

fn spawn_menu_button(
    parent: &mut ChildBuilder,
    label: &str,
    action: MenuAction,
    active: bool,
    width: f32,
) {
    parent
        .spawn((
            Button,
            action,
            Node {
                min_width: Val::Px(width),
                min_height: Val::Px(48.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor(if active {
                Color::srgb(0.98, 0.80, 0.24)
            } else {
                Color::srgb(0.25, 0.30, 0.42)
            }),
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(if active {
                Color::srgb(0.20, 0.52, 0.88)
            } else {
                Color::srgb(0.14, 0.18, 0.27)
            }),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

#[allow(clippy::type_complexity)]
fn handle_menu_buttons(
    time: Res<Time>,
    mut settings: ResMut<MatchSettings>,
    mut next_state: ResMut<NextState<GameState>>,
    mut buttons: Query<
        (&Interaction, &MenuAction, &mut BackgroundColor, &mut BorderColor),
        With<Button>,
    >,
    mut hdr_q: Query<&mut Text, (With<LoadoutHeaderText>, Without<StartMatchBtnText>)>,
    mut start_q: Query<&mut Text, (With<StartMatchBtnText>, Without<LoadoutHeaderText>)>,
) {
    let mut changed = false;
    for (interaction, action, _, _) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            MenuAction::SetDifficulty(d) => {
                settings.difficulty = d;
                changed = true;
            }
            MenuAction::SetMap(m) => {
                settings.map_id = m;
                changed = true;
            }
            MenuAction::SetMode(mode) => {
                settings.game_mode = mode;
                let max_len = mode.loadout_size();
                if settings.player_loadout.len() > max_len {
                    settings.player_loadout.truncate(max_len);
                }
                changed = true;
            }
            MenuAction::ToggleTower(t) => {
                if let Some(pos) = settings.player_loadout.iter().position(|&x| x == t) {
                    settings.player_loadout.remove(pos);
                    changed = true;
                } else if settings.player_loadout.len() < settings.game_mode.loadout_size() {
                    settings.player_loadout.push(t);
                    changed = true;
                }
            }
            MenuAction::RandomizeLoadout => {
                let nanos = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.subsec_nanos())
                    .unwrap_or(0);
                randomize_player_loadout(&mut settings, time.elapsed_secs().to_bits() ^ nanos);
                changed = true;
            }
            MenuAction::StartMatch => {
                if settings.player_loadout.len() == settings.game_mode.loadout_size() {
                    next_state.set(GameState::Playing);
                    return;
                }
            }
        }
    }

    if !changed {
        return;
    }

    let required = settings.game_mode.loadout_size();
    let ready = settings.player_loadout.len() == required;

    if let Ok(mut t) = hdr_q.get_single_mut() {
        **t = format!(
            "Select Loadout ({}/{})",
            settings.player_loadout.len(),
            required
        );
    }
    if let Ok(mut t) = start_q.get_single_mut() {
        **t = if ready {
            "START 1v1 MATCH".to_string()
        } else {
            format!("PICK {} MORE", required - settings.player_loadout.len())
        };
    }

    for (_, action, mut bg, mut border) in &mut buttons {
        match *action {
            MenuAction::SetDifficulty(d) => {
                let active = settings.difficulty == d;
                bg.0 = if active { Color::srgb(0.20, 0.52, 0.88) } else { Color::srgb(0.14, 0.18, 0.27) };
                border.0 = if active { Color::srgb(0.98, 0.80, 0.24) } else { Color::srgb(0.25, 0.30, 0.42) };
            }
            MenuAction::SetMap(m) => {
                let active = settings.map_id == m;
                bg.0 = if active { Color::srgb(0.20, 0.52, 0.88) } else { Color::srgb(0.14, 0.18, 0.27) };
                border.0 = if active { Color::srgb(0.98, 0.80, 0.24) } else { Color::srgb(0.25, 0.30, 0.42) };
            }
            MenuAction::SetMode(mode) => {
                let active = settings.game_mode == mode;
                bg.0 = if active { Color::srgb(0.20, 0.52, 0.88) } else { Color::srgb(0.14, 0.18, 0.27) };
                border.0 = if active { Color::srgb(0.98, 0.80, 0.24) } else { Color::srgb(0.25, 0.30, 0.42) };
            }
            MenuAction::ToggleTower(t) => {
                let active = settings.player_loadout.contains(&t);
                bg.0 = if active { tower_cfg(t).color } else { Color::srgb(0.12, 0.16, 0.24) };
                border.0 = if active { Color::srgb(0.98, 0.80, 0.24) } else { Color::srgb(0.24, 0.30, 0.40) };
            }
            MenuAction::StartMatch => {
                bg.0 = if ready { Color::srgb(0.95, 0.65, 0.12) } else { Color::srgb(0.24, 0.28, 0.36) };
            }
            MenuAction::RandomizeLoadout => {}
        }
    }
}
