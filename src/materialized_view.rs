use async_trait::async_trait;

use crate::view::{View, ViewStateComputation};

/// View State repository trait
#[async_trait]
pub trait ViewStateRepository<E, S> {
    /// The error type returned by the repository methods.
    type Error: std::error::Error + Send + Sync;
    /// Fetches state based on the event.
    async fn fetch_state(&self, event: &E) -> Result<Option<S>, Self::Error>;
    /// Saves state.
    async fn save(&self, state: &S) -> Result<S, Self::Error>;
}

pub struct MaterializedView<'a, S, E, R, Err>
where
    R: ViewStateRepository<E, S, Error = Err>,
{
    pub repository: R,
    pub view: View<'a, S, E>,
}

impl<'a, S, E, R, Err> MaterializedView<'a, S, E, R, Err>
where
    R: ViewStateRepository<E, S, Error = Err>,
{
    /// Handles the event by fetching the state from the repository, computing new state based on the current state and the event, and saving the new state to the repository.
    pub async fn handle(&self, event: &E) -> Result<S, Err> {
        let state = self.repository.fetch_state(event).await?;
        let new_state = self.view.compute_new_state(state, &[event]);
        let saved_state = self.repository.save(&new_state).await?;
        Ok(saved_state)
    }
}
