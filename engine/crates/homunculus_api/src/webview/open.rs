//! Opening webviews in world space, from the HTTP API or from a Bevy system.

use crate::error::{ApiError, ApiResult};
use crate::prelude::WebviewApi;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy_cef::prelude::{
    PreloadScripts, WebviewExtendStandardMaterial, WebviewResizable, WebviewSize,
};
use bevy_flurx::action::once;
use bevy_vrm1::prelude::Cameras;
use homunculus_core::prelude::{
    AspectLockMode, AssetResolver, AssetType, LinkedPersona, PersonaIndex, TransformConstraint,
    WebviewMeshSize, WebviewOpenOptions, WebviewResizableOptions, WebviewSource, WebviewSourceInfo,
};
use homunculus_effects::{Entity, Update};

/// Tracks the original API-level source used to create/navigate a webview.
#[derive(Component, Debug, Clone)]
pub(crate) struct OriginalWebviewSource(pub WebviewSource);

/// Opens webviews in world space from a Bevy system.
#[derive(SystemParam)]
pub struct WebviewSpawner<'w, 's> {
    commands: Commands<'w, 's>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<WebviewExtendStandardMaterial>>,
    cameras: Cameras<'w, 's>,
    asset_resolver: AssetResolver<'w>,
    index: Res<'w, PersonaIndex>,
}

impl WebviewApi {
    /// Opens a global webview in world space.
    pub async fn open(&self, options: WebviewOpenOptions) -> ApiResult<Entity> {
        self.0
            .schedule(move |task| async move {
                task.will(Update, once::run(create_global_webview).with(options))
                    .await
            })
            .await?
    }
}

impl WebviewSpawner<'_, '_> {
    /// Spawns a webview described by `options` and returns its commands.
    ///
    /// The webview gets its mesh and material, the preload script that tells
    /// the page its own entity, and a resize handle when `options.resizable`
    /// is set. A linked persona is recorded as [`LinkedPersona`]; when that
    /// persona is spawned, the webview also becomes its child and follows it.
    /// Components inserted through the returned commands are applied on the
    /// same command queue as the spawn.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AssetNotFound`] when a local source names an
    /// unknown asset, and [`ApiError::AssetTypeMismatch`] when it names an
    /// asset that is not HTML.
    pub fn open(&mut self, options: WebviewOpenOptions) -> ApiResult<EntityCommands<'_>> {
        let webview_source = source_to_webview_source(&options.source, &self.asset_resolver)?;
        let webview = spawn_webview_entity(
            &mut self.commands,
            &mut self.meshes,
            &mut self.materials,
            &self.cameras,
            webview_source,
            &options,
        );
        let persona_entity = options
            .linked_persona
            .as_ref()
            .and_then(|persona_id| self.index.get(persona_id));
        let mut entity = self.commands.entity(webview);
        entity.try_insert((
            OriginalWebviewSource(options.source),
            preload_scripts(webview),
        ));
        if let Some(resizable) = options.resizable {
            entity.try_insert(to_webview_resizable(resizable));
        }
        if let Some(persona_id) = options.linked_persona {
            entity.try_insert(LinkedPersona(persona_id));
        }
        if let Some(persona_entity) = persona_entity {
            entity.insert(ChildOf(persona_entity));
        }
        Ok(entity)
    }
}

pub(crate) struct WebviewOpenPlugin;

impl Plugin for WebviewOpenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, visible);
    }
}

fn visible(
    mut webviews: Query<(
        &mut Visibility,
        &MeshMaterial3d<WebviewExtendStandardMaterial>,
    )>,
    materials: Res<Assets<WebviewExtendStandardMaterial>>,
) {
    for (mut visibility, handle) in webviews.iter_mut() {
        if matches!(*visibility, Visibility::Hidden)
            && let Some(material) = materials.get(handle)
            && material.extension.surface.is_some()
        {
            *visibility = Visibility::Visible;
        }
    }
}

fn create_global_webview(
    In(options): In<WebviewOpenOptions>,
    mut spawner: WebviewSpawner,
) -> ApiResult<Entity> {
    spawner.open(options).map(|webview| webview.id())
}

