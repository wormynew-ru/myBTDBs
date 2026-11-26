use bevy::prelude::*;

use crate::core::events::{BloonLeakedEvent, BloonPoppedEvent, SendBloonPackEvent};
use crate::core::state::{GameState, MatchResult, MatchSettings};
use crate::core::types::{Side, SideEconomy, SidesResource};
use crate::data::bloons_cfg::{SEND_PACKS, WAVE_DURATION_SECS};
use crate::data::towers_cfg::{ECO_INTERVAL_SECS, STARTING_CASH, STARTING_ECO, STARTING_LIVES};
use crate::plugins::bloons::{QueuedSpawn, SpawnQueue};

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
    *sides = SidesResource {
        player: SideEconomy {
            lives: STARTING_LIVES,
            cash: STARTING_CASH,
            eco: STARTING_ECO,
            pops: 0,
        },
        bot: SideEconomy {
            lives: STARTING_LIVES,
            cash: STARTING_CASH,
            eco: STARTING_ECO,
            pops: 0,
        },
        eco_timer: ECO_INTERVAL_SECS,
        wave: 1,
        wave_timer: WAVE_DURATION_SECS,
        match_time: 0.0,
    };
}

fn eco_payout_system(
    time: Res<Time>,
    settings: Res<MatchSettings>,
    mut sides: ResMut<SidesResource>,
) {
    if settings.paused {
        return;
    }
    let dt = time.delta_secs() * settings.speed_mult;
    sides.eco_timer -= dt;
    if sides.eco_timer <= 0.0 {
        sides.eco_timer += ECO_INTERVAL_SECS;
        sides.player.cash += sides.player.eco;
        sides.bot.cash += sides.bot.eco;
    }
}

fn handle_send_bloon_packs(
    mut sides: ResMut<SidesResource>,
    mut queue: ResMut<SpawnQueue>,
    mut reader: EventReader<SendBloonPackEvent>,
) {
    for ev in reader.read() {
        let Some(pack) = SEND_PACKS.get(ev.pack_index) else {
            continue;
        };
        if sides.wave < pack.unlock_wave || sides.get(ev.sender).cash < pack.cost {
            continue;
        }
        {
            let econ = sides.get_mut(ev.sender);
            econ.cash -= pack.cost;
            econ.eco += pack.eco_gain;
        }

        let target = ev.sender.opponent();
        let mut base_delay = 0.05_f32;
        for item in &queue.items {
            if item.side == target && item.from_send && item.delay > base_delay {
                base_delay = (item.delay + 0.12).min(1.5);
            }
        }

        for i in 0..pack.count {
            queue.items.push(QueuedSpawn {
                side: target,
                layer: pack.layer,
                camo: pack.camo,
                lead: pack.lead,
                delay: base_delay + (i as f32) * pack.spacing,
                from_send: true,
            });
        }
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
