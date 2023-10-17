use std::marker::PhantomData;

use async_trait::async_trait;

use crate::decider::{EventComputation, StateComputation};

/// Event Repository trait
///
/// Generic parameters:
///
/// - `C` - Command
/// - `E` - Event
/// - `Version` - Version/Offset/Sequence number
/// - `Error` - Error
#[async_trait]
pub trait EventRepository<C, E, Version, Error> {
    /// Fetches current events, based on the command.
    async fn fetch_events(&self, command: &C) -> Result<Vec<(E, Version)>, Error>;
    /// Saves events.
    async fn save(
        &self,
        events: &[E],
        latest_version: &Option<Version>,
    ) -> Result<Vec<(E, Version)>, Error>;
}

/// Event Sourced Aggregate.
///
/// It is using a `Decider` / [EventComputation] to compute new events based on the current events and the command.
/// It is using a [EventRepository] to fetch the current events and to save the new events.
///
/// Generic parameters:
///
/// - `C` - Command
/// - `S` - State
/// - `E` - Event
/// - `Repository` - Event repository
/// - `Decider` - Event computation
/// - `Version` - Version/Offset/Sequence number
/// - `Error` - Error
pub struct EventSourcedAggregate<C, S, E, Repository, Decider, Version, Error>
where
    Repository: EventRepository<C, E, Version, Error>,
    Decider: EventComputation<C, S, E>,
{
    repository: Repository,
    decider: Decider,
    _marker: PhantomData<(C, S, E, Version, Error)>,
}

impl<C, S, E, Repository, Decider, Version, Error>
    EventSourcedAggregate<C, S, E, Repository, Decider, Version, Error>
where
    Repository: EventRepository<C, E, Version, Error>,
    Decider: EventComputation<C, S, E>,
{
    /// Creates a new instance of [EventSourcedAggregate].
    pub fn new(repository: Repository, decider: Decider) -> Self {
        EventSourcedAggregate {
            repository,
            decider,
            _marker: PhantomData,
        }
    }
    /// Handles the command by fetching the events from the repository, computing new events based on the current events and the command, and saving the new events to the repository.
    pub async fn handle(&self, command: &C) -> Result<Vec<(E, Version)>, Error> {
        let events: Vec<(E, Version)> = self.repository.fetch_events(command).await?;
        let mut version: Option<Version> = None;
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

/// State Repository trait
///
/// Generic parameters:
///
/// - `C` - Command
/// - `S` - State
/// - `Version` - Version
/// - `Error` - Error
#[async_trait]
pub trait StateRepository<C, S, Version, Error> {
    /// Fetches current state, based on the command.
    async fn fetch_state(&self, command: &C) -> Result<Option<(S, Version)>, Error>;
    /// Saves state.
    async fn save(&self, state: &S, version: &Option<Version>) -> Result<(S, Version), Error>;
}

/// State Stored Aggregate.
///
/// It is using a `Decider` / [StateComputation] to compute new state based on the current state and the command.
/// It is using a [StateRepository] to fetch the current state and to save the new state.
///
/// Generic parameters:
///
/// - `C` - Command
/// - `S` - State
/// - `E` - Event
/// - `Repository` - State repository
/// - `Decider` - State computation
/// - `Version` - Version
/// - `Error` - Error
pub struct StateStoredAggregate<C, S, E, Repository, Decider, Version, Error>
where
    Repository: StateRepository<C, S, Version, Error>,
    Decider: StateComputation<C, S, E>,
{
    repository: Repository,
    decider: Decider,
    _marker: PhantomData<(C, S, E, Version, Error)>,
}

impl<C, S, E, Repository, Decider, Version, Error>
    StateStoredAggregate<C, S, E, Repository, Decider, Version, Error>
where
    Repository: StateRepository<C, S, Version, Error>,
    Decider: StateComputation<C, S, E>,
{
    /// Creates a new instance of [StateStoredAggregate].
    pub fn new(repository: Repository, decider: Decider) -> Self {
        StateStoredAggregate {
            repository,
            decider,
            _marker: PhantomData,
        }
    }
    /// Handles the command by fetching the state from the repository, computing new state based on the current state and the command, and saving the new state to the repository.
    pub async fn handle(&self, command: &C) -> Result<(S, Version), Error> {
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