/// Converts our API-level `WebviewSource` to bevy_cef's `WebviewSource` component.
pub(crate) fn source_to_webview_source(
    source: &WebviewSource,
    asset_resolver: &AssetResolver,
) -> ApiResult<bevy_cef::prelude::WebviewSource> {
    match source {
        WebviewSource::Html { content } => Ok(bevy_cef::prelude::WebviewSource::inline(content)),
        WebviewSource::Url { url } => Ok(bevy_cef::prelude::WebviewSource::new(url)),
        WebviewSource::Local { id } => {
            let entry = asset_resolver
                .resolve(id)
                .map_err(|_| ApiError::AssetNotFound(id.clone()))?;
            if entry.asset_type != AssetType::Html {
                return Err(ApiError::AssetTypeMismatch {
                    id: id.clone(),
                    expected: AssetType::Html,
                    actual: entry.asset_type.clone(),
                });
            }
            // Use forward slashes for URL compatibility. Windows backslash paths
            // in `cef://localhost/C:\...` URLs can be mangled by CEF's URL parser.
            let path = entry.absolute_path.display().to_string().replace('\\', "/");
            Ok(bevy_cef::prelude::WebviewSource::local(path))
        }
    }
}

/// Converts our API-level `WebviewSource` to `WebviewSourceInfo` for responses.
pub(crate) fn webview_source_to_info(
    source: &WebviewSource,
    include_content: bool,
) -> WebviewSourceInfo {
    match source {
        WebviewSource::Url { url } => WebviewSourceInfo::Url { url: url.clone() },
        WebviewSource::Html { content } => WebviewSourceInfo::Html {
            content: if include_content {
                Some(content.clone())
            } else {
                None
            },
        },
        WebviewSource::Local { id } => WebviewSourceInfo::Local { id: id.clone() },
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_webview_entity(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<WebviewExtendStandardMaterial>>,
    cameras: &Cameras,
    webview_source: bevy_cef::prelude::WebviewSource,
    options: &WebviewOpenOptions,
) -> Entity {
    let size = options.size.unwrap_or(Vec2::splat(0.7));
    let translation = options
        .transform
        .as_ref()
        .and_then(|t| t.translation)
        .unwrap_or(Vec3::new(0.0, 0.0, 10.0));
    let constraint = build_transform_constraint(options);
    let mut entity_commands = commands.spawn((
        Name::new("Webview"),
        webview_source,
        cameras.all_layers(),
        NotShadowCaster,
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, size))),
        MeshMaterial3d(materials.add(WebviewExtendStandardMaterial {
            base: StandardMaterial {
                unlit: true,
                alpha_mode: AlphaMode::Premultiplied,
                ..default()
            },
            ..default()
        })),
        Pickable {
            should_block_lower: false,
            is_hoverable: true,
        },
        Visibility::Hidden,
        Transform::from_translation(translation),
        constraint,
        WebviewMeshSize(size),
    ));

    if let Some(size) = options.viewport_size {
        entity_commands.try_insert(WebviewSize(size));
    }

    entity_commands.id()
}

fn build_transform_constraint(options: &WebviewOpenOptions) -> TransformConstraint {
    let mut constraint = TransformConstraint::default();
    if let Some(c) = &options.constraints {
        if let Some(v) = c.rotation_follow {
            constraint.rotation_follow = v;
        }
        if let Some(v) = c.max_tilt_degrees {
            constraint.max_tilt_degrees = v;
        }
        if let Some(v) = c.lock_scale {
            constraint.lock_scale = v;
        }
    }
    constraint
}

fn preload_scripts(webview: Entity) -> PreloadScripts {
    PreloadScripts::from([&include_str!("../webview/webviewEntity.js")
        .replace("undefined", &webview.to_bits().to_string())])
}

fn to_aspect_lock_mode(mode: AspectLockMode) -> bevy_cef::prelude::AspectLockMode {
    match mode {
        AspectLockMode::LockOnShift => bevy_cef::prelude::AspectLockMode::LockOnShift,
        AspectLockMode::Always => bevy_cef::prelude::AspectLockMode::Always,
        AspectLockMode::Never => bevy_cef::prelude::AspectLockMode::Never,
    }
}

