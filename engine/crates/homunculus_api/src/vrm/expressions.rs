use crate::error::ApiResult;
use crate::vrm::VrmApi;
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use bevy_vrm1::prelude::*;
use homunculus_core::prelude::{ExpressionChangeEvent, OutputLog, VrmEvent, VrmEventSender};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct ExpressionInfo {
    pub name: String,
    pub weight: f32,
    pub is_binary: bool,
    pub override_blink: String,
    pub override_look_at: String,
    pub override_mouth: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExpressionsResponse {
    pub expressions: Vec<ExpressionInfo>,
}

impl VrmApi {
    pub async fn list_expressions(&self, vrm: Entity) -> ApiResult<ExpressionsResponse> {
        self.0
            .schedule(move |task| async move {
                task.will(Update, once::run(list_expressions).with(vrm))
                    .await
            })
            .await
    }

    pub async fn set_expressions(&self, vrm: Entity, weights: HashMap<String, f32>) -> ApiResult {
        self.0
            .schedule(move |task| async move {
                task.will(Update, once::run(do_set_expressions).with((vrm, weights)))
                    .await;
            })
            .await
    }

    pub async fn modify_expressions(
        &self,
        vrm: Entity,
        weights: HashMap<String, f32>,
    ) -> ApiResult {
        self.0
            .schedule(move |task| async move {
                task.will(
                    Update,
                    once::run(do_modify_expressions).with((vrm, weights)),
                )
                .await;
            })
            .await
    }

    pub async fn modify_mouth(&self, vrm: Entity, weights: HashMap<String, f32>) -> ApiResult {
        self.0
            .schedule(move |task| async move {
                task.will(Update, once::run(do_modify_mouth).with((vrm, weights)))
                    .await;
            })
            .await
    }

    pub async fn clear_expressions(&self, vrm: Entity) -> ApiResult {
        self.0
            .schedule(move |task| async move {
                task.will(Update, once::run(do_clear_expressions).with(vrm))
                    .await;
            })
            .await
    }
}

/// Builds a VRM root's expression list from its load-time expression components.
///
/// Each declared expression's weight is its override if one is set, otherwise its
/// animated weight, otherwise `0.0`, clamped to `[0.0, 1.0]`. The list is sorted by name.
/// Returns an empty list when the root has no [`ExpressionSettings`].
pub(crate) fn expressions_response(
    settings: Option<&ExpressionSettings>,
    overrides: Option<&ExpressionOverrides>,
    weights: Option<&VrmExpressionWeights>,
) -> ExpressionsResponse {
    let Some(settings) = settings else {
        return ExpressionsResponse {
            expressions: Vec::new(),
        };
    };
    let mut expressions: Vec<ExpressionInfo> = settings
        .0
        .iter()
        .map(|(name, setting)| expression_info(name, setting, overrides, weights))
        .collect();
    expressions.sort_by(|a, b| a.name.cmp(&b.name));
    ExpressionsResponse { expressions }
}

fn list_expressions(
    In(vrm): In<Entity>,
    expressions: Query<(
        Option<&ExpressionSettings>,
        Option<&ExpressionOverrides>,
        Option<&VrmExpressionWeights>,
    )>,
) -> ExpressionsResponse {
    let (settings, overrides, weights) = expressions.get(vrm).unwrap_or((None, None, None));
    expressions_response(settings, overrides, weights)
}

fn expression_info(
    name: &str,
    setting: &ExpressionSetting,
    overrides: Option<&ExpressionOverrides>,
    weights: Option<&VrmExpressionWeights>,
) -> ExpressionInfo {
    let weight = overrides
        .and_then(|o| o.0.get(name))
        .or_else(|| weights.and_then(|w| w.0.get(name)))
        .copied()
        .unwrap_or(0.0);
    ExpressionInfo {
        name: name.to_string(),
        weight: weight.clamp(0.0, 1.0),
        is_binary: setting.is_binary,
        override_blink: override_type_to_string(&setting.override_blink).into(),
        override_look_at: override_type_to_string(&setting.override_look_at).into(),
        override_mouth: override_type_to_string(&setting.override_mouth).into(),
    }
}

fn override_type_to_string(ty: &ExpressionOverrideType) -> &'static str {
    match ty {
        ExpressionOverrideType::None => "none",
        ExpressionOverrideType::Block => "block",
        ExpressionOverrideType::Blend => "blend",
    }
}

fn do_set_expressions(
    In((vrm, weights)): In<(Entity, HashMap<String, f32>)>,
    mut commands: Commands,
    tx: Option<Res<VrmEventSender<ExpressionChangeEvent>>>,
) {
    let clamped: HashMap<String, f32> = weights
        .into_iter()
        .map(|(k, v)| (k, v.clamp(0.0, 1.0)))
        .collect();

    commands.trigger(SetExpressions::from_iter(
        vrm,
        clamped.iter().map(|(k, &v)| (k.as_str(), v)),
    ));

    if let Some(tx) = tx {
        tx.try_broadcast(VrmEvent {
            vrm,
            payload: ExpressionChangeEvent {
                expressions: clamped,
            },
        })
        .output_log_if_error("Failed to broadcast ExpressionChangeEvent");
    }
}

