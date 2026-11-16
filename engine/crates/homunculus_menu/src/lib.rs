//! The right-click menu built into the engine: a secondary click on a
//! character opens or closes the menu HUD and plays its sound.

mod toggle;

use bevy::prelude::*;

const MENU_HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/menu.html"));

/// Marks the webview that shows the right-click menu.
#[derive(Component, Debug)]
pub(crate) struct MenuHud;

/// Opens the right-click menu when a character is clicked with the secondary
/// button, and closes it on the next secondary click.
///
/// The menu webview is linked to the clicked persona and opened from HTML
/// embedded in the binary. At most one secondary click is acted on per frame;
/// when it hits several characters, the nearest one gets the menu. A menu
/// left without its character is closed.
pub struct HomunculusMenuPlugin;

impl Plugin for HomunculusMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PointerClick>()
            .add_systems(Update, (toggle::toggle_menu, close_orphaned_menus));
    }
}

fn close_orphaned_menus(
    menus: Query<Entity, (With<MenuHud>, Without<ChildOf>)>,
    mut commands: Commands,
) {
    for menu in &menus {
        commands.entity(menu).try_despawn();
    }
}
