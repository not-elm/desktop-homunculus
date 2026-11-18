//! The built-in Speech to Text panel, opened from the "Speech to Text" tray
//! item.

use crate::panel::TrayPanel;
use bevy::prelude::*;

/// Marks the webview that shows the Speech to Text panel.
#[derive(Component, Debug, Default)]
pub(crate) struct SttPanel;

impl TrayPanel for SttPanel {
    const KEY: &'static str = "stt";
    const TEXT: &'static str = "Speech to Text";
    const ORDER: u32 = 50;
    const HTML: &'static str = include_str!(concat!(env!("OUT_DIR"), "/stt.html"));
    const VIEWPORT_SIZE: Vec2 = Vec2::new(700.0, 800.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BuiltinTrayItems;
    use crate::panel::settings::SettingsPanel;
    use crate::panel::test_support::{click, opened, panel_app, se_log};
    use bevy_cef::prelude::{WebviewResizable, WebviewSource};
    use homunculus_core::prelude::LinkedPersona;

    /// Asserts that Speech to Text is registered as a built-in tray item
    /// listed right above Settings.
    ///
    /// Case: the engine starts and builds its tray menu.
    #[test]
    fn stt_is_listed_right_above_settings() {
        let app = panel_app();
        let items = app.world().resource::<BuiltinTrayItems>();

        let keys: Vec<&str> = items.sorted().iter().map(|item| item.key).collect();

        assert_eq!(keys, vec!["stt", "settings"]);
        assert_eq!(
            items.get("stt").map(|item| item.text),
            Some("Speech to Text")
        );
    }

    /// Asserts that picking Speech to Text opens one resizable STT panel from
    /// the embedded HTML, linked to no character, and plays the open sound.
    ///
    /// Case: the user picks Speech to Text from the tray menu.
    #[test]
    fn tray_click_opens_the_stt_panel() {
        let mut app = panel_app();

        click(&mut app, "stt");

        let panels = opened::<SttPanel>(&mut app);
        assert_eq!(panels.len(), 1);
        assert!(matches!(
            app.world().get::<WebviewSource>(panels[0]),
            Some(WebviewSource::InlineHtml(html)) if html == SttPanel::HTML
        ));
        assert!(app.world().get::<LinkedPersona>(panels[0]).is_none());
        assert!(app.world().get::<WebviewResizable>(panels[0]).is_some());
        assert_eq!(se_log(&app), vec!["se/open.mp3"]);
    }

    /// Asserts that picking Speech to Text while the STT panel is open opens
    /// nothing and plays nothing.
    ///
    /// Case: the user picks Speech to Text again while the STT panel is
    /// showing.
    #[test]
    fn click_while_the_stt_panel_is_open_does_nothing() {
        let mut app = panel_app();
        click(&mut app, "stt");
        app.update();

        click(&mut app, "stt");

        assert_eq!(opened::<SttPanel>(&mut app).len(), 1);
        assert_eq!(se_log(&app).len(), 1);
    }

    /// Asserts that an open settings panel does not keep Speech to Text from
    /// opening the STT panel.
    ///
    /// Case: the user opens Settings and then picks Speech to Text while the
    /// settings panel is still showing.
    #[test]
    fn stt_opens_while_settings_is_open() {
        let mut app = panel_app();
        click(&mut app, "settings");
        app.update();

        click(&mut app, "stt");

        assert_eq!(opened::<SettingsPanel>(&mut app).len(), 1);
        assert_eq!(opened::<SttPanel>(&mut app).len(), 1);
        assert_eq!(se_log(&app).len(), 2);
    }

    /// Asserts that an open STT panel does not keep Settings from opening the
    /// settings panel.
    ///
    /// Case: the user opens Speech to Text and then picks Settings while the
    /// STT panel is still showing.
    #[test]
    fn settings_opens_while_stt_is_open() {
        let mut app = panel_app();
        click(&mut app, "stt");
        app.update();

        click(&mut app, "settings");

        assert_eq!(opened::<SettingsPanel>(&mut app).len(), 1);
        assert_eq!(opened::<SttPanel>(&mut app).len(), 1);
        assert_eq!(se_log(&app).len(), 2);
    }
}
