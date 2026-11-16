//! When the application exits, it automatically saves each VRM's Transform
//! using the asset ID as the preferences key (e.g., `elmer:vrm:transform`).
//! VRM transforms are also periodically saved to protect against unexpected
//! termination (SIGKILL, Force Quit, power loss).

use crate::{PrefsDatabase, PrefsKeys};
use bevy::app::OnAppExitSystems;
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_vrm1::vrm::Vrm;
use homunculus_core::prelude::AssetIdComponent;
use std::time::Duration;

/// Interval in seconds between periodic VRM transform saves.
pub(super) struct PrefsVrmTransformPlugin;

impl Plugin for PrefsVrmTransformPlugin {
    fn build(&self, app: &mut App) {
        // NOTE: The winit runner stops right after the frame in which `AppExit` is
        // written and never runs another update, so this save must stay in `Last`
        // under `OnAppExitSystems` to see an exit requested anywhere in that frame.
        app.add_systems(
            Last,
            save_vrm_transforms
                .run_if(on_message::<AppExit>)
                .in_set(OnAppExitSystems),
        );
        app.add_systems(
            Update,
            periodic_save_vrm_transforms.run_if(on_timer(Duration::from_secs(5))),
        );
    }
}

fn save_vrm_transforms(
    db: NonSend<PrefsDatabase>,
    transforms: Query<(&AssetIdComponent, &Transform), With<Vrm>>,
) {
    info!("Saving VRM transforms to preferences...");
    for (asset_id, transform) in transforms.iter() {
        let key = PrefsKeys::asset_transform(asset_id.0.as_ref());
        let _ = db.save_as(&key, transform);
    }
}

fn periodic_save_vrm_transforms(
    db: NonSend<PrefsDatabase>,
    transforms: Query<(&AssetIdComponent, &Transform), With<Vrm>>,
) {
    for (asset_id, transform) in transforms.iter() {
        let key = PrefsKeys::asset_transform(asset_id.0.as_ref());
        let _ = db.save_as(&key, transform);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::OnAppExitSystems;
    use homunculus_core::prelude::AssetId;

    fn write_app_exit(mut app_exit: MessageWriter<AppExit>) {
        app_exit.write(AppExit::Success);
    }

    fn app_with_vrm(transform: Transform) -> (App, AssetId) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_non_send(PrefsDatabase::open_in_memory())
            .add_plugins(PrefsVrmTransformPlugin);
        let asset_id = AssetId::new("elmer:vrm");
        app.world_mut()
            .spawn((AssetIdComponent(asset_id.clone()), transform, Vrm));
        (app, asset_id)
    }

    fn saved_transform(app: &App, asset_id: &AssetId) -> Option<Transform> {
        let key = PrefsKeys::asset_transform(asset_id.as_ref());
        app.world()
            .non_send::<PrefsDatabase>()
            .load_as::<Transform>(&key)
            .expect("loading transform should not fail")
    }

    /// Asserts that an exit requested in `PostUpdate` saves the VRM transform
    /// before the same frame ends.
    ///
    /// Case: the user clicks Exit in the tray menu, the click is handled in
    /// `PostUpdate`, and the runner shuts down right after that frame.
    #[test]
    fn saves_vrm_transform_when_exit_is_requested_in_post_update() {
        let expected = Transform::from_xyz(42.0, -3.0, 1.5);
        let (mut app, asset_id) = app_with_vrm(expected);
        app.add_systems(PostUpdate, write_app_exit);
        app.update();
        assert_eq!(saved_transform(&app, &asset_id), Some(expected));
    }

    /// Asserts that an exit requested in `Last` before `OnAppExitSystems`
    /// still saves the VRM transform before the same frame ends.
    ///
    /// Case: the user closes every window, and Bevy requests the exit at the
    /// end of the frame.
    #[test]
    fn saves_vrm_transform_when_exit_is_requested_in_last() {
        let expected = Transform::from_xyz(-7.0, 2.0, 0.5);
        let (mut app, asset_id) = app_with_vrm(expected);
        app.add_systems(Last, write_app_exit.before(OnAppExitSystems));
        app.update();
        assert_eq!(saved_transform(&app, &asset_id), Some(expected));
    }
}
