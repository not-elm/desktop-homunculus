//! # Homunculus Tray
//!
//! System tray integration for Desktop Homunculus: builds the tray menu from
//! MOD declarations, the built-in Speech to Text and Settings panels, other
//! built-in items, and the built-in Exit item, and handles clicks on it.

mod panel;

use crate::panel::BuiltinPanelsPlugin;
use bevy::ecs::system::SystemId;
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

const BUILTIN_ID_PREFIX: &str = "::builtin::";
const BUILTIN_EXIT_ID: &str = "::builtin::exit";
const BUILTIN_EXIT_TEXT: &str = "Exit";

/// A tray item built into the engine, listed after the MOD items and right
/// before Exit.
#[derive(Debug, Clone)]
pub struct BuiltinTrayItem {
    /// Identifies the item among the built-in items. `"exit"` is reserved.
    pub key: &'static str,
    /// The label shown in the tray menu.
    pub text: &'static str,
    /// Items are listed in ascending order, and by key when orders tie.
    pub order: u32,
    /// The system that runs when the item is picked.
    pub on_click: SystemId,
}

/// The built-in tray items registered by engine features.
#[derive(Resource, Debug, Default)]
pub struct BuiltinTrayItems {
    items: Vec<BuiltinTrayItem>,
}

/// Registers built-in tray items on an [`App`].
pub trait BuiltinTrayAppExt {
    /// Registers a built-in tray item that runs `on_click` when it is picked.
    ///
    /// The item is listed after the MOD items and right before Exit. An item
    /// whose key is `"exit"` or is already registered is skipped with a
    /// warning. Must be called before `Startup`, when the tray menu is built.
    fn add_builtin_tray_item<M>(
        &mut self,
        key: &'static str,
        text: &'static str,
        order: u32,
        on_click: impl IntoSystem<(), (), M> + 'static,
    ) -> &mut Self;
}

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
/// The menu lists the tray items declared by MODs, then the built-in items,
/// and always ends with the built-in Exit item. The built-in items are always
/// Speech to Text and Settings, in that order, plus any registered with
/// [`BuiltinTrayAppExt::add_builtin_tray_item`]. Clicking a MOD item runs its
/// command, clicking a built-in item runs its system, and clicking Exit
/// requests an app exit. Speech to Text and Settings each open their panel
/// unless that panel is already open.
pub struct HomunculusTrayPlugin;

impl Plugin for HomunculusTrayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TrayIconPlugin, BuiltinPanelsPlugin))
            .init_resource::<BuiltinTrayItems>()
            .add_systems(Startup, setup_tray)
            .add_systems(Update, handle_tray_clicks)
            .add_systems(
                PostUpdate,
                (exit_on_builtin_exit_click, run_builtin_item_clicks),
            );
    }
}

impl BuiltinTrayItem {
    fn menu_id(&self) -> String {
        format!("{BUILTIN_ID_PREFIX}{}", self.key)
    }
}

impl BuiltinTrayItems {
    /// Returns the built-in item registered under `key`, or `None` when no
    /// item has that key.
    pub fn get(&self, key: &str) -> Option<&BuiltinTrayItem> {
        self.items.iter().find(|item| item.key == key)
    }

    /// Adds `item` and returns `true`, or returns `false` with a warning when
    /// its key is `"exit"` or is already registered.
    pub(crate) fn push(&mut self, item: BuiltinTrayItem) -> bool {
        if item.menu_id() == BUILTIN_EXIT_ID || self.get(item.key).is_some() {
            warn!(
                "Skipping built-in tray item '{}': the key is reserved or already registered",
                item.key
            );
            return false;
        }
        self.items.push(item);
        true
    }

    fn by_menu_id(&self, menu_id: &str) -> Option<&BuiltinTrayItem> {
        self.get(menu_id.strip_prefix(BUILTIN_ID_PREFIX)?)
    }

