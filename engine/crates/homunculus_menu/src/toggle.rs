//! Deciding what a secondary click on a character does to the menu.

use crate::{MENU_HTML, MenuHud};
use bevy::prelude::*;
use bevy_vrm1::prelude::ParentSearcher;
use homunculus_api::prelude::WebviewSpawner;
use homunculus_audio::prelude::{KiraAudioSource, RequestSe};
use homunculus_core::prelude::{
    AssetResolver, LinkedPersona, Persona, PersonaId, TransformArgs, WebviewOpenOptions,
    WebviewSource,
};

const MENU_SIZE: Vec2 = Vec2::new(0.8, 1.0);
const MENU_VIEWPORT_SIZE: Vec2 = Vec2::new(500.0, 600.0);
const MENU_TRANSLATION: Vec3 = Vec3::new(1.0, 0.8, 10.0);
const OPEN_SOUND: &str = "se:open";
const CLOSE_SOUND: &str = "se:close";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuAction {
    Open,
    Close,
    Ignore,
}

/// Opens the right-click menu on a secondary click on a character, and closes
/// it on the next secondary click.
pub(crate) struct MenuTogglePlugin;

impl Plugin for MenuTogglePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PointerClick>()
            .add_systems(Update, toggle_menu);
    }
}

impl MenuAction {
    fn decide(menu_open: bool, persona_has_linked_webview: bool) -> Self {
        if menu_open {
            Self::Close
        } else if persona_has_linked_webview {
            Self::Ignore
        } else {
            Self::Open
        }
    }
}

fn toggle_menu(
    mut clicks: MessageReader<PointerClick>,
    parent_searcher: ParentSearcher,
    personas: Query<&Persona>,
    menus: Query<Entity, With<MenuHud>>,
    linked_personas: Query<&LinkedPersona>,
    mut spawner: WebviewSpawner,
    resolver: AssetResolver,
    mut commands: Commands,
) {
    let Some(persona_id) = nearest_clicked_persona(&mut clicks, &parent_searcher, &personas) else {
        return;
    };
    let has_linked_webview = linked_personas.iter().any(|linked| linked.0 == persona_id);
    match MenuAction::decide(!menus.is_empty(), has_linked_webview) {
        MenuAction::Close => close_menus(&mut commands, &menus, &resolver),
        MenuAction::Open => open_menu(&mut spawner, &mut commands, &resolver, persona_id),
        MenuAction::Ignore => {}
    }
}

fn nearest_clicked_persona(
    clicks: &mut MessageReader<PointerClick>,
    parent_searcher: &ParentSearcher,
    personas: &Query<&Persona>,
) -> Option<PersonaId> {
    clicks
        .read()
        .filter(|click| click.button == PointerButton::Secondary)
        .filter_map(|click| clicked_persona(click, parent_searcher, personas))
        .min_by(|(near, _), (far, _)| near.total_cmp(far))
        .map(|(_, persona)| persona.id.clone())
}

fn clicked_persona<'a>(
    click: &PointerClick,
    parent_searcher: &ParentSearcher,
    personas: &'a Query<&Persona>,
) -> Option<(f32, &'a Persona)> {
    let vrm = parent_searcher.find_vrm(click.entity)?;
    let persona = personas.get(vrm).ok()?;
    Some((click.hit.depth, persona))
}

fn close_menus(
    commands: &mut Commands,
    menus: &Query<Entity, With<MenuHud>>,
    resolver: &AssetResolver,
) {
    for menu in menus.iter() {
        commands.entity(menu).despawn();
    }
    play_se(commands, resolver, CLOSE_SOUND);
}

fn open_menu(
    spawner: &mut WebviewSpawner,
    commands: &mut Commands,
    resolver: &AssetResolver,
    persona_id: PersonaId,
) {
    match spawner.open(menu_webview_options(persona_id)) {
        Ok(mut menu) => {
            menu.insert(MenuHud);
            play_se(commands, resolver, OPEN_SOUND);
        }
        Err(e) => error!("Failed to open the right-click menu: {e}"),
    }
}

