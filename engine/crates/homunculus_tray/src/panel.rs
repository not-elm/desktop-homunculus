//! Built-in panels opened from their own tray items, with at most one panel
//! of each kind open at a time.

mod settings;
mod stt;

use crate::BuiltinTrayAppExt;
use bevy::prelude::*;
use homunculus_api::prelude::WebviewSpawner;
use homunculus_audio::prelude::SePlayer;
use homunculus_core::prelude::{
    TransformArgs, WebviewOpenOptions, WebviewResizableOptions, WebviewSource,
};
use settings::SettingsPanel;
use stt::SttPanel;

const PANEL_SIZE: Vec2 = Vec2::new(0.8, 1.0);
const PANEL_TRANSLATION: Vec3 = Vec3::new(0.0, 0.0, 10.0);
const OPEN_SOUND: &str = "se:open";

/// A panel opened from its own built-in tray item; the implementing
/// component marks the panel's webview.
trait TrayPanel: Component + Default {
    /// The key of the panel's built-in tray item.
    const KEY: &'static str;
    /// The label of the panel's tray item.
    const TEXT: &'static str;
    /// The position of the tray item among the built-in items; lower comes
    /// first.
    const ORDER: u32;
    /// The HTML the panel shows.
    const HTML: &'static str;
    /// The panel's viewport size in logical pixels.
    const VIEWPORT_SIZE: Vec2;
}

/// Adds a built-in tray item for each panel. Picking one opens its panel,
/// resizable and linked to no character, and plays `se:open`, unless that
/// panel is already open.
pub(crate) struct BuiltinPanelsPlugin;

impl Plugin for BuiltinPanelsPlugin {
    fn build(&self, app: &mut App) {
        add_panel::<SttPanel>(app);
        add_panel::<SettingsPanel>(app);
    }
}

fn add_panel<P: TrayPanel>(app: &mut App) {
    app.add_builtin_tray_item(P::KEY, P::TEXT, P::ORDER, open_panel::<P>);
}

fn open_panel<P: TrayPanel>(
    panels: Query<(), With<P>>,
    mut spawner: WebviewSpawner,
    mut se: SePlayer,
) {
    if !panels.is_empty() {
        return;
    }
    match spawner.open(panel_options::<P>()) {
        Ok(mut panel) => {
            panel.insert(P::default());
            se.play(OPEN_SOUND);
        }
        Err(e) => error!("Failed to open the {} panel: {e}", P::TEXT),
    }
}

fn panel_options<P: TrayPanel>() -> WebviewOpenOptions {
    WebviewOpenOptions {
        source: WebviewSource::Html {
            content: P::HTML.to_string(),
        },
        size: Some(PANEL_SIZE),
        viewport_size: Some(P::VIEWPORT_SIZE),
        transform: Some(TransformArgs {
            translation: Some(PANEL_TRANSLATION),
            rotation: None,
            scale: None,
        }),
        constraints: None,
        linked_persona: None,
        resizable: Some(WebviewResizableOptions::default()),
    }
}

#[cfg(test)]
mod test_support {
    use super::*;
    use crate::BuiltinTrayItems;
    use bevy::ecs::system::{RunSystemOnce, SystemId};
    use bevy_cef::prelude::WebviewExtendStandardMaterial;
    use homunculus_audio::prelude::{KiraAudioSource, RequestSe};
    use homunculus_core::prelude::{
        AssetEntry, AssetId, AssetOrigin, AssetRegistry, AssetType, PersonaIndex,
    };
    use std::path::PathBuf;

    #[derive(Resource, Default)]
    struct SeLog(Vec<String>);

    pub(super) fn panel_app() -> App {
        let mut app = panel_app_without_sounds();
        app.world_mut()
            .resource_mut::<AssetRegistry>()
            .register_builtin(sound_entry("se:open", "se/open.mp3"));
        app
    }

    pub(super) fn panel_app_without_sounds() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_asset::<WebviewExtendStandardMaterial>()
            .init_asset::<KiraAudioSource>()
            .init_resource::<AssetRegistry>()
            .init_resource::<PersonaIndex>()
            .init_resource::<SeLog>()
            .add_plugins(BuiltinPanelsPlugin)
            .add_observer(|trigger: On<RequestSe>, mut log: ResMut<SeLog>| {
                let path = trigger.source.path().map(ToString::to_string);
                log.0.push(path.unwrap_or_default());
            });
        app
    }

    pub(super) fn click(app: &mut App, key: &str) {
        let on_click = on_click(app, key);
        app.world_mut()
            .run_system(on_click)
            .expect("the tray item's system runs");
    }

    pub(super) fn click_twice_in_one_command_queue(app: &mut App, key: &str) {
        let on_click = on_click(app, key);
        app.world_mut()
            .run_system_once(move |mut commands: Commands| {
                commands.run_system(on_click);
                commands.run_system(on_click);
            })
            .expect("both clicks are queued");
    }

    pub(super) fn opened<P: Component>(app: &mut App) -> Vec<Entity> {
        app.world_mut()
            .query_filtered::<Entity, With<P>>()
            .iter(app.world())
            .collect()
    }

    pub(super) fn se_log(app: &App) -> Vec<String> {
        app.world().resource::<SeLog>().0.clone()
    }

    fn on_click(app: &App, key: &str) -> SystemId {
        app.world()
            .resource::<BuiltinTrayItems>()
            .get(key)
            .expect("the tray item is registered")
            .on_click
    }

    fn sound_entry(id: &str, path: &str) -> AssetEntry {
        AssetEntry {
            id: AssetId::new(id),
            path: PathBuf::from(path),
            absolute_path: PathBuf::from("/engine/assets").join(path),
            asset_type: AssetType::Sound,
            description: None,
            origin: AssetOrigin::Builtin,
        }
    }
}
