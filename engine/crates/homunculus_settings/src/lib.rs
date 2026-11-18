//! The settings panel built into the engine: a built-in "Settings" tray item
//! that opens the panel.

use bevy::prelude::*;
use homunculus_api::prelude::WebviewSpawner;
use homunculus_audio::prelude::SePlayer;
use homunculus_core::prelude::{
    TransformArgs, WebviewOpenOptions, WebviewResizableOptions, WebviewSource,
};
use homunculus_tray::BuiltinTrayAppExt;

const SETTINGS_HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/settings.html"));
const TRAY_KEY: &str = "settings";
const TRAY_TEXT: &str = "Settings";
const TRAY_ORDER: u32 = 100;
const PANEL_SIZE: Vec2 = Vec2::new(0.8, 1.0);
const PANEL_VIEWPORT_SIZE: Vec2 = Vec2::new(600.0, 800.0);
const PANEL_TRANSLATION: Vec3 = Vec3::new(0.0, 0.0, 10.0);
const OPEN_SOUND: &str = "se:open";

/// Marks the webview that shows the settings panel.
#[derive(Component, Debug)]
struct SettingsPanel;

/// Adds the built-in tray item "Settings", which opens the settings panel
/// unless one is already open.
///
/// The panel is opened from HTML embedded in the binary, linked to no
/// character, and resizable; opening it plays `se:open`.
pub struct HomunculusSettingsPlugin;

impl Plugin for HomunculusSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_builtin_tray_item(TRAY_KEY, TRAY_TEXT, TRAY_ORDER, open_settings_panel);
    }
}

fn open_settings_panel(
    panels: Query<(), With<SettingsPanel>>,
    mut spawner: WebviewSpawner,
    mut se: SePlayer,
) {
    if !panels.is_empty() {
        return;
    }
    match spawner.open(settings_panel_options()) {
        Ok(mut panel) => {
            panel.insert(SettingsPanel);
            se.play(OPEN_SOUND);
        }
        Err(e) => error!("Failed to open the settings panel: {e}"),
    }
}

fn settings_panel_options() -> WebviewOpenOptions {
    WebviewOpenOptions {
        source: WebviewSource::Html {
            content: SETTINGS_HTML.to_string(),
        },
        size: Some(PANEL_SIZE),
        viewport_size: Some(PANEL_VIEWPORT_SIZE),
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
mod tests {
    use super::*;
    use bevy_cef::prelude::{WebviewExtendStandardMaterial, WebviewResizable, WebviewSource};
    use homunculus_audio::prelude::{KiraAudioSource, RequestSe};
    use homunculus_core::prelude::{
        AssetEntry, AssetId, AssetOrigin, AssetRegistry, AssetType, LinkedPersona, PersonaIndex,
    };
    use homunculus_tray::BuiltinTrayItems;
    use std::path::PathBuf;

    #[derive(Resource, Default)]
    struct SeLog(Vec<String>);

    /// Asserts that the plugin registers the built-in tray item "Settings".
    ///
    /// Case: the engine starts and builds its tray menu.
    #[test]
    fn settings_is_a_builtin_tray_item() {
        let app = settings_app();

        let text = app
            .world()
            .resource::<BuiltinTrayItems>()
            .get("settings")
            .map(|item| item.text);

        assert_eq!(text, Some("Settings"));
    }

    /// Asserts that picking Settings opens one resizable panel from the
    /// embedded HTML, linked to no character, and plays the open sound.
    ///
    /// Case: the user picks Settings from the tray menu.
    #[test]
    fn tray_click_opens_the_settings_panel() {
        let mut app = settings_app();

        click_settings(&mut app);

        let panels = panels(&mut app);
        assert_eq!(panels.len(), 1);
        assert!(matches!(
            app.world().get::<WebviewSource>(panels[0]),
            Some(WebviewSource::InlineHtml(html)) if html == SETTINGS_HTML
        ));
        assert!(app.world().get::<LinkedPersona>(panels[0]).is_none());
        assert!(app.world().get::<WebviewResizable>(panels[0]).is_some());
        assert_eq!(se_log(&app), vec!["se/open.mp3"]);
    }

    /// Asserts that picking Settings while the panel is open opens nothing
    /// and plays nothing.
    ///
    /// Case: the user picks Settings again while the settings panel is
    /// showing.
    #[test]
    fn click_while_the_panel_is_open_does_nothing() {
        let mut app = settings_app();
        click_settings(&mut app);
        app.update();

        click_settings(&mut app);

        assert_eq!(panels(&mut app).len(), 1);
        assert_eq!(se_log(&app).len(), 1);
    }

    /// Asserts that two picks of Settings within one frame open a single
    /// panel and play the open sound once.
    ///
    /// Case: the frame rate is set to 1 fps and the user picks Settings twice
    /// before the next frame.
    #[test]
    fn two_clicks_in_one_frame_open_one_panel() {
        let mut app = settings_app();

        click_settings(&mut app);
        click_settings(&mut app);

        assert_eq!(panels(&mut app).len(), 1);
        assert_eq!(se_log(&app).len(), 1);
    }

    /// Asserts that Settings opens the panel again once the panel is closed.
    ///
    /// Case: the user closes the settings panel and later picks Settings
    /// again.
    #[test]
    fn panel_reopens_after_it_is_closed() {
        let mut app = settings_app();
        click_settings(&mut app);
        let panel = panels(&mut app)[0];
        app.world_mut().entity_mut(panel).despawn();

        click_settings(&mut app);

        assert_eq!(panels(&mut app).len(), 1);
        assert_eq!(se_log(&app).len(), 2);
    }

    /// Asserts that the panel opens even when the open sound is not
    /// registered.
    ///
    /// Case: the built-in sound files are missing from an installation.
    #[test]
    fn panel_opens_without_registered_sounds() {
        let mut app = settings_app_without_sounds();

        click_settings(&mut app);

        assert_eq!(panels(&mut app).len(), 1);
        assert!(se_log(&app).is_empty());
    }

    fn settings_app() -> App {
        let mut app = settings_app_without_sounds();
        app.world_mut()
            .resource_mut::<AssetRegistry>()
            .register_builtin(sound_entry("se:open", "se/open.mp3"));
        app
    }

    fn settings_app_without_sounds() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_asset::<WebviewExtendStandardMaterial>()
            .init_asset::<KiraAudioSource>()
            .init_resource::<AssetRegistry>()
            .init_resource::<PersonaIndex>()
            .init_resource::<SeLog>()
            .add_plugins(HomunculusSettingsPlugin)
            .add_observer(|trigger: On<RequestSe>, mut log: ResMut<SeLog>| {
                let path = trigger.source.path().map(ToString::to_string);
                log.0.push(path.unwrap_or_default());
            });
        app
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

    fn click_settings(app: &mut App) {
        let on_click = app
            .world()
            .resource::<BuiltinTrayItems>()
            .get("settings")
            .expect("Settings is registered")
            .on_click;
        app.world_mut()
            .run_system(on_click)
            .expect("the Settings system runs");
    }

    fn panels(app: &mut App) -> Vec<Entity> {
        app.world_mut()
            .query_filtered::<Entity, With<SettingsPanel>>()
            .iter(app.world())
            .collect()
    }

    fn se_log(app: &App) -> Vec<String> {
        app.world().resource::<SeLog>().0.clone()
    }
}