fn menu_webview_options(persona_id: PersonaId) -> WebviewOpenOptions {
    WebviewOpenOptions {
        source: WebviewSource::Html {
            content: MENU_HTML.to_string(),
        },
        size: Some(MENU_SIZE),
        viewport_size: Some(MENU_VIEWPORT_SIZE),
        transform: Some(TransformArgs {
            translation: Some(MENU_TRANSLATION),
            rotation: None,
            scale: None,
        }),
        constraints: None,
        linked_persona: Some(persona_id),
        resizable: None,
    }
}

fn play_se(commands: &mut Commands, resolver: &AssetResolver, asset_id: &str) {
    match resolver.load::<KiraAudioSource>(asset_id) {
        Ok(source) => commands.trigger(RequestSe {
            source,
            volume: 1.0,
            speed: 1.0,
            panning: 0.0,
        }),
        Err(e) => warn!("The right-click menu sound is unavailable: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HomunculusMenuPlugin, MENU_HTML, MenuHud};
    use bevy::camera::NormalizedRenderTarget;
    use bevy::picking::backend::HitData;
    use bevy::picking::pointer::{Location, PointerId};
    use bevy_cef::prelude::{WebviewExtendStandardMaterial, WebviewSource};
    use bevy_vrm1::prelude::Vrm;
    use homunculus_audio::prelude::{KiraAudioSource, RequestSe};
    use homunculus_core::prelude::{
        AssetEntry, AssetId, AssetOrigin, AssetRegistry, AssetType, LinkedPersona, Persona,
        PersonaId, PersonaIndex,
    };
    use std::path::PathBuf;
    use std::time::Duration;

    #[derive(Resource, Default)]
    struct SeLog(Vec<String>);

    fn menu_app() -> App {
        let mut app = menu_app_without_sounds();
        let mut registry = app.world_mut().resource_mut::<AssetRegistry>();
        registry.register_builtin(sound_entry("se:open", "se/open.mp3"));
        registry.register_builtin(sound_entry("se:close", "se/close.mp3"));
        app
    }

    fn menu_app_without_sounds() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_asset::<WebviewExtendStandardMaterial>()
            .init_asset::<KiraAudioSource>()
            .init_resource::<AssetRegistry>()
            .init_resource::<PersonaIndex>()
            .init_resource::<SeLog>()
            .add_plugins(HomunculusMenuPlugin)
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

    fn spawn_persona(app: &mut App, id: &str) -> (Entity, Entity) {
        let persona_id = PersonaId(id.to_string());
        let persona = app
            .world_mut()
            .spawn((
                Vrm,
                Persona {
                    id: persona_id.clone(),
                    ..default()
                },
            ))
            .id();
        let mesh = app.world_mut().spawn(ChildOf(persona)).id();
        app.world_mut()
            .resource_mut::<PersonaIndex>()
            .insert(persona_id, persona);
        (persona, mesh)
    }

    fn click(app: &mut App, entity: Entity, button: PointerButton) {
        click_at_depth(app, entity, button, 0.0);
    }

    fn click_at_depth(app: &mut App, entity: Entity, button: PointerButton, depth: f32) {
        let location = Location {
            target: NormalizedRenderTarget::None {
                width: 1,
                height: 1,
            },
            position: Vec2::ZERO,
        };
        app.world_mut().write_message(PointerClick {
            entity,
            pointer: Pointer::new(PointerId::Mouse, location),
            button,
            hit: HitData::new(Entity::PLACEHOLDER, depth, None, None),
            duration: Duration::ZERO,
            count: 1,
        });
    }

    fn secondary_click(app: &mut App, entity: Entity) {
        click(app, entity, PointerButton::Secondary);
        app.update();
    }

    fn menus(app: &mut App) -> Vec<Entity> {
        app.world_mut()
            .query_filtered::<Entity, With<MenuHud>>()
            .iter(app.world())
            .collect()
    }

    fn se_log(app: &App) -> Vec<String> {
        app.world().resource::<SeLog>().0.clone()
    }

    fn linked_persona_of(app: &App, webview: Entity) -> Option<String> {
        app.world()
            .get::<LinkedPersona>(webview)
            .map(|linked| linked.0.0.clone())
    }

    /// Asserts that a secondary click on a character opens one menu linked to
    /// that character, served from the embedded HTML, and plays the open
    /// sound.
    ///
    /// Case: the user right-clicks a character to open its menu.
    #[test]
    fn secondary_click_opens_a_menu_for_the_persona() {
        let mut app = menu_app();
        let (persona, _) = spawn_persona(&mut app, "p1");

        secondary_click(&mut app, persona);

        let menus = menus(&mut app);
        assert_eq!(menus.len(), 1);
        assert_eq!(linked_persona_of(&app, menus[0]).as_deref(), Some("p1"));
        assert!(matches!(
            app.world().get::<WebviewSource>(menus[0]),
            Some(WebviewSource::InlineHtml(html)) if html == MENU_HTML
        ));
        assert_eq!(se_log(&app), vec!["se/open.mp3"]);
    }

    /// Asserts that a secondary click while any menu is open closes it, plays
    /// the close sound, and opens nothing new.
    ///
    /// Case: the user opens one character's menu and then right-clicks a
    /// different character.
    #[test]
    fn secondary_click_while_a_menu_is_open_closes_it() {
        let mut app = menu_app();
        let (first, _) = spawn_persona(&mut app, "p1");
        let (second, _) = spawn_persona(&mut app, "p2");
        secondary_click(&mut app, first);

        secondary_click(&mut app, second);

        assert!(menus(&mut app).is_empty());
        assert_eq!(
            se_log(&app).last().map(String::as_str),
            Some("se/close.mp3")
        );
        app.update();
        assert!(menus(&mut app).is_empty());
    }

    /// Asserts that a secondary click is ignored while another webview is
    /// linked to the character, with no sound.
    ///
    /// Case: a settings window opened from a menu item is showing for the
    /// character, and the user right-clicks the character.
    #[test]
    fn persona_with_another_linked_webview_ignores_the_click() {
        let mut app = menu_app();
        let (persona, _) = spawn_persona(&mut app, "p1");
        app.world_mut()
            .spawn(LinkedPersona(PersonaId("p1".to_string())));

        secondary_click(&mut app, persona);

        assert!(menus(&mut app).is_empty());
        assert!(se_log(&app).is_empty());
    }

    /// Asserts that a primary click does not open the menu.
    ///
    /// Case: the user left-clicks a character.
    #[test]
    fn primary_click_is_ignored() {
        let mut app = menu_app();
        let (persona, _) = spawn_persona(&mut app, "p1");

        click(&mut app, persona, PointerButton::Primary);
        app.update();

        assert!(menus(&mut app).is_empty());
        assert!(se_log(&app).is_empty());
    }

    /// Asserts that one right-click delivered to both the character and its
    /// mesh in the same frame acts once, both when opening and when closing.
    ///
    /// Case: the user right-clicks where the character's mesh and the
    /// character's own entity are both hit by the same click.
    #[test]
    fn two_hits_of_one_click_act_once() {
        let mut app = menu_app();
        let (persona, mesh) = spawn_persona(&mut app, "p1");

        click(&mut app, persona, PointerButton::Secondary);
        click(&mut app, mesh, PointerButton::Secondary);
        app.update();
        assert_eq!(menus(&mut app).len(), 1);
        assert_eq!(se_log(&app), vec!["se/open.mp3"]);
        app.update();
        assert_eq!(menus(&mut app).len(), 1);
        assert_eq!(se_log(&app), vec!["se/open.mp3"]);

        click(&mut app, persona, PointerButton::Secondary);
        click(&mut app, mesh, PointerButton::Secondary);
        app.update();
        assert!(menus(&mut app).is_empty());
        assert_eq!(se_log(&app), vec!["se/open.mp3", "se/close.mp3"]);
        app.update();
        assert!(menus(&mut app).is_empty());
        assert_eq!(se_log(&app), vec!["se/open.mp3", "se/close.mp3"]);
    }

    /// Asserts that a secondary click on the menu itself closes it.
    ///
    /// Case: the user right-clicks inside the open menu.
    #[test]
    fn secondary_click_on_the_menu_closes_it() {
        let mut app = menu_app();
        let (persona, _) = spawn_persona(&mut app, "p1");
        secondary_click(&mut app, persona);
        let menu = menus(&mut app)[0];

        secondary_click(&mut app, menu);

        assert!(menus(&mut app).is_empty());
    }

    /// Asserts that a secondary click on an entity that belongs to no
    /// character does nothing.
    ///
    /// Case: the user right-clicks a webview that is not linked to any
    /// character.
    #[test]
    fn click_without_persona_is_ignored() {
        let mut app = menu_app();
        let unrelated = app.world_mut().spawn_empty().id();

        secondary_click(&mut app, unrelated);

        assert!(menus(&mut app).is_empty());
        assert!(se_log(&app).is_empty());
    }

    /// Asserts that a nearer hit that belongs to no character does not stop
    /// the same click from reaching the character behind it.
    ///
    /// Case: one right-click hits an unrelated webview in front of a
    /// character and the character behind it.
    #[test]
    fn click_behind_an_unrelated_hit_reaches_the_persona() {
        let mut app = menu_app();
        let unrelated = app.world_mut().spawn_empty().id();
        let (persona, _) = spawn_persona(&mut app, "p1");

        click_at_depth(&mut app, unrelated, PointerButton::Secondary, 1.0);
        click_at_depth(&mut app, persona, PointerButton::Secondary, 5.0);
        app.update();

        let menus = menus(&mut app);
        assert_eq!(menus.len(), 1);
        assert_eq!(linked_persona_of(&app, menus[0]).as_deref(), Some("p1"));
    }

    /// Asserts that when one click hits two characters, the nearer one gets
    /// the menu, whatever order the hits arrive in.
    ///
    /// Case: the user right-clicks where one character stands in front of
    /// another, and the click hits both of them.
    #[test]
    fn nearest_persona_gets_the_menu() {
        let mut app = menu_app();
        let (far, _) = spawn_persona(&mut app, "far");
        let (near, _) = spawn_persona(&mut app, "near");

        click_at_depth(&mut app, far, PointerButton::Secondary, 5.0);
        click_at_depth(&mut app, near, PointerButton::Secondary, 1.0);
        app.update();

        let menus = menus(&mut app);
        assert_eq!(menus.len(), 1);
        assert_eq!(linked_persona_of(&app, menus[0]).as_deref(), Some("near"));
    }

    /// Asserts that a menu left without its character is closed.
    ///
    /// Case: a script deletes a character in the same frame in which the user
    /// right-clicks it, so the menu is spawned after its character is gone.
    #[test]
    fn menu_without_its_persona_is_closed() {
        let mut app = menu_app();
        let orphan = app
            .world_mut()
            .spawn((MenuHud, LinkedPersona(PersonaId("gone".to_string()))))
            .id();

        app.update();

        assert!(app.world().get_entity(orphan).is_err());
    }

    /// Asserts that a menu removed together with its character does not stop
    /// the next character's menu from opening.
    ///
    /// Case: a character is deleted while its menu is open, and the user then
    /// right-clicks another character.
    #[test]
    fn despawned_persona_does_not_block_the_next_menu() {
        let mut app = menu_app();
        let (first, _) = spawn_persona(&mut app, "p1");
        let (second, _) = spawn_persona(&mut app, "p2");
        secondary_click(&mut app, first);
        app.world_mut().entity_mut(first).despawn();
        app.update();

        secondary_click(&mut app, second);

        let menus = menus(&mut app);
        assert_eq!(menus.len(), 1);
        assert_eq!(linked_persona_of(&app, menus[0]).as_deref(), Some("p2"));
    }

    /// Asserts that the menu opens even when its sounds are not registered.
    ///
    /// Case: the built-in sound files are missing from an installation.
    #[test]
    fn menu_opens_without_registered_sounds() {
        let mut app = menu_app_without_sounds();
        let (persona, _) = spawn_persona(&mut app, "p1");

        secondary_click(&mut app, persona);

        assert_eq!(menus(&mut app).len(), 1);
        assert!(se_log(&app).is_empty());
    }
}