    fn sorted(&self) -> Vec<&BuiltinTrayItem> {
        let mut items: Vec<&BuiltinTrayItem> = self.items.iter().collect();
        items.sort_by_key(|item| (item.order, item.key));
        items
    }
}

impl BuiltinTrayAppExt for App {
    fn add_builtin_tray_item<M>(
        &mut self,
        key: &'static str,
        text: &'static str,
        order: u32,
        on_click: impl IntoSystem<(), (), M> + 'static,
    ) -> &mut Self {
        let on_click = self.register_system(on_click);
        let added = self
            .world_mut()
            .get_resource_or_init::<BuiltinTrayItems>()
            .push(BuiltinTrayItem {
                key,
                text,
                order,
                on_click,
            });
        if !added {
            let _ = self.world_mut().unregister_system(on_click);
        }
        self
    }
}

/// Shows the tray icon with a menu of the MOD tray items, the built-in items,
/// and Exit.
fn setup_tray(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mod_registry: Res<ModRegistry>,
    builtin_items: Res<BuiltinTrayItems>,
) {
    let (menu, registry) = build_menu(&mod_registry, &builtin_items);
    commands.insert_resource(registry);
    commands.insert_resource(TrayIcon {
        icon: Some(asset_server.load("icons/icon.png")),
        tooltip: Some("Desktop Homunculus".to_string()),
        menu,
        show_menu_on_left_click: true,
    });
}

