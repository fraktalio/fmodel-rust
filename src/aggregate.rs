use async_trait::async_trait;

use crate::decider::{Decider, EventComputation, StateComputation};

/// Event repository trait
#[async_trait]
pub trait EventRepository<C, E> {
    /// The error type returned by the repository methods.
    type Error: std::error::Error + Send + Sync;
    /// Version of the event stream / Highest sequence number of the event stream.
    type Version: Send + Sync;

    /// Fetches events based on the command.
    async fn fetch_events(&self, command: &C) -> Result<Vec<(E, Self::Version)>, Self::Error>;

    /// Saves events.
    async fn save(
        &self,
        events: &[E],
        latest_version: &Option<Self::Version>,
    ) -> Result<Vec<(E, Self::Version)>, Self::Error>;
}

/// Event sourced aggregate.
///
/// It is using a [Decider] to compute new events based on the current events and the command.
/// It is using a [EventRepository] to fetch the current events and to save the new events.
pub struct EventSourcedAggregate<'a, C, S, E, R, V, Err>
where
    R: EventRepository<C, E, Error = Err, Version = V>,
{
    pub repository: R,
    pub decider: Decider<'a, C, S, E>,
}

impl<'a, C, S, E, R, V, Err> EventSourcedAggregate<'a, C, S, E, R, V, Err>
where
    R: EventRepository<C, E, Error = Err, Version = V>,
{
    /// Handles the command by fetching the events from the repository, computing new events based on the current events and the command, and saving the new events to the repository.
    pub async fn handle(&self, command: &C) -> Result<Vec<(E, V)>, Err> {
        let events: Vec<(E, V)> = self.repository.fetch_events(command).await?;
        let mut version: Option<V> = None;
        let mut current_events: Vec<E> = vec![];
        for (event, ver) in events {
            version = Some(ver);
            current_events.push(event);
        }
        let new_events = self.decider.compute_new_events(&current_events, command);
        let saved_events = self.repository.save(&new_events, &version).await?;
        Ok(saved_events)
    }
}

/// State repository trait
#[async_trait]
pub trait StateRepository<C, S> {
    /// The error type returned by the repository methods.
    type Error: std::error::Error + Send + Sync;
    /// Version of the state.
    type Version: Send + Sync;
    /// Fetches state based on the command.
    async fn fetch_state(&self, command: &C) -> Result<Option<(S, Self::Version)>, Self::Error>;
    /// Saves state.
    async fn save(
        &self,
        state: &S,
        version: &Option<Self::Version>,
    ) -> Result<(S, Self::Version), Self::Error>;
}

/// State stored aggregate.
///
/// It is using a [Decider] to compute new state based on the current state and the command.
/// It is using a [StateRepository] to fetch the current state and to save the new state.
pub struct StateStoredAggregate<'a, C, S, E, R, V, Err>
where
    R: StateRepository<C, S, Error = Err, Version = V>,
{
    pub repository: R,
    pub decider: Decider<'a, C, S, E>,
}

impl<'a, C, S, E, R, V, Err> StateStoredAggregate<'a, C, S, E, R, V, Err>
where
    R: StateRepository<C, S, Error = Err, Version = V>,
{
    /// Handles the command by fetching the state from the repository, computing new state based on the current state and the command, and saving the new state to the repository.
    pub async fn handle(&self, command: &C) -> Result<(S, V), Err> {
        let state_version = self.repository.fetch_state(command).await?;
        match state_version {
            None => {
                let new_state = self.decider.compute_new_state(None, command);
                let saved_state = self.repository.save(&new_state, &None).await?;
                Ok(saved_state)
            }
            Some((state, version)) => {
                let new_state = self.decider.compute_new_state(Some(state), command);
                let saved_state = self.repository.save(&new_state, &Some(version)).await?;
                Ok(saved_state)
            }
        }
    }
}
