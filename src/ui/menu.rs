use bevy::prelude::*;

use crate::core::state::{Difficulty, GameState, MapId, MatchSettings};

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

#[derive(Component)]
enum MenuAction {
    SetDifficulty(Difficulty),
    SetMap(MapId),
    StartMatch,
}

fn spawn_main_menu(mut commands: Commands, settings: Res<MatchSettings>) {
    commands
        .spawn((
            MenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(18.0),
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.09, 0.15)),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("myBTDBs — Bloons TD Battles PvE"),
                TextFont {
                    font_size: 34.0,
                    ..default()
                },
                TextColor(Color::srgb(0.98, 0.80, 0.24)),
            ));

            root.spawn((
                Text::new("Select Bot Difficulty"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.80, 0.88)),
            ));

            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|row| {
                for diff in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
                    let active = settings.difficulty == diff;
                    spawn_menu_button(
                        row,
                        diff.label(),
                        MenuAction::SetDifficulty(diff),
                        active,
                        140.0,
                    );
                }
            });

            root.spawn((
                Text::new("Select Arena Map"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.80, 0.88)),
            ));

            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|row| {
                for map_id in [MapId::Meadow, MapId::Zigzag, MapId::Canyon] {
                    let active = settings.map_id == map_id;
                    spawn_menu_button(
                        row,
                        map_id.label(),
                        MenuAction::SetMap(map_id),
                        active,
                        180.0,
                    );
                }
            });

            root.spawn((
                Button,
                MenuAction::StartMatch,
                Node {
                    min_width: Val::Px(220.0),
                    min_height: Val::Px(56.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(12.0)),
                    padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                    ..default()
                },
                BorderRadius::all(Val::Px(10.0)),
                BackgroundColor(Color::srgb(0.95, 0.65, 0.12)),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("START 1v1 MATCH"),
                    TextFont {
                        font_size: 20.0,
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
    let bg = if active {
        Color::srgb(0.20, 0.52, 0.88)
    } else {
        Color::srgb(0.14, 0.18, 0.27)
    };
    parent
        .spawn((
            Button,
            action,
            Node {
                min_width: Val::Px(width),
                min_height: Val::Px(52.0),
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
            BackgroundColor(bg),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
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

fn handle_menu_buttons(
    mut settings: ResMut<MatchSettings>,
    mut next_state: ResMut<NextState<GameState>>,
    mut buttons: Query<(&Interaction, &MenuAction, &mut BackgroundColor, &mut BorderColor), With<Button>>,
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
            MenuAction::StartMatch => {
                next_state.set(GameState::Playing);
                return;
            }
        }
    }

    if changed {
        for (_, action, mut bg, mut border) in &mut buttons {
            let active = match action {
                MenuAction::SetDifficulty(d) => settings.difficulty == *d,
                MenuAction::SetMap(m) => settings.map_id == *m,
                MenuAction::StartMatch => continue,
            };
            bg.0 = if active {
                Color::srgb(0.20, 0.52, 0.88)
            } else {
                Color::srgb(0.14, 0.18, 0.27)
            };
            border.0 = if active {
                Color::srgb(0.98, 0.80, 0.24)
            } else {
                Color::srgb(0.25, 0.30, 0.42)
            };
        }
    }
}
