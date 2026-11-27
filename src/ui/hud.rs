use bevy::prelude::*;

use crate::core::state::{GameState, MatchSettings};
use crate::core::types::{Side, SidesResource, TowerType};
use crate::data::tower_stats::compute_effective_stats;
use crate::plugins::towers::Tower;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_hud)
            .add_systems(OnExit(GameState::Playing), despawn_hud)
            .add_systems(
                Update,
                (update_hud_texts, handle_hud_buttons).run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct PlayerHudText;

#[derive(Component)]
struct WaveHudText;

#[derive(Component)]
struct BotHudText;

#[derive(Component)]
enum HudAction {
    ToggleSpeed,
    TogglePause,
    BackToMenu,
}

fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(58.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.08, 0.14, 0.92)),
        ))
        .with_children(|bar| {
            bar.spawn((
                PlayerHudText,
                Text::new("YOU | HP: 150 | $650 | Eco: 250"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.45, 0.85, 0.98)),
            ));

            bar.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|center| {
                center.spawn((
                    WaveHudText,
                    Text::new("WAVE 1 | Eco 6.0s"),
                    TextFont {
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                spawn_hud_btn(center, "1x/2x", HudAction::ToggleSpeed);
                spawn_hud_btn(center, "Pause", HudAction::TogglePause);
                spawn_hud_btn(center, "Menu", HudAction::BackToMenu);
            });

            bar.spawn((
                BotHudText,
                Text::new("BOT | HP: 150 | $650 | Eco: 250"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.98, 0.45, 0.45)),
            ));
        });
}

fn spawn_hud_btn(parent: &mut ChildBuilder, label: &str, action: HudAction) {
    parent
        .spawn((
            Button,
            action,
            Node {
                min_width: Val::Px(58.0),
                min_height: Val::Px(48.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                ..default()
            },
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(Color::srgb(0.16, 0.21, 0.31)),
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

fn despawn_hud(mut commands: Commands, query: Query<Entity, With<HudRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn update_hud_texts(
    sides: Res<SidesResource>,
    settings: Res<MatchSettings>,
    send_q: Res<crate::plugins::bloons::SendPackQueue>,
    towers: Query<(&Side, &Tower)>,
    mut player_q: Query<&mut Text, (With<PlayerHudText>, Without<WaveHudText>, Without<BotHudText>)>,
    mut wave_q: Query<&mut Text, (With<WaveHudText>, Without<PlayerHudText>, Without<BotHudText>)>,
    mut bot_q: Query<&mut Text, (With<BotHudText>, Without<PlayerHudText>, Without<WaveHudText>)>,
) {
    let mut p_farm_eco = 0;
    let mut b_farm_eco = 0;
    for (side, tower) in &towers {
        if tower.tower_type == TowerType::Farm && tower.path2 > 0 {
            let bonus = compute_effective_stats(tower.tower_type, tower.path1, tower.path2)
                .farm_eco_bonus;
            match side {
                Side::Player => p_farm_eco += bonus,
                Side::Bot => b_farm_eco += bonus,
            }
        }
    }

    if let Ok(mut text) = player_q.get_single_mut() {
        let q_bot = send_q.count_for(Side::Bot);
        let q_tag = if q_bot > 0 { format!(" | Q:{}", q_bot) } else { String::new() };
        **text = format!(
            "YOU | HP: {} | ${} | Eco: {}{}",
            sides.player.lives,
            sides.player.cash,
            sides.player.eco + p_farm_eco,
            q_tag
        );
    }
    if let Ok(mut text) = wave_q.get_single_mut() {
        let pause_tag = if settings.paused { " [PAUSED]" } else { "" };
        **text = format!(
            "W{} | {}x | Eco {:.1}s{}",
            sides.wave, settings.speed_mult as u32, sides.eco_timer, pause_tag
        );
    }
    if let Ok(mut text) = bot_q.get_single_mut() {
        let q_pl = send_q.count_for(Side::Player);
        let q_tag = if q_pl > 0 { format!(" | Q:{}", q_pl) } else { String::new() };
        **text = format!(
            "BOT | HP: {} | ${} | Eco: {}{}",
            sides.bot.lives,
            sides.bot.cash,
            sides.bot.eco + b_farm_eco,
            q_tag
        );
    }
}

fn handle_hud_buttons(
    mut settings: ResMut<MatchSettings>,
    mut next_state: ResMut<NextState<GameState>>,
    interactions: Query<(&Interaction, &HudAction), Changed<Interaction>>,
) {
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            HudAction::ToggleSpeed => {
                settings.speed_mult = if settings.speed_mult > 1.5 { 1.0 } else { 2.0 };
            }
            HudAction::TogglePause => {
                settings.paused = !settings.paused;
            }
            HudAction::BackToMenu => {
                next_state.set(GameState::MainMenu);
            }
        }
    }
}
