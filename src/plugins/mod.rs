pub mod ai;
pub mod bloons;
pub mod economy;
pub mod map;
pub mod towers;

use bevy::prelude::*;

use ai::AiPlugin;
use bloons::BloonsPlugin;
use economy::EconomyPlugin;
use map::MapPlugin;
use towers::TowersPlugin;

pub struct GamePlugins;

impl Plugin for GamePlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MapPlugin,
            BloonsPlugin,
            TowersPlugin,
            EconomyPlugin,
            AiPlugin,
        ));
    }
}
