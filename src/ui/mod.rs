pub mod build_bar;
pub mod game_over;
pub mod hud;
pub mod menu;
pub mod upgrade_panel;

use bevy::prelude::*;

use build_bar::BuildBarPlugin;
use game_over::GameOverPlugin;
use hud::HudPlugin;
use menu::MenuPlugin;
use upgrade_panel::UpgradePanelPlugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MenuPlugin,
            HudPlugin,
            BuildBarPlugin,
            UpgradePanelPlugin,
            GameOverPlugin,
        ));
    }
}
