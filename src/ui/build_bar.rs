use bevy::prelude::*;

use crate::core::events::SendBloonPackEvent;
use crate::core::state::{BuildSelection, GameState, SelectedTower};
use crate::core::types::{Side, SidesResource, TowerType};
use crate::data::bloons_cfg::{SendPackCfg, SEND_PACKS};
use crate::data::tower_stats::{compute_effective_stats, discounted_send_cost};
use crate::data::towers_cfg::{tower_cfg, ALL_TOWERS};
use crate::plugins::towers::Tower;

const TOWERS_PER_PAGE: usize = 6;
const PACKS_PER_PAGE: usize = 8;

pub struct BuildBarPlugin;

impl Plugin for BuildBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TowerPage>()
            .init_resource::<SendPackPage>()
            .add_systems(OnEnter(GameState::Playing), spawn_build_bar)
            .add_systems(OnExit(GameState::Playing), despawn_build_bar)
            .add_systems(
                Update,
                (handle_build_bar_buttons, update_build_bar_styles)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Resource, Default)]
struct TowerPage(usize);

#[derive(Resource, Default)]
struct SendPackPage(usize);

#[derive(Component)]
struct BuildBarRoot;

#[derive(Component)]
enum BuildBarAction {
    SelectTowerSlot(usize),
    ToggleTowerPage,
    SendPackSlot(usize),
    TogglePackPage,
}

#[derive(Component)]
struct TowerSlotLabel(usize);

#[derive(Component)]
struct TowerPageToggleLabel;

#[derive(Component)]
struct SendPackSlotLabel(usize);

#[derive(Component)]
struct PageToggleLabel;

fn spawn_build_bar(
    mut commands: Commands,
    mut tower_page: ResMut<TowerPage>,
    mut pack_page: ResMut<SendPackPage>,
) {
    tower_page.0 = 0;
    pack_page.0 = 0;
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
                padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)),
                column_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.08, 0.14, 0.94)),
        ))
        .with_children(|bar| {
            bar.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|row| {
                for slot in 0..TOWERS_PER_PAGE {
                    let cfg = tower_cfg(ALL_TOWERS[slot]);
                    let label = format!("{}\n${}", cfg.short_name, cfg.cost);
                    spawn_slot_btn(
                        row, BuildBarAction::SelectTowerSlot(slot), TowerSlotLabel(slot),
                        &label, 58.0, 11.0, Color::srgb(0.30, 0.36, 0.48), cfg.color, Color::WHITE,
                    );
                }
                spawn_slot_btn(
                    row, BuildBarAction::ToggleTowerPage, TowerPageToggleLabel,
                    "1/2", 48.0, 13.0, Color::srgb(0.38, 0.72, 0.98),
                    Color::srgb(0.18, 0.24, 0.36), Color::srgb(0.38, 0.72, 0.98),
                );
            });

            bar.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(3.0),
                ..default()
            })
            .with_children(|row| {
                for slot in 0..PACKS_PER_PAGE {
                    let pack = &SEND_PACKS[slot];
                    let label = format_pack_label(pack, 0.0);
                    spawn_slot_btn(
                        row, BuildBarAction::SendPackSlot(slot), SendPackSlotLabel(slot),
                        &label, 60.0, 10.0, pack.badge_color,
                        Color::srgb(0.11, 0.15, 0.24), Color::WHITE,
                    );
                }
                spawn_slot_btn(
                    row, BuildBarAction::TogglePackPage, PageToggleLabel,
                    "1/2", 48.0, 13.0, Color::srgb(0.98, 0.80, 0.24),
                    Color::srgb(0.18, 0.24, 0.36), Color::srgb(0.98, 0.80, 0.24),
                );
            });
        });
}

