use crate::character::CharacterApi;
use crate::error::{ApiError, ApiResult};
use bevy::prelude::*;
use bevy_flurx::prelude::*;
use homunculus_core::prelude::{CharacterId, CharacterRegistry, CharacterState};

impl CharacterApi {
    /// Returns the current runtime state of the character.
    pub async fn get_state(&self, id: CharacterId) -> ApiResult<CharacterState> {
        self.0
            .schedule(move |task| async move {
                task.will(Update, once::run(get_character_state).with(id))
                    .await
            })
            .await?
    }

    /// Updates the runtime state of the character.
    ///
    /// This is transient (not persisted to the database). The ECS `Changed<CharacterState>`
    /// query handles downstream notifications automatically.
    pub async fn set_state(&self, id: CharacterId, state: CharacterState) -> ApiResult {
        self.0
            .schedule(move |task| async move {
                task.will(Update, once::run(set_character_state).with((id, state)))
                    .await
            })
            .await?
    }
}

fn get_character_state(
    In(id): In<CharacterId>,
    registry: Res<CharacterRegistry>,
    states: Query<&CharacterState>,
) -> ApiResult<CharacterState> {
    let entity = registry
        .get(&id)
        .ok_or_else(|| ApiError::CharacterNotFound(id.to_string()))?;
    let state = states
        .get(entity)
        .map_err(|_| ApiError::CharacterNotFound(id.to_string()))?;
    Ok(state.clone())
}

fn set_character_state(
    In((id, state)): In<(CharacterId, CharacterState)>,
    mut commands: Commands,
    registry: Res<CharacterRegistry>,
) -> ApiResult {
    let entity = registry
        .get(&id)
        .ok_or_else(|| ApiError::CharacterNotFound(id.to_string()))?;
    commands.entity(entity).try_insert(state);
    Ok(())
}
