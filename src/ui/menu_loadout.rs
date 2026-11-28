use bevy::prelude::*;

use crate::core::state::{GameMode, MatchSettings};
use crate::core::types::TowerType;
use crate::data::towers_cfg::{tower_cfg, ALL_TOWERS};

#[derive(Component)]
pub enum MenuAction {
    SetDifficulty(crate::core::state::Difficulty),
    SetMap(crate::core::state::MapId),
    SetMode(GameMode),
    ToggleTower(TowerType),
    RandomizeLoadout,
    StartMatch,
}

#[derive(Component)]
pub struct LoadoutHeaderText;

#[derive(Component)]
pub struct StartMatchBtnText;

pub fn spawn_loadout_section(root: &mut ChildBuilder, settings: &MatchSettings) {
    let required = settings.game_mode.loadout_size();
    root.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(14.0),
        margin: UiRect::top(Val::Px(4.0)),
        ..default()
    })
    .with_children(|hdr| {
        hdr.spawn((
            LoadoutHeaderText,
            Text::new(format!(
                "Select Loadout ({}/{})",
                settings.player_loadout.len(),
                required
            )),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.85, 0.90, 0.96)),
        ));
        hdr.spawn((
            Button,
            MenuAction::RandomizeLoadout,
            Node {
                min_width: Val::Px(104.0),
                min_height: Val::Px(48.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                ..default()
            },
            BorderColor(Color::srgb(0.38, 0.72, 0.98)),
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(Color::srgb(0.14, 0.22, 0.34)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("Random"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
    });

    for chunk in ALL_TOWERS.chunks(6) {
        root.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            for &t in chunk {
                let cfg = tower_cfg(t);
                let active = settings.player_loadout.contains(&t);
                row.spawn((
                    Button,
                    MenuAction::ToggleTower(t),
                    Node {
                        min_width: Val::Px(116.0),
                        min_height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor(if active {
                        Color::srgb(0.98, 0.80, 0.24)
                    } else {
                        Color::srgb(0.24, 0.30, 0.40)
                    }),
                    BorderRadius::all(Val::Px(8.0)),
                    BackgroundColor(if active {
                        cfg.color
                    } else {
                        Color::srgb(0.12, 0.16, 0.24)
                    }),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(format!("{}\n${}", cfg.short_name, cfg.cost)),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });
    }
}

pub fn randomize_player_loadout(settings: &mut MatchSettings, seed_bits: u32) {
    let target = settings.game_mode.loadout_size();
    let mut rng = seed_bits | 1;
    let mut pool: Vec<TowerType> = ALL_TOWERS.to_vec();
    let mut next_deck = Vec::with_capacity(target);
    while next_deck.len() < target && !pool.is_empty() {
        rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
        let idx = (rng as usize) % pool.len();
        next_deck.push(pool.swap_remove(idx));
    }
    settings.player_loadout = next_deck;
}
