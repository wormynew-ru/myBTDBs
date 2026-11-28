use bevy::prelude::*;

use crate::core::events::{BloonLeakedEvent, BloonPoppedEvent, SendBloonPackEvent};
use crate::core::state::{GameState, MatchResult, MatchSettings};
use crate::core::types::{Side, SideEconomy, SidesResource, TowerType};
use crate::data::bloons_cfg::SEND_PACKS;
use crate::data::tower_stats::{compute_effective_stats, discounted_send_cost};
use crate::data::towers_cfg::STARTING_LIVES;
use crate::plugins::bloons::{QueuedSendPack, SendPackQueue};
use crate::plugins::towers::Tower;

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), reset_economy)
            .add_systems(
                Update,
                (
                    eco_payout_system,
                    handle_send_bloon_packs,
                    handle_bloon_popped,
                    handle_bloon_leaked,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn reset_economy(mut sides: ResMut<SidesResource>, mut settings: ResMut<MatchSettings>) {
    settings.paused = false;
    settings.speed_mult = 1.0;
    let mode = settings.game_mode;
    *sides = SidesResource {
        player: SideEconomy {
            lives: STARTING_LIVES,
            cash: mode.starting_cash(),
            eco: mode.starting_eco(),
            pops: 0,
        },
        bot: SideEconomy {
            lives: STARTING_LIVES,
            cash: mode.starting_cash(),
            eco: mode.starting_eco(),
            pops: 0,
        },
        eco_timer: mode.eco_interval(),
        wave: 1,
        wave_timer: mode.wave_duration(),
        match_time: 0.0,
    };
}

fn eco_payout_system(
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut sides: ResMut<SidesResource>,
    towers: Query<(&Side, &Tower)>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;
    sides.eco_timer -= dt;
    if sides.eco_timer <= 0.0 {
        sides.eco_timer += settings.game_mode.eco_interval();
        let mut p_bonus = 0;
        let mut b_bonus = 0;
        for (side, tower) in &towers {
            if tower.tower_type == TowerType::Farm && tower.path2 > 0 {
                let bonus = compute_effective_stats(tower.tower_type, tower.path1, tower.path2)
                    .farm_eco_bonus;
                match side {
                    Side::Player => p_bonus += bonus,
                    Side::Bot => b_bonus += bonus,
                }
            }
        }
        sides.player.cash += sides.player.eco + p_bonus;
        sides.bot.cash += sides.bot.eco + b_bonus;
    }
}

fn handle_send_bloon_packs(
    mut sides: ResMut<SidesResource>,
    mut send_q: ResMut<SendPackQueue>,
    towers: Query<(&Side, &Tower)>,
    mut reader: EventReader<SendBloonPackEvent>,
) {
    const MAX_QUEUED_PACKS: usize = 12;

    for ev in reader.read() {
        let Some(pack) = SEND_PACKS.get(ev.pack_index) else {
            continue;
        };
        let discount: f32 = towers
            .iter()
            .filter(|(s, t)| **s == ev.sender && t.tower_type == TowerType::Farm && t.path2 >= 2)
            .map(|(_, t)| compute_effective_stats(t.tower_type, t.path1, t.path2).farm_send_discount)
            .sum::<f32>()
            .min(0.20);
        let cost = discounted_send_cost(pack.cost, discount);

        let target = ev.sender.opponent();
        if sides.wave < pack.unlock_wave
            || sides.get(ev.sender).cash < cost
            || send_q.count_for(target) >= MAX_QUEUED_PACKS
        {
            continue;
        }
        {
            let econ = sides.get_mut(ev.sender);
            econ.cash -= cost;
            econ.eco = (econ.eco + pack.eco_gain).max(0);
        }

        let (queue, _) = send_q.queue_for_mut(target);
        queue.push_back(QueuedSendPack {
            layer: pack.layer,
            camo: pack.camo,
            lead: pack.lead,
            remaining: pack.count,
            spacing: pack.spacing,
            timer: 0.05,
        });
    }
}

fn handle_bloon_popped(
    mut sides: ResMut<SidesResource>,
    mut reader: EventReader<BloonPoppedEvent>,
) {
    for ev in reader.read() {
        let econ = sides.get_mut(ev.side);
        econ.cash += ev.cash_gain;
        econ.pops += ev.pops;
    }
}

fn handle_bloon_leaked(
    mut sides: ResMut<SidesResource>,
    mut result: ResMut<MatchResult>,
    mut next_state: ResMut<NextState<GameState>>,
    mut reader: EventReader<BloonLeakedEvent>,
) {
    for ev in reader.read() {
        let econ = sides.get_mut(ev.side);
        econ.lives = (econ.lives - ev.life_loss).max(0);
        if econ.lives == 0 {
            result.winner = Some(ev.side.opponent());
            result.wave_reached = sides.wave;
            result.match_time = sides.match_time;
            next_state.set(GameState::GameOver);
            break;
        }
    }
}