fn to_webview_resizable(opts: WebviewResizableOptions) -> WebviewResizable {
    WebviewResizable {
        edge_thickness: opts.edge_thickness,
        min_size: opts.min_size,
        max_size: opts.max_size,
        aspect_lock: to_aspect_lock_mode(opts.aspect_lock),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use homunculus_core::prelude::{AssetEntry, AssetId, AssetOrigin, AssetRegistry, PersonaId};
    use std::path::PathBuf;

    fn spawner_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_asset::<WebviewExtendStandardMaterial>()
            .init_resource::<AssetRegistry>()
            .init_resource::<PersonaIndex>();
        app
    }

    fn options(source: WebviewSource, linked_persona: Option<PersonaId>) -> WebviewOpenOptions {
        WebviewOpenOptions {
            source,
            size: None,
            viewport_size: None,
            transform: None,
            constraints: None,
            linked_persona,
            resizable: None,
        }
    }

    fn open(app: &mut App, options: WebviewOpenOptions) -> ApiResult<Entity> {
        app.world_mut()
            .run_system_once_with(
                |In(options): In<WebviewOpenOptions>, mut spawner: WebviewSpawner| {
                    spawner.open(options).map(|webview| webview.id())
                },
                options,
            )
            .expect("the system runs")
    }

    /// Asserts that an inline HTML source opens a webview that serves the
    /// HTML inline and carries the preload script naming its own entity.
    ///
    /// Case: the engine opens its built-in right-click menu from HTML embedded
    /// in the binary.
    #[test]
    fn html_source_opens_an_inline_webview() {
        let mut app = spawner_app();

        let webview = open(
            &mut app,
            options(
                WebviewSource::Html {
                    content: "<p>menu</p>".to_string(),
                },
                None,
            ),
        )
        .expect("the webview opens");

        let entity = app.world().entity(webview);
        assert!(matches!(
            entity.get::<bevy_cef::prelude::WebviewSource>(),
            Some(bevy_cef::prelude::WebviewSource::InlineHtml(html)) if html == "<p>menu</p>"
        ));
        assert!(matches!(
            entity.get::<OriginalWebviewSource>(),
            Some(OriginalWebviewSource(WebviewSource::Html { .. }))
        ));
        let scripts = entity
            .get::<PreloadScripts>()
            .expect("preload scripts are attached");
        let entity_bits = webview.to_bits().to_string();
        assert!(
            scripts.0.iter().any(|script| script.contains(&entity_bits)),
            "no preload script names entity {entity_bits}"
        );
    }

    /// Asserts that a webview linked to a spawned persona is linked to it and
    /// parented to its entity.
    ///
    /// Case: a menu opened for a character follows that character on screen.
    #[test]
    fn linked_persona_is_parented_to_the_persona() {
        let mut app = spawner_app();
        let persona = app.world_mut().spawn_empty().id();
        let persona_id = PersonaId("p1".to_string());
        app.world_mut()
            .resource_mut::<PersonaIndex>()
            .insert(persona_id.clone(), persona);

        let webview = open(
            &mut app,
            options(
                WebviewSource::Html {
                    content: "<p>menu</p>".to_string(),
                },
                Some(persona_id.clone()),
            ),
        )
        .expect("the webview opens");

        let entity = app.world().entity(webview);
        assert_eq!(
            entity.get::<LinkedPersona>().map(|linked| &linked.0),
            Some(&persona_id)
        );
        assert_eq!(entity.get::<ChildOf>().map(ChildOf::parent), Some(persona));
    }

    /// Asserts that a local source naming a non-HTML asset is rejected.
    ///
    /// Case: a MOD asks to open a webview from the ID of a VRM model by
    /// mistake.
    #[test]
    fn local_source_must_be_an_html_asset() {
        let mut app = spawner_app();
        app.world_mut()
            .resource_mut::<AssetRegistry>()
            .register(AssetEntry {
                id: AssetId::new("vrm:x"),
                path: PathBuf::from("x.vrm"),
                absolute_path: PathBuf::from("/mods/x/x.vrm"),
                asset_type: AssetType::Vrm,
                description: None,
                origin: AssetOrigin::Mod("x".to_string()),
            });

        let result = open(
            &mut app,
            options(
                WebviewSource::Local {
                    id: AssetId::new("vrm:x"),
                },
                None,
            ),
        );

        assert!(matches!(result, Err(ApiError::AssetTypeMismatch { .. })));
    }
}
