use bevy::prelude::*;

use crate::core::state::{GameState, MatchResult};
use crate::core::types::{Side, SidesResource};

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), spawn_game_over)
            .add_systems(OnExit(GameState::GameOver), despawn_game_over)
            .add_systems(
                Update,
                handle_game_over_buttons.run_if(in_state(GameState::GameOver)),
            );
    }
}

#[derive(Component)]
struct GameOverRoot;

#[derive(Component)]
enum GameOverAction {
    Rematch,
    MainMenu,
}

fn spawn_game_over(
    mut commands: Commands,
    result: Res<MatchResult>,
    sides: Res<SidesResource>,
) {
    let won = result.winner == Some(Side::Player);
    let title = if won { "VICTORY!" } else { "DEFEAT" };
    let title_color = if won {
        Color::srgb(0.25, 0.90, 0.48)
    } else {
        Color::srgb(0.95, 0.30, 0.35)
    };

    commands
        .spawn((
            GameOverRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.05, 0.10, 0.85)),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        width: Val::Px(420.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(14.0),
                        padding: UiRect::all(Val::Px(28.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor(Color::srgb(0.32, 0.40, 0.54)),
                    BorderRadius::all(Val::Px(14.0)),
                    BackgroundColor(Color::srgb(0.08, 0.11, 0.18)),
                ))
                .with_children(|card| {
                    card.spawn((
                        Text::new(title),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(title_color),
                    ));

                    card.spawn((
                        Text::new(format!(
                            "Wave: {} | Time: {:.0}s\nYour Pops: {} (Eco {})\nBot Pops: {} (Eco {})",
                            result.wave_reached,
                            result.match_time,
                            sides.player.pops,
                            sides.player.eco,
                            sides.bot.pops,
                            sides.bot.eco
                        )),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.84, 0.88, 0.94)),
                    ));

                    card.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(12.0),
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_end_btn(
                            row,
                            "Main Menu",
                            GameOverAction::MainMenu,
                            Color::srgb(0.20, 0.25, 0.35),
                        );
                        spawn_end_btn(
                            row,
                            "Rematch",
                            GameOverAction::Rematch,
                            Color::srgb(0.95, 0.65, 0.12),
                        );
                    });
                });
        });
}

fn spawn_end_btn(
    parent: &mut ChildBuilder,
    label: &str,
    action: GameOverAction,
    bg: Color,
) {
    parent
        .spawn((
            Button,
            action,
            Node {
                min_width: Val::Px(140.0),
                min_height: Val::Px(52.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
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

fn despawn_game_over(mut commands: Commands, query: Query<Entity, With<GameOverRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn handle_game_over_buttons(
    mut next_state: ResMut<NextState<GameState>>,
    interactions: Query<(&Interaction, &GameOverAction), Changed<Interaction>>,
) {
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            GameOverAction::Rematch => next_state.set(GameState::Playing),
            GameOverAction::MainMenu => next_state.set(GameState::MainMenu),
        }
    }
}
