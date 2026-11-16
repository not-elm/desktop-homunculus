//! # Homunculus Tray
//!
//! System tray integration for Desktop Homunculus: builds the tray menu from
//! MOD declarations plus the built-in Exit item, and handles clicks on it.

use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy_tray_icon::plugin::TrayIconPlugin;
use bevy_tray_icon::plugin::menu_event::MenuMessage;
use bevy_tray_icon::resource::{Menu, MenuItem, TrayIcon};
use homunculus_core::prelude::{HomunculusConfig, ModRegistry, TrayMenuItem};
use homunculus_utils::runtime::RuntimeResolver;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{error, info, warn};

const BUILTIN_EXIT_ID: &str = "::builtin::exit";
const BUILTIN_EXIT_TEXT: &str = "Exit";

/// Maps prefixed menu IDs (`"{mod_name}::{item_id}"`) to the mod name and
/// command string that should be executed when the item is clicked.
#[derive(Resource, Debug, Default)]
pub struct TrayMenuRegistry {
    entries: HashMap<String, (String, String)>,
}

impl TrayMenuRegistry {
    /// Register all leaf items from a `TrayMenuItem` tree.
    ///
    /// Only items with a `command` are registered. Submenu containers are
    /// traversed recursively but not registered themselves.
    pub fn register_item(&mut self, mod_name: &str, item: &TrayMenuItem) {
        if let Some(ref command) = item.command {
            let prefixed_id = format!("{mod_name}::{}", item.id);
            self.entries
                .insert(prefixed_id, (mod_name.to_string(), command.clone()));
        }
        if let Some(ref children) = item.items {
            for child in children {
                self.register_item(mod_name, child);
            }
        }
    }

    /// Look up the mod name and command for a prefixed menu ID.
    #[inline]
    pub fn lookup(&self, prefixed_id: &str) -> Option<&(String, String)> {
        self.entries.get(prefixed_id)
    }
}

/// Plugin that shows the Desktop Homunculus tray icon and its menu.
///
/// The menu lists the tray items declared by MODs and always ends with the
/// built-in Exit item. Clicking a MOD item runs its command, and clicking
/// Exit requests an app exit.
pub struct HomunculusTrayPlugin;

impl Plugin for HomunculusTrayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TrayIconPlugin)
            .add_systems(Startup, setup_tray)
            .add_systems(Update, handle_tray_clicks)
            .add_systems(PostUpdate, exit_on_builtin_exit_click);
    }
}

/// Startup system that builds the tray icon with menu items from all mods.
fn setup_tray(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mod_registry: Res<ModRegistry>,
) {
    let (menu, registry) = build_menu(&mod_registry);
    commands.insert_resource(registry);
    commands.insert_resource(TrayIcon {
        icon: Some(asset_server.load("icons/icon.png")),
        tooltip: Some("Desktop Homunculus".to_string()),
        menu,
        show_menu_on_left_click: true,
    });
}

/// Update system that handles tray menu click events.
///
/// Looks up the clicked menu ID in the registry and spawns a fire-and-forget
/// task to execute the associated mod command.
fn handle_tray_clicks(
    mut mr: MessageReader<MenuMessage>,
    registry: Option<Res<TrayMenuRegistry>>,
    config: Res<HomunculusConfig>,
    runtime: Res<RuntimeResolver>,
) {
    let Some(registry) = registry else {
        return;
    };
    let mods_dir = config.mods_dir.clone();
    for message in mr.read() {
        let id = &message.id.0;
        if id == BUILTIN_EXIT_ID {
            continue;
        }
        if let Some((_mod_name, command)) = registry.lookup(id) {
            let mods_dir = mods_dir.clone();
            let command = command.clone();
            let runtime = runtime.clone();
            info!("Tray menu clicked: id={id}, command={command}");
            IoTaskPool::get()
                .spawn(async move {
                    execute_command(mods_dir, command, &runtime).await;
                })
                .detach();
        } else {
            warn!("Unknown tray menu id: {id}");
        }
    }
}

/// Execute a mod command via `pnpm exec` in the mods directory.
///
/// Stdout is discarded; stderr is piped and logged on failure.
async fn execute_command(mods_dir: PathBuf, command: String, runtime: &RuntimeResolver) {
    use homunculus_utils::process::CommandNoWindow;
    let (program, pnpm_args) = runtime.pnpm_program_and_args();
    let mut cmd = std::process::Command::new(program);
    cmd.no_window()
        .args(&pnpm_args)
        .arg("exec")
        .arg(&command)
        .current_dir(&mods_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    if !runtime.is_bundled()
        && let Some(path) = homunculus_utils::process::path_with_node_prepended()
    {
        cmd.env("PATH", path);
    }
    let result = cmd.output();

    match result {
        Ok(output) if !output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(
                "Tray command `{command}` failed (status {}): {stderr}",
                output.status
            );
        }
        Err(e) => {
            error!("Failed to spawn tray command `{command}`: {e}");
        }
        _ => {}
    }
}

