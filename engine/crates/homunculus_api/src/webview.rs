//! Webview API: opening, listing, navigating, and closing webviews.

mod close;
pub(crate) mod constraint;
mod get;
mod is_closed;
mod linked_persona;
mod list;
mod navigate;
pub(super) mod open;
mod reload;
mod update;

pub use open::WebviewSpawner;

use crate::api;
use bevy::prelude::*;

api!(WebviewApi);

pub struct WebviewApiPlugin;

impl Plugin for WebviewApiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((open::WebviewOpenPlugin, constraint::ConstraintPlugin));
    }
}
