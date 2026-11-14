use crate::prelude::GlobalViewport;
use bevy::prelude::*;
use bevy_vrm1::vrm::VrmBone;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OnDragStartEvent {
    #[serde(rename = "globalViewport")]
    pub global_viewport: GlobalViewport,
    pub bone: Option<VrmBone>,
}

impl From<(GlobalViewport, Option<VrmBone>, PointerDragStart)> for OnDragStartEvent {
    fn from(
        (global_viewport, bone, _): (GlobalViewport, Option<VrmBone>, PointerDragStart),
    ) -> Self {
        Self {
            global_viewport,
            bone,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OnDragEvent {
    pub delta: Vec2,
    #[serde(rename = "globalViewport")]
    pub global_viewport: GlobalViewport,
    pub bone: Option<VrmBone>,
}

impl From<(GlobalViewport, Option<VrmBone>, PointerDrag)> for OnDragEvent {
    fn from(
        (global_viewport, bone, event): (GlobalViewport, Option<VrmBone>, PointerDrag),
    ) -> Self {
        Self {
            delta: event.delta,
            global_viewport,
            bone,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OnDragEndEvent {
    #[serde(rename = "globalViewport")]
    pub global_viewport: GlobalViewport,
    pub bone: Option<VrmBone>,
}

impl From<(GlobalViewport, Option<VrmBone>, PointerDragEnd)> for OnDragEndEvent {
    fn from((global_viewport, bone, _): (GlobalViewport, Option<VrmBone>, PointerDragEnd)) -> Self {
        Self {
            global_viewport,
            bone,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum Button {
    Primary,
    /// The secondary pointer button
    Secondary,
    /// The tertiary pointer button
    Middle,
}

impl From<PointerButton> for Button {
    fn from(button: PointerButton) -> Self {
        match button {
            PointerButton::Primary => Button::Primary,
            PointerButton::Secondary => Button::Secondary,
            PointerButton::Middle => Button::Middle,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OnPointerPressedEvent {
    #[serde(rename = "globalViewport")]
    pub global_viewport: GlobalViewport,
    pub button: Button,
    pub bone: Option<VrmBone>,
}

impl From<(GlobalViewport, Option<VrmBone>, PointerPress)> for OnPointerPressedEvent {
    fn from(
        (global_viewport, bone, event): (GlobalViewport, Option<VrmBone>, PointerPress),
    ) -> Self {
        Self {
            global_viewport,
            button: event.button.into(),
            bone,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OnPointerMoveEvent {
    #[serde(rename = "globalViewport")]
    pub global_viewport: GlobalViewport,
    pub delta: Vec2,
    pub bone: Option<VrmBone>,
}

impl From<(GlobalViewport, Option<VrmBone>, PointerMove)> for OnPointerMoveEvent {
    fn from(
        (global_viewport, bone, event): (GlobalViewport, Option<VrmBone>, PointerMove),
    ) -> Self {
        Self {
            global_viewport,
            delta: event.delta,
            bone,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OnPointerReleasedEvent {
    #[serde(rename = "globalViewport")]
    pub global_viewport: GlobalViewport,
    pub button: Button,
    pub bone: Option<VrmBone>,
}

impl From<(GlobalViewport, Option<VrmBone>, PointerRelease)> for OnPointerReleasedEvent {
    fn from(
        (global_viewport, bone, event): (GlobalViewport, Option<VrmBone>, PointerRelease),
    ) -> Self {
        Self {
            global_viewport,
            button: event.button.into(),
            bone,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OnPointerOverEvent {
    #[serde(rename = "globalViewport")]
    pub global_viewport: GlobalViewport,
    pub bone: Option<VrmBone>,
}

impl From<(GlobalViewport, Option<VrmBone>, PointerOver)> for OnPointerOverEvent {
    fn from((global_viewport, bone, _): (GlobalViewport, Option<VrmBone>, PointerOver)) -> Self {
        Self {
            global_viewport,
            bone,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OnPointerOutEvent {
    #[serde(rename = "globalViewport")]
    pub global_viewport: GlobalViewport,
    pub bone: Option<VrmBone>,
}

impl From<(GlobalViewport, Option<VrmBone>, PointerOut)> for OnPointerOutEvent {
    fn from((global_viewport, bone, _): (GlobalViewport, Option<VrmBone>, PointerOut)) -> Self {
        Self {
            global_viewport,
            bone,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OnPointerCancelEvent {
    #[serde(rename = "globalViewport")]
    pub global_viewport: GlobalViewport,
    pub bone: Option<VrmBone>,
}

impl From<(GlobalViewport, Option<VrmBone>, PointerCancel)> for OnPointerCancelEvent {
    fn from((global_viewport, bone, _): (GlobalViewport, Option<VrmBone>, PointerCancel)) -> Self {
        Self {
            global_viewport,
            bone,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OnClickEvent {
    #[serde(rename = "globalViewport")]
    pub global_viewport: GlobalViewport,
    pub button: Button,
    pub bone: Option<VrmBone>,
}

impl From<(GlobalViewport, Option<VrmBone>, PointerClick)> for OnClickEvent {
    fn from(
        (global_viewport, bone, event): (GlobalViewport, Option<VrmBone>, PointerClick),
    ) -> Self {
        Self {
            global_viewport,
            button: event.button.into(),
            bone,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ExpressionChangeEvent {
    pub expressions: HashMap<String, f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct VrmaPlayEvent {
    pub vrma: u64,
    pub name: String,
    pub repeat: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct VrmaFinishEvent {
    pub vrma: u64,
    pub name: String,
}