/// Requests a successful app exit when the built-in Exit item is clicked.
fn exit_on_builtin_exit_click(
    mut clicks: MessageReader<MenuMessage>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for click in clicks.read() {
        if click.id.0 == BUILTIN_EXIT_ID {
            app_exit.write(AppExit::Success);
        }
    }
}

/// Resolve the tray position string to a sort-order group index.
///
/// Returns 0 for `"top"`, 1 for `"middle"` (default), 2 for `"bottom"`.
/// Unknown values are treated as `"middle"` with a warning log.
fn resolve_position(position: Option<&str>, mod_name: &str) -> u8 {
    match position {
        Some("top") => 0,
        Some("bottom") => 2,
        Some("middle") | None => 1,
        Some(unknown) => {
            tracing::warn!(
                "MOD {mod_name}: unknown tray position \"{unknown}\", falling back to \"middle\""
            );
            1
        }
    }
}

/// Builds the tray menu and the MOD command registry from all mods in the
/// `ModRegistry`.
///
/// MOD items are grouped by position (`top` → `middle` → `bottom`) and sorted
/// alphabetically by mod name within each group, with a separator between
/// groups. The built-in Exit item always comes last, after a separator when
/// any MOD item precedes it, and is not registered as a MOD command.
fn build_menu(mod_registry: &ModRegistry) -> (Menu, TrayMenuRegistry) {
    let mut registry = TrayMenuRegistry::default();

    let mut entries = collect_tray_entries(mod_registry);
    entries.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut menu_items = build_grouped_menu_items(&entries, &mut registry);
    append_builtin_exit(&mut menu_items);
    (Menu::new(menu_items), registry)
}

fn append_builtin_exit(menu_items: &mut Vec<MenuItem>) {
    if !menu_items.is_empty() {
        menu_items.push(MenuItem::separator());
    }
    menu_items.push(MenuItem::common(
        BUILTIN_EXIT_ID,
        BUILTIN_EXIT_TEXT,
        true,
        None,
    ));
}

/// Collect (position_key, mod_name, tray_item) tuples from all mods.
fn collect_tray_entries(mod_registry: &ModRegistry) -> Vec<(u8, String, TrayMenuItem)> {
    mod_registry
        .all()
        .iter()
        .filter_map(|mod_info| {
            let tray_item = mod_info.tray.as_ref()?;
            let pos = resolve_position(tray_item.position.as_deref(), &mod_info.name);
            Some((pos, mod_info.name.clone(), tray_item.clone()))
        })
        .collect()
}

/// Build the flat `Vec<MenuItem>` with separators between position groups.
fn build_grouped_menu_items(
    entries: &[(u8, String, TrayMenuItem)],
    registry: &mut TrayMenuRegistry,
) -> Vec<MenuItem> {
    let mut menu_items: Vec<MenuItem> = Vec::new();
    let mut last_group: Option<u8> = None;

    for (group, mod_name, tray_item) in entries {
        if let Some(prev) = last_group
            && *group != prev
        {
            menu_items.push(MenuItem::separator());
        }
        last_group = Some(*group);

        menu_items.push(to_bevy_menu_item(mod_name, tray_item));
        registry.register_item(mod_name, tray_item);
    }

    menu_items
}