/// Runs the command of each clicked MOD tray item without waiting for it to
/// finish, and ignores clicks on built-in items.
///
/// Any other clicked ID that belongs to no MOD tray item is logged as a
/// warning.
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
        if id.starts_with(BUILTIN_ID_PREFIX) {
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

/// Runs the system of each clicked built-in item.
fn run_builtin_item_clicks(
    mut clicks: MessageReader<MenuMessage>,
    builtin_items: Res<BuiltinTrayItems>,
    mut commands: Commands,
) {
    for click in clicks.read() {
        if let Some(item) = builtin_items.by_menu_id(&click.id.0) {
            commands.run_system(item.on_click);
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
/// `ModRegistry` and the registered built-in items.
///
/// MOD items are grouped by position (`top` → `middle` → `bottom`) and sorted
/// alphabetically by mod name within each group, with a separator between
/// groups. The built-in items follow, after a separator when any MOD item
/// precedes them, and the built-in Exit item always comes last with no
/// separator before it. Built-in items are not registered as MOD commands.
fn build_menu(
    mod_registry: &ModRegistry,
    builtin_items: &BuiltinTrayItems,
) -> (Menu, TrayMenuRegistry) {
    let mut registry = TrayMenuRegistry::default();

    let mut entries = collect_tray_entries(mod_registry);
    entries.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut menu_items = build_grouped_menu_items(&entries, &mut registry);
    append_builtin_section(&mut menu_items, builtin_items);
    (Menu::new(menu_items), registry)
}

fn append_builtin_section(menu_items: &mut Vec<MenuItem>, builtin_items: &BuiltinTrayItems) {
    if !menu_items.is_empty() {
        menu_items.push(MenuItem::separator());
    }
    menu_items.extend(
        builtin_items
            .sorted()
            .into_iter()
            .map(|item| MenuItem::common(item.menu_id(), item.text, true, None)),
    );
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
    use bevy::ecs::system::SystemIdMarker;
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
        let (menu, _) = build_menu(
            &registry_with_tray_mods(&[
                ("@hmcs/voicevox", "middle"),
                ("@hmcs/bottom-example", "bottom"),
                ("@hmcs/persona", "top"),
            ]),
            &BuiltinTrayItems::default(),
        );

        assert_eq!(
            menu_texts(&menu),
            vec![
                "@hmcs/persona",
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
        let (menu, _) = build_menu(
            &registry_with_tray_mods(&[
                ("@hmcs/persona", "top"),
                ("@hmcs/bottom-example", "bottom"),
            ]),
            &BuiltinTrayItems::default(),
        );

        assert_eq!(
            menu_texts(&menu),
            vec!["@hmcs/persona", "@hmcs/bottom-example", "Exit"]
        );
        assert_eq!(separator_count(&menu), 2);
    }

    /// Asserts that the built-in Exit item comes last, right after a separator
    /// that follows the MOD items.
    ///
    /// Case: the user opens the tray menu with the persona MOD installed while
    /// no engine feature has registered a built-in item.
    #[test]
    fn build_menu_appends_builtin_exit_after_mod_items() {
        let (menu, _) = build_menu(
            &registry_with_tray_mods(&[("@hmcs/persona", "top")]),
            &BuiltinTrayItems::default(),
        );
        assert_eq!(menu.len(), 3);
        assert!(matches!(menu[1], MenuItem::Separator));
        assert_builtin_exit(&menu[2]);
    }

    /// Asserts that the menu holds only the built-in Exit item, without a
    /// separator, when no MOD declares a tray entry and no built-in item is
    /// registered.
    ///
    /// Case: MOD discovery fails at startup, so the registry is empty, and no
    /// engine feature has registered a built-in item.
    #[test]
    fn build_menu_shows_only_exit_without_mod_items() {
        let (menu, _) = build_menu(&ModRegistry::default(), &BuiltinTrayItems::default());
        assert_eq!(menu.len(), 1);
        assert_builtin_exit(&menu[0]);
    }

    /// Asserts that the built-in Exit item is not registered as a MOD command.
    ///
    /// Case: the tray click handler looks the clicked ID up among MOD commands.
    #[test]
    fn builtin_exit_is_not_registered_as_mod_command() {
        let (_, registry) = build_menu(
            &registry_with_tray_mods(&[("@hmcs/persona", "top")]),
            &BuiltinTrayItems::default(),
        );
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

    /// Asserts that built-in items follow the MOD items after a separator and
    /// sit right before Exit with no separator between them.
    ///
    /// Case: the user opens the tray menu with the persona MOD installed and
    /// the built-in Settings item registered.
    #[test]
    fn build_menu_places_builtin_items_between_mod_items_and_exit() {
        let mut builtins = BuiltinTrayItems::default();
        register(&mut builtins, "settings", "Settings", 0);

        let (menu, _) = build_menu(
            &registry_with_tray_mods(&[("@hmcs/persona", "top")]),
            &builtins,
        );

        assert_eq!(menu_texts(&menu), vec!["@hmcs/persona", "Settings", "Exit"]);
        assert_eq!(separator_count(&menu), 1);
        assert!(matches!(menu[1], MenuItem::Separator));
    }

    /// Asserts that without MOD items the menu holds the built-in items and
    /// Exit with no separator.
    ///
    /// Case: the user opens the tray menu on an installation with no MODs.
    #[test]
    fn build_menu_shows_builtin_items_and_exit_without_mod_items() {
        let mut builtins = BuiltinTrayItems::default();
        register(&mut builtins, "settings", "Settings", 0);

        let (menu, _) = build_menu(&ModRegistry::default(), &builtins);

        assert_eq!(menu_texts(&menu), vec!["Settings", "Exit"]);
        assert_eq!(separator_count(&menu), 0);
    }

    /// Asserts that built-in items are listed by ascending order, and by key
    /// when their orders tie, whatever order they were registered in.
    ///
    /// Case: several engine features register built-in tray items from
    /// plugins that are added in an arbitrary order.
    #[test]
    fn build_menu_orders_builtin_items_by_order_then_key() {
        let mut builtins = BuiltinTrayItems::default();
        register(&mut builtins, "b", "B", 1);
        register(&mut builtins, "a", "A", 1);
        register(&mut builtins, "c", "C", 0);

        let (menu, _) = build_menu(&ModRegistry::default(), &builtins);

        assert_eq!(menu_texts(&menu), vec!["C", "A", "B", "Exit"]);
    }

    /// Asserts that a built-in item is not registered as a MOD command.
    ///
    /// Case: the tray click handler looks the clicked ID up among MOD
    /// commands while the built-in Settings item is registered.
    #[test]
    fn builtin_item_is_not_registered_as_mod_command() {
        let mut builtins = BuiltinTrayItems::default();
        register(&mut builtins, "settings", "Settings", 0);

        let (_, registry) = build_menu(&ModRegistry::default(), &builtins);

        assert!(registry.lookup("::builtin::settings").is_none());
    }

    /// Asserts that clicking a built-in item runs its system within the same
    /// frame.
    ///
    /// Case: the user picks Settings from the tray menu.
    #[test]
    fn builtin_item_click_runs_its_system() {
        let app = click_builtin("::builtin::settings");

        assert_eq!(app.world().resource::<Clicks>().0, 1);
    }

    /// Asserts that a MOD item whose prefixed ID ends like a built-in one does
    /// not run the built-in item.
    ///
    /// Case: a MOD named `homunculus` declares a tray item whose ID is
    /// `builtin::settings`.
    #[test]
    fn mod_item_click_does_not_run_builtin_item() {
        let app = click_builtin("homunculus::builtin::settings");

        assert_eq!(app.world().resource::<Clicks>().0, 0);
    }

    /// Asserts that a built-in item registered before the tray plugin is added
    /// is kept by the tray's own setup.
    ///
    /// Case: an engine feature registers its tray item before the tray plugin
    /// is added.
    #[test]
    fn items_registered_before_tray_setup_are_kept() {
        let mut app = App::new();
        app.add_builtin_tray_item("test-item", "Test item", 0, || {});

        app.add_plugins(HomunculusTrayPlugin);

        assert!(
            app.world()
                .resource::<BuiltinTrayItems>()
                .get("test-item")
                .is_some()
        );
    }

    /// Asserts that a built-in item keyed `exit` and a second item with an
    /// already registered key are skipped rather than listed.
    ///
    /// Case: an engine feature registers its tray item under a key that Exit
    /// or another feature already uses.
    #[test]
    fn reserved_and_duplicate_keys_are_skipped() {
        let mut builtins = BuiltinTrayItems::default();
        register(&mut builtins, "exit", "Quit", 0);
        register(&mut builtins, "settings", "Settings", 0);
        register(&mut builtins, "settings", "Settings again", 0);

        let (menu, _) = build_menu(&ModRegistry::default(), &builtins);

        assert!(builtins.get("exit").is_none());
        assert_eq!(menu_texts(&menu), vec!["Settings", "Exit"]);
    }

    /// Asserts that a built-in item skipped for its key leaves no registered
    /// system behind.
    ///
    /// Case: an engine feature registers its tray item under the key that Exit
    /// uses.
    #[test]
    fn skipped_item_leaves_no_registered_system() {
        let mut app = App::new();
        let systems_before = registered_system_count(&mut app);

        app.add_builtin_tray_item("exit", "Quit", 0, || {});

        assert_eq!(registered_system_count(&mut app), systems_before);
    }

    #[derive(Resource, Default)]
    struct Clicks(u32);

    fn register(items: &mut BuiltinTrayItems, key: &'static str, text: &'static str, order: u32) {
        items.push(BuiltinTrayItem {
            key,
            text,
            order,
            on_click: SystemId::from_entity(Entity::PLACEHOLDER),
        });
    }

    fn registered_system_count(app: &mut App) -> usize {
        app.world_mut()
            .query::<&SystemIdMarker>()
            .iter(app.world())
            .count()
    }

    fn click_builtin(menu_id: &str) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<MenuMessage>()
            .init_resource::<Clicks>()
            .add_builtin_tray_item("settings", "Settings", 0, |mut clicks: ResMut<Clicks>| {
                clicks.0 += 1;
            })
            .add_systems(PostUpdate, run_builtin_item_clicks);
        app.world_mut().write_message(MenuMessage {
            id: MenuId::new(menu_id),
        });
        app.update();
        app
    }
}
