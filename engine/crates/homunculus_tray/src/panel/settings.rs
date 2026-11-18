//! The built-in settings panel, opened from the "Settings" tray item.

use crate::panel::TrayPanel;
use bevy::prelude::*;

/// Marks the webview that shows the settings panel.
#[derive(Component, Debug, Default)]
pub(crate) struct SettingsPanel;

impl TrayPanel for SettingsPanel {
    const KEY: &'static str = "settings";
    const TEXT: &'static str = "Settings";
    const ORDER: u32 = 100;
    const HTML: &'static str = include_str!(concat!(env!("OUT_DIR"), "/settings.html"));
    const VIEWPORT_SIZE: Vec2 = Vec2::new(600.0, 800.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BuiltinTrayItems;
    use crate::panel::test_support::{
        click, click_twice_in_one_command_queue, opened, panel_app, panel_app_without_sounds,
        se_log,
    };
    use bevy_cef::prelude::{WebviewResizable, WebviewSource};
    use homunculus_core::prelude::LinkedPersona;

    /// Asserts that the plugin registers the built-in tray item "Settings".
    ///
    /// Case: the engine starts and builds its tray menu.
    #[test]
    fn settings_is_a_builtin_tray_item() {
        let app = panel_app();

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
        let mut app = panel_app();

        click(&mut app, "settings");

        let panels = opened::<SettingsPanel>(&mut app);
        assert_eq!(panels.len(), 1);
        assert!(matches!(
            app.world().get::<WebviewSource>(panels[0]),
            Some(WebviewSource::InlineHtml(html)) if html == SettingsPanel::HTML
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
        let mut app = panel_app();
        click(&mut app, "settings");
        app.update();

        click(&mut app, "settings");

        assert_eq!(opened::<SettingsPanel>(&mut app).len(), 1);
        assert_eq!(se_log(&app).len(), 1);
    }

    /// Asserts that two picks of Settings within one frame open a single
    /// panel and play the open sound once.
    ///
    /// Case: the frame rate is set to 1 fps and the user picks Settings twice
    /// before the next frame.
    #[test]
    fn two_clicks_in_one_frame_open_one_panel() {
        let mut app = panel_app();

        click_twice_in_one_command_queue(&mut app, "settings");

        assert_eq!(opened::<SettingsPanel>(&mut app).len(), 1);
        assert_eq!(se_log(&app).len(), 1);
    }

    /// Asserts that Settings opens the panel again once the panel is closed.
    ///
    /// Case: the user closes the settings panel and later picks Settings
    /// again.
    #[test]
    fn panel_reopens_after_it_is_closed() {
        let mut app = panel_app();
        click(&mut app, "settings");
        let panel = opened::<SettingsPanel>(&mut app)[0];
        app.world_mut().entity_mut(panel).despawn();

        click(&mut app, "settings");

        assert_eq!(opened::<SettingsPanel>(&mut app).len(), 1);
        assert_eq!(se_log(&app).len(), 2);
    }

    /// Asserts that the panel opens even when the open sound is not
    /// registered.
    ///
    /// Case: the built-in sound files are missing from an installation.
    #[test]
    fn panel_opens_without_registered_sounds() {
        let mut app = panel_app_without_sounds();

        click(&mut app, "settings");

        assert_eq!(opened::<SettingsPanel>(&mut app).len(), 1);
        assert!(se_log(&app).is_empty());
    }
}