/// Convert a `TrayMenuItem` to a `bevy_tray_icon` `MenuItem`.
///
/// Leaf items (those with `command`) become `MenuItem::Common`.
/// Items with `items` become `MenuItem::SubMenu`.
/// IDs are prefixed with `"{mod_name}::"`.
fn to_bevy_menu_item(mod_name: &str, item: &TrayMenuItem) -> MenuItem {
    let prefixed_id = format!("{mod_name}::{}", item.id);
    if let Some(ref children) = item.items {
        let child_items: Vec<MenuItem> = children
            .iter()
            .map(|child| to_bevy_menu_item(mod_name, child))
            .collect();
        MenuItem::submenu(prefixed_id, &item.text, true, Menu::new(child_items))
    } else {
        MenuItem::common(prefixed_id, &item.text, true, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_tray_icon::menu::MenuId;
    use homunculus_core::prelude::ModInfo;
    use homunculus_utils::prelude::TrayMenuItem;

    #[test]
    fn registry_registers_leaf_item() {
        let mut registry = TrayMenuRegistry::default();
        let item = TrayMenuItem {
            id: "open-settings".to_string(),
            text: "Settings".to_string(),
            command: Some("open-ui".to_string()),
            items: None,
            position: None,
        };
        registry.register_item("my-mod", &item);

        let result = registry.lookup("my-mod::open-settings");
        assert!(result.is_some());
        let (mod_name, command) = result.expect("should find registered leaf item");
        assert_eq!(mod_name, "my-mod");
        assert_eq!(command, "open-ui");
    }

    #[test]
    fn registry_registers_submenu_items_recursively() {
        let mut registry = TrayMenuRegistry::default();
        let item = TrayMenuItem {
            id: "tools".to_string(),
            text: "Tools".to_string(),
            command: None,
            items: Some(vec![
                TrayMenuItem {
                    id: "tool-a".to_string(),
                    text: "Tool A".to_string(),
                    command: Some("run-a".to_string()),
                    items: None,
                    position: None,
                },
                TrayMenuItem {
                    id: "tool-b".to_string(),
                    text: "Tool B".to_string(),
                    command: Some("run-b".to_string()),
                    items: None,
                    position: None,
                },
            ]),
            position: None,
        };
        registry.register_item("my-mod", &item);

        // Container should NOT be registered (no command).
        assert!(registry.lookup("my-mod::tools").is_none());

        // Children should be registered.
        let a = registry.lookup("my-mod::tool-a");
        assert!(a.is_some());
        assert_eq!(a.expect("tool-a should be registered").1, "run-a");

        let b = registry.lookup("my-mod::tool-b");
        assert!(b.is_some());
        assert_eq!(b.expect("tool-b should be registered").1, "run-b");
    }

    #[test]
    fn registry_lookup_returns_none_for_unknown_id() {
        let registry = TrayMenuRegistry::default();
        assert!(registry.lookup("nonexistent::id").is_none());
    }

    #[test]
    fn to_bevy_menu_item_leaf() {
        let item = TrayMenuItem {
            id: "open".to_string(),
            text: "Open".to_string(),
            command: Some("do-open".to_string()),
            items: None,
            position: None,
        };
        let bevy_item = to_bevy_menu_item("test-mod", &item);
        match bevy_item {
            MenuItem::Common {
                id, text, enabled, ..
            } => {
                assert_eq!(id.0, "test-mod::open");
                assert_eq!(text, "Open");
                assert!(enabled);
            }
            other => panic!("expected Common, got {other:?}"),
        }
    }

    #[test]
    fn to_bevy_menu_item_submenu() {
        let item = TrayMenuItem {
            id: "parent".to_string(),
            text: "Parent".to_string(),
            command: None,
            items: Some(vec![TrayMenuItem {
                id: "child".to_string(),
                text: "Child".to_string(),
                command: Some("run-child".to_string()),
                items: None,
                position: None,
            }]),
            position: None,
        };
        let bevy_item = to_bevy_menu_item("test-mod", &item);
        match bevy_item {
            MenuItem::SubMenu {
                id,
                text,
                enabled,
                menu,
            } => {
                assert_eq!(id.0, "test-mod::parent");
                assert_eq!(text, "Parent");
                assert!(enabled);
                assert_eq!(menu.len(), 1);
                match &menu[0] {
                    MenuItem::Common { id, text, .. } => {
                        assert_eq!(id.0, "test-mod::child");
                        assert_eq!(text, "Child");
                    }
                    other => panic!("expected Common child, got {other:?}"),
                }
            }
            other => panic!("expected SubMenu, got {other:?}"),
        }
    }

    /// Asserts that MOD items are grouped top, middle, then bottom with a
    /// separator between groups, and that the built-in Exit item comes last.
    ///
    /// Case: the user opens the tray menu with MODs installed at every
    /// position.
    #[test]
    fn build_menu_orders_by_position_then_mod_name() {
        let (menu, _) = build_menu(&registry_with_tray_mods(&[
            ("@hmcs/voicevox", "middle"),
            ("@hmcs/bottom-example", "bottom"),
            ("@hmcs/settings", "top"),
        ]));

        assert_eq!(
            menu_texts(&menu),
            vec![
                "@hmcs/settings",
                "@hmcs/voicevox",
                "@hmcs/bottom-example",
                "Exit"
            ]
        );
        assert_eq!(separator_count(&menu), 3);
    }

    /// Asserts that a position group with no MOD items gets no separator of
    /// its own.
    ///
    /// Case: the user has MODs only at the top and bottom positions, so the
    /// middle group is empty.
    #[test]
    fn build_menu_skips_separator_for_empty_groups() {
        let (menu, _) = build_menu(&registry_with_tray_mods(&[
            ("@hmcs/settings", "top"),
            ("@hmcs/bottom-example", "bottom"),
        ]));

        assert_eq!(
            menu_texts(&menu),
            vec!["@hmcs/settings", "@hmcs/bottom-example", "Exit"]
        );
        assert_eq!(separator_count(&menu), 2);
    }

    /// Asserts that the built-in Exit item comes last, right after a separator
    /// that follows the MOD items.
    ///
    /// Case: the user opens the tray menu with the settings MOD installed.
    #[test]
    fn build_menu_appends_builtin_exit_after_mod_items() {
        let (menu, _) = build_menu(&registry_with_tray_mods(&[("@hmcs/settings", "top")]));
        assert_eq!(menu.len(), 3);
        assert!(matches!(menu[1], MenuItem::Separator));
        assert_builtin_exit(&menu[2]);
    }

    /// Asserts that the menu holds only the built-in Exit item, without a
    /// separator, when no MOD declares a tray entry.
    ///
    /// Case: MOD discovery fails at startup, so the registry is empty.
    #[test]
    fn build_menu_shows_only_exit_without_mod_items() {
        let (menu, _) = build_menu(&ModRegistry::default());
        assert_eq!(menu.len(), 1);
        assert_builtin_exit(&menu[0]);
    }

    /// Asserts that the built-in Exit item is not registered as a MOD command.
    ///
    /// Case: the tray click handler looks the clicked ID up among MOD commands.
    #[test]
    fn builtin_exit_is_not_registered_as_mod_command() {
        let (_, registry) = build_menu(&registry_with_tray_mods(&[("@hmcs/settings", "top")]));
        assert!(registry.lookup(BUILTIN_EXIT_ID).is_none());
    }

    /// Asserts that clicking the built-in Exit item requests a successful app
    /// exit within the same frame.
    ///
    /// Case: the user picks Exit from the tray menu.
    #[test]
    fn builtin_exit_click_requests_app_exit() {
        assert_eq!(click(BUILTIN_EXIT_ID).should_exit(), Some(AppExit::Success));
    }

    /// Asserts that clicking a MOD item does not request an app exit, even when
    /// its prefixed ID ends like the built-in one.
    ///
    /// Case: a MOD named `homunculus` declares a tray item whose ID is
    /// `builtin::exit`.
    #[test]
    fn mod_item_click_does_not_request_app_exit() {
        assert_eq!(click("homunculus::builtin::exit").should_exit(), None);
    }

    fn registry_with_tray_mods(mods: &[(&str, &str)]) -> ModRegistry {
        let mut registry = ModRegistry::default();
        for (name, position) in mods {
            registry.register(ModInfo {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: None,
                author: None,
                license: None,
                service_script_path: None,
                commands: vec![],
                assets: HashMap::new(),
                menus: vec![],
                tray: Some(TrayMenuItem {
                    id: format!("{name}-tray"),
                    text: name.to_string(),
                    command: Some(format!("{name}-cmd")),
                    items: None,
                    position: Some(position.to_string()),
                }),
                mod_dir: PathBuf::from("/tmp"),
            });
        }
        registry
    }

    fn menu_texts(menu: &Menu) -> Vec<&str> {
        menu.iter()
            .filter_map(|item| match item {
                MenuItem::Common { text, .. } => Some(text.as_str()),
                MenuItem::SubMenu { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    fn separator_count(menu: &Menu) -> usize {
        menu.iter()
            .filter(|item| matches!(item, MenuItem::Separator))
            .count()
    }

    fn assert_builtin_exit(item: &MenuItem) {
        match item {
            MenuItem::Common { id, text, .. } => {
                assert_eq!(id.0, BUILTIN_EXIT_ID);
                assert_eq!(text, BUILTIN_EXIT_TEXT);
            }
            other => panic!("expected the built-in Exit item, got {other:?}"),
        }
    }

    fn click(menu_id: &str) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<MenuMessage>()
            .add_systems(PostUpdate, exit_on_builtin_exit_click);
        app.world_mut().write_message(MenuMessage {
            id: MenuId::new(menu_id),
        });
        app.update();
        app
    }
}