#[allow(clippy::too_many_arguments)]
fn spawn_slot_btn<M: Component>(
    row: &mut ChildBuilder,
    action: BuildBarAction,
    marker: M,
    label: &str,
    min_w: f32,
    font_size: f32,
    border: Color,
    bg: Color,
    text_color: Color,
) {
    row.spawn((
        Button,
        action,
        Node {
            min_width: Val::Px(min_w),
            min_height: Val::Px(56.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BorderColor(border),
        BorderRadius::all(Val::Px(7.0)),
        BackgroundColor(bg),
    ))
    .with_children(|btn| {
        btn.spawn((
            marker,
            Text::new(label),
            TextFont { font_size, ..default() },
            TextColor(text_color),
        ));
    });
}

fn format_pack_label(pack: &SendPackCfg, discount: f32) -> String {
    let cost = discounted_send_cost(pack.cost, discount);
    let eco_str = if pack.eco_gain >= 0 {
        format!("+{}", pack.eco_gain)
    } else {
        format!("{}", pack.eco_gain)
    };
    format!("{}\n${} {}", pack.short_label, cost, eco_str)
}

fn despawn_build_bar(mut commands: Commands, query: Query<Entity, With<BuildBarRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn handle_build_bar_buttons(
    mut tower_page: ResMut<TowerPage>,
    mut pack_page: ResMut<SendPackPage>,
    mut build_sel: ResMut<BuildSelection>,
    mut selected_tower: ResMut<SelectedTower>,
    mut send_writer: EventWriter<SendBloonPackEvent>,
    interactions: Query<(&Interaction, &BuildBarAction), Changed<Interaction>>,
) {
    let total_tower_pages = ALL_TOWERS.len().div_ceil(TOWERS_PER_PAGE).max(1);
    let total_pack_pages = SEND_PACKS.len().div_ceil(PACKS_PER_PAGE).max(1);

    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            BuildBarAction::SelectTowerSlot(slot) => {
                let idx = tower_page.0 * TOWERS_PER_PAGE + slot;
                if let Some(&t) = ALL_TOWERS.get(idx) {
                    if build_sel.0 == Some(t) {
                        build_sel.0 = None;
                    } else {
                        build_sel.0 = Some(t);
                        selected_tower.0 = None;
                    }
                }
            }
            BuildBarAction::ToggleTowerPage => {
                tower_page.0 = (tower_page.0 + 1) % total_tower_pages;
            }
            BuildBarAction::SendPackSlot(slot) => {
                let pack_index = pack_page.0 * PACKS_PER_PAGE + slot;
                if pack_index < SEND_PACKS.len() {
                    send_writer.send(SendBloonPackEvent { sender: Side::Player, pack_index });
                }
            }
            BuildBarAction::TogglePackPage => {
                pack_page.0 = (pack_page.0 + 1) % total_pack_pages;
            }
        }
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
fn update_build_bar_styles(
    tower_page: Res<TowerPage>,
    pack_page: Res<SendPackPage>,
    build_sel: Res<BuildSelection>,
    sides: Res<SidesResource>,
    towers: Query<(&Side, &Tower)>,
    mut buttons: Query<(&BuildBarAction, &mut BorderColor, &mut BackgroundColor, &mut Visibility)>,
    mut tower_texts: Query<
        (&TowerSlotLabel, &mut Text),
        (Without<TowerPageToggleLabel>, Without<SendPackSlotLabel>, Without<PageToggleLabel>),
    >,
    mut tower_page_texts: Query<
        &mut Text,
        (With<TowerPageToggleLabel>, Without<TowerSlotLabel>, Without<SendPackSlotLabel>, Without<PageToggleLabel>),
    >,
    mut slot_texts: Query<
        (&SendPackSlotLabel, &mut Text),
        (Without<PageToggleLabel>, Without<TowerSlotLabel>, Without<TowerPageToggleLabel>),
    >,
    mut page_texts: Query<
        &mut Text,
        (With<PageToggleLabel>, Without<SendPackSlotLabel>, Without<TowerSlotLabel>, Without<TowerPageToggleLabel>),
    >,
) {
    let total_tower_pages = ALL_TOWERS.len().div_ceil(TOWERS_PER_PAGE).max(1);
    let total_pack_pages = SEND_PACKS.len().div_ceil(PACKS_PER_PAGE).max(1);
    let discount: f32 = towers
        .iter()
        .filter(|(s, t)| **s == Side::Player && t.tower_type == TowerType::Farm && t.path2 >= 2)
        .map(|(_, t)| compute_effective_stats(t.tower_type, t.path1, t.path2).farm_send_discount)
        .sum::<f32>()
        .min(0.20);

    for (action, mut border, mut bg, mut vis) in &mut buttons {
        match *action {
            BuildBarAction::SelectTowerSlot(slot) => {
                let idx = tower_page.0 * TOWERS_PER_PAGE + slot;
                if let Some(&t) = ALL_TOWERS.get(idx) {
                    *vis = Visibility::Inherited;
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
                    bg.0 = if affordable || active { cfg.color } else { Color::srgb(0.14, 0.16, 0.20) };
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            BuildBarAction::SendPackSlot(slot) => {
                let idx = pack_page.0 * PACKS_PER_PAGE + slot;
                if let Some(pack) = SEND_PACKS.get(idx) {
                    let unlocked = sides.wave >= pack.unlock_wave;
                    let affordable = sides.player.cash >= discounted_send_cost(pack.cost, discount);
                    border.0 = pack.badge_color;
                    bg.0 = if unlocked && affordable {
                        Color::srgb(0.12, 0.18, 0.28)
                    } else {
                        Color::srgb(0.07, 0.09, 0.14)
                    };
                }
            }
            BuildBarAction::ToggleTowerPage | BuildBarAction::TogglePackPage => {}
        }
    }

    for (slot, mut text) in &mut tower_texts {
        let idx = tower_page.0 * TOWERS_PER_PAGE + slot.0;
        if let Some(&t) = ALL_TOWERS.get(idx) {
            let cfg = tower_cfg(t);
            **text = format!("{}\n${}", cfg.short_name, cfg.cost);
        }
    }
    if let Ok(mut text) = tower_page_texts.get_single_mut() {
        **text = format!("{}/{}", tower_page.0 + 1, total_tower_pages);
    }
    for (slot, mut text) in &mut slot_texts {
        let idx = pack_page.0 * PACKS_PER_PAGE + slot.0;
        if let Some(pack) = SEND_PACKS.get(idx) {
            **text = format_pack_label(pack, discount);
        }
    }
    if let Ok(mut text) = page_texts.get_single_mut() {
        **text = format!("{}/{}", pack_page.0 + 1, total_pack_pages);
    }
}