fn do_modify_expressions(
    In((vrm, weights)): In<(Entity, HashMap<String, f32>)>,
    mut commands: Commands,
    tx: Option<Res<VrmEventSender<ExpressionChangeEvent>>>,
) {
    let clamped: HashMap<String, f32> = weights
        .into_iter()
        .map(|(k, v)| (k, v.clamp(0.0, 1.0)))
        .collect();

    commands.trigger(ModifyExpressions::from_iter(
        vrm,
        clamped.iter().map(|(k, &v)| (k.as_str(), v)),
    ));

    if let Some(tx) = tx {
        tx.try_broadcast(VrmEvent {
            vrm,
            payload: ExpressionChangeEvent {
                expressions: clamped,
            },
        })
        .output_log_if_error("Failed to broadcast ExpressionChangeEvent");
    }
}

fn do_modify_mouth(
    In((vrm, weights)): In<(Entity, HashMap<String, f32>)>,
    mut commands: Commands,
    tx: Option<Res<VrmEventSender<ExpressionChangeEvent>>>,
) {
    let clamped: HashMap<String, f32> = weights
        .into_iter()
        .map(|(k, v)| (k, v.clamp(0.0, 1.0)))
        .collect();

    commands.trigger(ModifyExpressions::mouth_weights(
        vrm,
        clamped.iter().map(|(k, &v)| (k.as_str(), v)),
    ));

    if let Some(tx) = tx {
        tx.try_broadcast(VrmEvent {
            vrm,
            payload: ExpressionChangeEvent {
                expressions: clamped,
            },
        })
        .output_log_if_error("Failed to broadcast ExpressionChangeEvent");
    }
}

fn do_clear_expressions(
    In(vrm): In<Entity>,
    mut commands: Commands,
    tx: Option<Res<VrmEventSender<ExpressionChangeEvent>>>,
) {
    commands.trigger(ClearExpressions { entity: vrm });

    if let Some(tx) = tx {
        tx.try_broadcast(VrmEvent {
            vrm,
            payload: ExpressionChangeEvent {
                expressions: HashMap::new(),
            },
        })
        .output_log_if_error("Failed to broadcast ExpressionChangeEvent");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::platform::collections::HashMap;

    fn setting(is_binary: bool) -> ExpressionSetting {
        ExpressionSetting {
            is_binary,
            category: ExpressionCategory::Other,
            override_blink: ExpressionOverrideType::None,
            override_look_at: ExpressionOverrideType::None,
            override_mouth: ExpressionOverrideType::None,
        }
    }

    fn settings(names: &[&str]) -> ExpressionSettings {
        ExpressionSettings(
            names
                .iter()
                .map(|n| (n.to_string(), setting(false)))
                .collect(),
        )
    }

    fn map(entries: &[(&str, f32)]) -> HashMap<String, f32> {
        entries.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn override_takes_precedence_over_animated_weight() {
        let overrides = ExpressionOverrides(map(&[("happy", 0.8)]));
        let weights = VrmExpressionWeights(map(&[("happy", 0.3)]));
        let res = expressions_response(
            Some(&settings(&["happy"])),
            Some(&overrides),
            Some(&weights),
        );
        assert_eq!(res.expressions[0].weight, 0.8);
    }

    #[test]
    fn animated_weight_is_used_without_override() {
        let weights = VrmExpressionWeights(map(&[("happy", 0.3)]));
        let res = expressions_response(Some(&settings(&["happy"])), None, Some(&weights));
        assert_eq!(res.expressions[0].weight, 0.3);
    }

    #[test]
    fn missing_weight_defaults_to_zero() {
        let res = expressions_response(Some(&settings(&["happy"])), None, None);
        assert_eq!(res.expressions[0].weight, 0.0);
    }

    #[test]
    fn weight_is_clamped_to_unit_range() {
        let overrides = ExpressionOverrides(map(&[("a", 1.5)]));
        let weights = VrmExpressionWeights(map(&[("b", -0.2)]));
        let res = expressions_response(
            Some(&settings(&["a", "b"])),
            Some(&overrides),
            Some(&weights),
        );
        assert_eq!(res.expressions[0].weight, 1.0);
        assert_eq!(res.expressions[1].weight, 0.0);
    }

    #[test]
    fn expressions_are_sorted_by_name() {
        let res = expressions_response(Some(&settings(&["sad", "angry", "happy"])), None, None);
        let names: Vec<&str> = res.expressions.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["angry", "happy", "sad"]);
    }

    #[test]
    fn flags_are_reported_as_strings() {
        let blink = ExpressionSetting {
            is_binary: true,
            category: ExpressionCategory::Blink,
            override_blink: ExpressionOverrideType::Block,
            override_look_at: ExpressionOverrideType::Blend,
            override_mouth: ExpressionOverrideType::None,
        };
        let s = ExpressionSettings([("blink".to_string(), blink)].into_iter().collect());
        let info = &expressions_response(Some(&s), None, None).expressions[0];
        assert!(info.is_binary);
        assert_eq!(info.override_blink, "block");
        assert_eq!(info.override_look_at, "blend");
        assert_eq!(info.override_mouth, "none");
    }

    #[test]
    fn override_for_undeclared_expression_is_ignored() {
        let overrides = ExpressionOverrides(map(&[("wink", 1.0)]));
        let res = expressions_response(Some(&settings(&["happy"])), Some(&overrides), None);
        let names: Vec<&str> = res.expressions.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["happy"]);
    }

    #[test]
    fn no_settings_yields_empty_list() {
        assert!(
            expressions_response(None, None, None)
                .expressions
                .is_empty()
        );
    }
}
