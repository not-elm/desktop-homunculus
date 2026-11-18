//! Sound-effect playback on the dedicated SE channel.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use homunculus_core::prelude::AssetResolver;

#[derive(Resource)]
pub struct SeChannel;

#[derive(Event, Debug, Clone)]
pub struct RequestSe {
    pub source: Handle<bevy_kira_audio::AudioSource>,
    pub volume: f64,
    pub speed: f64,
    pub panning: f64,
}

/// Plays sound effects by asset ID.
#[derive(SystemParam)]
pub struct SePlayer<'w, 's> {
    commands: Commands<'w, 's>,
    resolver: AssetResolver<'w>,
}

pub(crate) struct SePlugin;

impl Plugin for SePlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_channel::<SeChannel>().add_observer(play_se);
    }
}

impl SePlayer<'_, '_> {
    /// Plays the sound effect registered under `asset_id` at volume 1.0,
    /// speed 1.0, and centered panning.
    ///
    /// When `asset_id` does not resolve to a sound, a warning is logged and
    /// nothing plays.
    pub fn play(&mut self, asset_id: &str) {
        match self.resolver.load::<bevy_kira_audio::AudioSource>(asset_id) {
            Ok(source) => self.commands.trigger(RequestSe {
                source,
                volume: 1.0,
                speed: 1.0,
                panning: 0.0,
            }),
            Err(e) => warn!("Sound effect {asset_id} is unavailable: {e}"),
        }
    }
}

pub fn volume_to_decibels(volume: f64) -> Decibels {
    if volume <= 0.0 {
        Decibels(-60.0)
    } else {
        Decibels(20.0 * (volume as f32).log10())
    }
}

fn play_se(trigger: On<RequestSe>, channel: Res<AudioChannel<SeChannel>>) {
    let event = trigger.event();
    channel
        .play(event.source.clone())
        .with_volume(volume_to_decibels(event.volume))
        .with_playback_rate(event.speed)
        .with_panning(event.panning as f32);
}
