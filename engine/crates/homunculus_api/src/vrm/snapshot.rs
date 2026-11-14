use crate::error::ApiResult;
use crate::vrm::VrmApi;
use crate::vrm::expressions::{ExpressionsResponse, expressions_response};
use crate::vrma::VrmaInfo;
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use bevy_vrm1::prelude::*;
use homunculus_core::prelude::{
    AssetIdComponent, Coordinate, GlobalViewport, LinkedPersona, Persona, PersonaState,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct VrmSnapshot {
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub entity: Entity,
    pub name: String,
    pub state: String,
    #[cfg_attr(feature = "openapi", schema(value_type = Object))]
    pub transform: Transform,
    #[cfg_attr(feature = "openapi", schema(value_type = Option<Object>))]
    pub global_viewport: Option<GlobalViewport>,
    pub expressions: ExpressionsResponse,
    pub animations: Vec<VrmaInfo>,
    pub look_at: Option<LookAtState>,
    #[cfg_attr(feature = "openapi", schema(value_type = Vec<String>))]
    pub linked_webviews: Vec<Entity>,
    #[cfg_attr(feature = "openapi", schema(value_type = Object))]
    pub persona: Persona,
    pub asset_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum LookAtState {
    Cursor,
    Target {
        #[cfg_attr(feature = "openapi", schema(value_type = String))]
        entity: Entity,
    },
}

impl VrmApi {
    pub async fn snapshot(&self) -> ApiResult<Vec<VrmSnapshot>> {
        self.0
            .schedule(
                move |task| async move { task.will(Update, once::run(snapshot_all_vrms)).await },
            )
            .await
    }
}

#[allow(clippy::too_many_arguments)]
fn snapshot_all_vrms(
    vrms: Query<
        (
            Entity,
            &Name,
            Option<&PersonaState>,
            &Transform,
            Option<&LookAt>,
            Option<&Persona>,
            Option<&ExpressionSettings>,
            Option<&ExpressionOverrides>,
            Option<&VrmExpressionWeights>,
            Option<&AssetIdComponent>,
        ),
        With<Vrm>,
    >,
    children_query: Query<&Children>,
    vrma_query: Query<(Entity, &Name, &VrmaAnimationPlayers), With<Vrma>>,
    players: Query<&AnimationPlayer>,
    coordinate: Coordinate,
    linked_personas: Query<(Entity, &LinkedPersona)>,
) -> Vec<VrmSnapshot> {
    vrms.iter()
        .map(
            |(
                entity,
                name,
                state,
                transform,
                look_at,
                persona,
                expr_settings,
                expr_overrides,
                expr_weights,
                asset_id_comp,
            )| {
                let expressions = expressions_response(expr_settings, expr_overrides, expr_weights);
                let animations =
                    collect_playing_animations(entity, &children_query, &vrma_query, &players);
                let global_viewport = coordinate.to_global_by_world(transform.translation);
                let look_at_state = look_at.map(|la| match la {
                    LookAt::Cursor => LookAtState::Cursor,
                    LookAt::Target(target) => LookAtState::Target { entity: *target },
                });
                let linked_webviews = persona
                    .map(|p| {
                        linked_personas
                            .iter()
                            .filter(|(_, linked)| linked.0 == p.id)
                            .map(|(webview_entity, _)| webview_entity)
                            .collect()
                    })
                    .unwrap_or_default();

                VrmSnapshot {
                    entity,
                    name: name.to_string(),
                    state: state
                        .map(|s| s.0.clone())
                        .unwrap_or_else(|| "idle".to_string()),
                    transform: *transform,
                    global_viewport,
                    expressions,
                    animations,
                    look_at: look_at_state,
                    linked_webviews,
                    persona: persona.cloned().unwrap_or_default(),
                    asset_id: asset_id_comp.map(|c| c.0.to_string()),
                }
            },
        )
        .collect()
}

fn collect_playing_animations(
    vrm: Entity,
    children: &Query<&Children>,
    vrma_query: &Query<(Entity, &Name, &VrmaAnimationPlayers), With<Vrma>>,
    players: &Query<&AnimationPlayer>,
) -> Vec<VrmaInfo> {
    let mut result = Vec::new();
    let Ok(vrm_children) = children.get(vrm) else {
        return result;
    };
    for child in vrm_children.iter() {
        if let Ok((entity, name, animation_players)) = vrma_query.get(child) {
            let playing = animation_players
                .0
                .iter()
                .any(|&pe| players.get(pe).map(|p| !p.all_finished()).unwrap_or(false));
            if playing {
                result.push(VrmaInfo {
                    entity,
                    name: name.to_string(),
                    playing: true,
                });
            }
        }
    }
    result
}
