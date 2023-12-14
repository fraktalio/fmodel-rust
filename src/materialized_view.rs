use std::marker::PhantomData;

use async_trait::async_trait;

use crate::view::ViewStateComputation;

/// View State Repository trait
///
/// Generic parameters:
///
/// - `E` - Event
/// - `S` - State
/// - `Error` - Error
#[async_trait]
pub trait ViewStateRepository<E, S, Error> {
    /// Fetches current state, based on the event.
    async fn fetch_state(&self, event: &E) -> Result<Option<S>, Error>;
    /// Saves the new state.
    async fn save(&self, state: &S) -> Result<S, Error>;
}

/// Materialized View.
///
/// It is using a `View` / [ViewStateComputation] to compute new state based on the current state and the event.
/// It is using a [ViewStateRepository] to fetch the current state and to save the new state.
///
/// Generic parameters:
///
/// - `S` - State
/// - `E` - Event
/// - `Repository` - View State repository
/// - `View` - View
/// - `Error` - Error
pub struct MaterializedView<S, E, Repository, View, Error>
where
    Repository: ViewStateRepository<E, S, Error>,
    View: ViewStateComputation<E, S>,
{
    repository: Repository,
    view: View,
    _marker: PhantomData<(S, E, Error)>,
}

impl<S, E, Repository, View, Error> ViewStateComputation<E, S>
    for MaterializedView<S, E, Repository, View, Error>
where
    Repository: ViewStateRepository<E, S, Error>,
    View: ViewStateComputation<E, S>,
{
    /// Computes new state based on the current state and the events.
    fn compute_new_state(&self, current_state: Option<S>, events: &[&E]) -> S {
        self.view.compute_new_state(current_state, events)
    }
}

#[async_trait]
impl<S, E, Repository, View, Error> ViewStateRepository<E, S, Error>
    for MaterializedView<S, E, Repository, View, Error>
where
    Repository: ViewStateRepository<E, S, Error> + Sync,
    View: ViewStateComputation<E, S> + Sync,
    E: Sync,
    S: Sync,
    Error: Sync,
{
    /// Fetches current state, based on the event.
    async fn fetch_state(&self, event: &E) -> Result<Option<S>, Error> {
        let state = self.repository.fetch_state(event).await?;
        Ok(state)
    }
    /// Saves the new state.
    async fn save(&self, state: &S) -> Result<S, Error> {
        self.repository.save(state).await
    }
}

impl<S, E, Repository, View, Error> MaterializedView<S, E, Repository, View, Error>
where
    Repository: ViewStateRepository<E, S, Error> + Sync,
    View: ViewStateComputation<E, S> + Sync,
    E: Sync,
    S: Sync,
    Error: Sync,
{
    /// Creates a new instance of [MaterializedView].
    pub fn new(repository: Repository, view: View) -> Self {
        MaterializedView {
            repository,
            view,
            _marker: PhantomData,
        }
    }
    /// Handles the event by fetching the state from the repository, computing new state based on the current state and the event, and saving the new state to the repository.
    pub async fn handle(&self, event: &E) -> Result<S, Error> {
        let state = self.fetch_state(event).await?;
        let new_state = self.compute_new_state(state, &[event]);
        let saved_state = self.save(&new_state).await?;
        Ok(saved_state)
    }
}
