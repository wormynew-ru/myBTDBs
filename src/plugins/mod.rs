pub mod ai;
pub mod bloons;
pub mod economy;
pub mod map;
pub mod sprites;
pub mod towers;

use bevy::prelude::*;

use ai::AiPlugin;
use bloons::BloonsPlugin;
use economy::EconomyPlugin;
use map::MapPlugin;
use sprites::SpritesPlugin;
use towers::TowersPlugin;

pub struct GamePlugins;

impl Plugin for GamePlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SpritesPlugin,
            MapPlugin,
            BloonsPlugin,
            TowersPlugin,
            EconomyPlugin,
            AiPlugin,
        ));
    }
}
