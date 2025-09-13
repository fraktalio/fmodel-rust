use std::future::Future;
use std::marker::PhantomData;

use crate::decider::{Decider, EventComputation, StateComputation};
use crate::saga::{ActionComputation, Saga};
use crate::Identifier;

/// Event Repository trait
///
/// Generic parameters:
///
/// - `C` - Command
/// - `E` - Event
/// - `Version` - Version/Offset/Sequence number
/// - `Error` - Error
#[cfg(not(feature = "not-send-futures"))]
pub trait EventRepository<C, E, Version, Error> {
    /// Fetches current events, based on the command.
    /// Desugared `async fn fetch_events(&self, command: &C) -> Result<Vec<(E, Version)>, Error>;` to a normal `fn` that returns `impl Future`, and adds bound `Send`.
    /// You can freely move between the `async fn` and `-> impl Future` spelling in your traits and impls. This is true even when one form has a Send bound.
    fn fetch_events(
        &self,
        command: &C,
    ) -> impl Future<Output = Result<Vec<(E, Version)>, Error>> + Send;
    /// Saves events.
    /// Desugared `async fn save(&self, events: &[E], latest_version: &Option<Version>) -> Result<Vec<(E, Version)>, Error>;` to a normal `fn` that returns `impl Future`, and adds bound `Send`
    /// You can freely move between the `async fn` and `-> impl Future` spelling in your traits and impls. This is true even when one form has a Send bound.
    fn save(&self, events: &[E]) -> impl Future<Output = Result<Vec<(E, Version)>, Error>> + Send;

    /// Version provider. It is used to provide the version/sequence of the stream to wich this event belongs to. Optimistic locking is useing this version to check if the event is already saved.
    /// Desugared `async fn version_provider(&self, event: &E) -> Result<Option<Version>, Error>;` to a normal `fn` that returns `impl Future`, and adds bound `Send`
    /// You can freely move between the `async fn` and `-> impl Future` spelling in your traits and impls. This is true even when one form has a Send bound.
    fn version_provider(
        &self,
        event: &E,
    ) -> impl Future<Output = Result<Option<Version>, Error>> + Send;
}

/// Event Repository trait
///
/// Generic parameters:
///
/// - `C` - Command
/// - `E` - Event
/// - `Version` - Version/Offset/Sequence number
/// - `Error` - Error
#[cfg(feature = "not-send-futures")]
pub trait EventRepository<C, E, Version, Error> {
    /// Fetches current events, based on the command.
    /// Desugared `async fn fetch_events(&self, command: &C) -> Result<Vec<(E, Version)>, Error>;` to a normal `fn` that returns `impl Future`.
    /// You can freely move between the `async fn` and `-> impl Future` spelling in your traits and impls.
    fn fetch_events(&self, command: &C) -> impl Future<Output = Result<Vec<(E, Version)>, Error>>;
    /// Saves events.
    /// Desugared `async fn save(&self, events: &[E], latest_version: &Option<Version>) -> Result<Vec<(E, Version)>, Error>;` to a normal `fn` that returns `impl Future`
    /// You can freely move between the `async fn` and `-> impl Future` spelling in your traits and impls.
    fn save(&self, events: &[E]) -> impl Future<Output = Result<Vec<(E, Version)>, Error>>;

    /// Version provider. It is used to provide the version/sequence of the stream to wich this event belongs to. Optimistic locking is useing this version to check if the event is already saved.
    /// Desugared `async fn version_provider(&self, event: &E) -> Result<Option<Version>, Error>;` to a normal `fn` that returns `impl Future`
    /// You can freely move between the `async fn` and `-> impl Future` spelling in your traits and impls.
    fn version_provider(&self, event: &E) -> impl Future<Output = Result<Option<Version>, Error>>;
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
    Decider: EventComputation<C, S, E, Error>,
{
    repository: Repository,
    decider: Decider,
    _marker: PhantomData<(C, S, E, Version, Error)>,
}

impl<C, S, E, Repository, Decider, Version, Error> EventComputation<C, S, E, Error>
    for EventSourcedAggregate<C, S, E, Repository, Decider, Version, Error>
where
    Repository: EventRepository<C, E, Version, Error>,
    Decider: EventComputation<C, S, E, Error>,
{
    /// Computes new events based on the current events and the command.
    fn compute_new_events(&self, current_events: &[E], command: &C) -> Result<Vec<E>, Error> {
        self.decider.compute_new_events(current_events, command)
    }
}

#[cfg(not(feature = "not-send-futures"))]
impl<C, S, E, Repository, Decider, Version, Error> EventRepository<C, E, Version, Error>
    for EventSourcedAggregate<C, S, E, Repository, Decider, Version, Error>
where
    Repository: EventRepository<C, E, Version, Error> + Sync,
    Decider: EventComputation<C, S, E, Error> + Sync,
    C: Sync,
    S: Sync,
    E: Sync,
    Version: Sync,
    Error: Sync,
{
    /// Fetches current events, based on the command.
    async fn fetch_events(&self, command: &C) -> Result<Vec<(E, Version)>, Error> {
        self.repository.fetch_events(command).await
    }
    /// Saves events.
    async fn save(&self, events: &[E]) -> Result<Vec<(E, Version)>, Error> {
        self.repository.save(events).await
    }
    /// Version provider. It is used to provide the version/sequence of the event. Optimistic locking is useing this version to check if the event is already saved.
    async fn version_provider(&self, event: &E) -> Result<Option<Version>, Error> {
        self.repository.version_provider(event).await
    }
}

#[cfg(feature = "not-send-futures")]
impl<C, S, E, Repository, Decider, Version, Error> EventRepository<C, E, Version, Error>
    for EventSourcedAggregate<C, S, E, Repository, Decider, Version, Error>
where
    Repository: EventRepository<C, E, Version, Error>,
    Decider: EventComputation<C, S, E, Error>,
{
    /// Fetches current events, based on the command.
    async fn fetch_events(&self, command: &C) -> Result<Vec<(E, Version)>, Error> {
        self.repository.fetch_events(command).await
    }
    /// Saves events.
    async fn save(&self, events: &[E]) -> Result<Vec<(E, Version)>, Error> {
        self.repository.save(events).await
    }
    /// Version provider. It is used to provide the version/sequence of the event. Optimistic locking is useing this version to check if the event is already saved.
    async fn version_provider(&self, event: &E) -> Result<Option<Version>, Error> {
        self.repository.version_provider(event).await
    }
}

#[cfg(not(feature = "not-send-futures"))]
impl<C, S, E, Repository, Decider, Version, Error>
    EventSourcedAggregate<C, S, E, Repository, Decider, Version, Error>
where
    Repository: EventRepository<C, E, Version, Error> + Sync,
    Decider: EventComputation<C, S, E, Error> + Sync,
    C: Sync,
    S: Sync,
    E: Sync,
    Version: Sync,
    Error: Sync,
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
        let events: Vec<(E, Version)> = self.fetch_events(command).await?;
        let mut current_events: Vec<E> = vec![];
        for (event, _) in events {
            current_events.push(event);
        }
        let new_events = self.compute_new_events(&current_events, command)?;
        let saved_events = self.save(&new_events).await?;
        Ok(saved_events)
    }
}

#[cfg(feature = "not-send-futures")]
impl<C, S, E, Repository, Decider, Version, Error>
    EventSourcedAggregate<C, S, E, Repository, Decider, Version, Error>
where
    Repository: EventRepository<C, E, Version, Error>,
    Decider: EventComputation<C, S, E, Error>,
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
        let events: Vec<(E, Version)> = self.fetch_events(command).await?;
        let mut current_events: Vec<E> = vec![];
        for (event, _) in events {
            current_events.push(event);
        }
        let new_events = self.compute_new_events(&current_events, command)?;
        let saved_events = self.save(&new_events).await?;
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
#[cfg(not(feature = "not-send-futures"))]
pub trait StateRepository<C, S, Version, Error> {
    /// Fetches current state, based on the command.
    /// Desugared `async fn fetch_state(&self, command: &C) -> Result<Option<(S, Version)>, Error>;` to a normal `fn` that returns `impl Future` and adds bound `Send`
    /// You can freely move between the `async fn` and `-> impl Future` spelling in your traits and impls. This is true even when one form has a Send bound.
    fn fetch_state(
        &self,
        command: &C,
    ) -> impl Future<Output = Result<Option<(S, Version)>, Error>> + Send;
    /// Saves state.
    /// Desugared `async fn save(&self, state: &S, version: &Option<Version>) -> Result<(S, Version), Error>;` to a normal `fn` that returns `impl Future` and adds bound `Send`
    /// You can freely move between the `async fn` and `-> impl Future` spelling in your traits and impls. This is true even when one form has a Send bound.
    fn save(
        &self,
        state: &S,
        version: &Option<Version>,
    ) -> impl Future<Output = Result<(S, Version), Error>> + Send;
}

/// State Repository trait
///
/// Generic parameters:
///
/// - `C` - Command
/// - `S` - State
/// - `Version` - Version
/// - `Error` - Error
#[cfg(feature = "not-send-futures")]
pub trait StateRepository<C, S, Version, Error> {
    /// Fetches current state, based on the command.
    /// Desugared `async fn fetch_state(&self, command: &C) -> Result<Option<(S, Version)>, Error>;` to a normal `fn` that returns `impl Future`
    /// You can freely move between the `async fn` and `-> impl Future` spelling in your traits and impls.
    fn fetch_state(&self, command: &C)
        -> impl Future<Output = Result<Option<(S, Version)>, Error>>;
    /// Saves state.
    /// Desugared `async fn save(&self, state: &S, version: &Option<Version>) -> Result<(S, Version), Error>;` to a normal `fn` that returns `impl Future`
    /// You can freely move between the `async fn` and `-> impl Future` spelling in your traits and impls.
    fn save(
        &self,
        state: &S,
        version: &Option<Version>,
    ) -> impl Future<Output = Result<(S, Version), Error>>;
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
    Decider: StateComputation<C, S, E, Error>,
{
    repository: Repository,
    decider: Decider,
    _marker: PhantomData<(C, S, E, Version, Error)>,
}

impl<C, S, E, Repository, Decider, Version, Error> StateComputation<C, S, E, Error>
    for StateStoredAggregate<C, S, E, Repository, Decider, Version, Error>
where
    Repository: StateRepository<C, S, Version, Error>,
    Decider: StateComputation<C, S, E, Error>,
{
    /// Computes new state based on the current state and the command.
    fn compute_new_state(&self, current_state: Option<S>, command: &C) -> Result<S, Error> {
        self.decider.compute_new_state(current_state, command)
    }
}

#[cfg(not(feature = "not-send-futures"))]
impl<C, S, E, Repository, Decider, Version, Error> StateRepository<C, S, Version, Error>
    for StateStoredAggregate<C, S, E, Repository, Decider, Version, Error>
where
    Repository: StateRepository<C, S, Version, Error> + Sync,
    Decider: StateComputation<C, S, E, Error> + Sync,
    C: Sync,
    S: Sync,
    E: Sync,
    Version: Sync,
    Error: Sync,
{
    /// Fetches current state, based on the command.
    async fn fetch_state(&self, command: &C) -> Result<Option<(S, Version)>, Error> {
        self.repository.fetch_state(command).await
    }
    /// Saves state.
    async fn save(&self, state: &S, version: &Option<Version>) -> Result<(S, Version), Error> {
        self.repository.save(state, version).await
    }
}

#[cfg(feature = "not-send-futures")]
impl<C, S, E, Repository, Decider, Version, Error> StateRepository<C, S, Version, Error>
    for StateStoredAggregate<C, S, E, Repository, Decider, Version, Error>
where
    Repository: StateRepository<C, S, Version, Error>,
    Decider: StateComputation<C, S, E, Error>,
{
    /// Fetches current state, based on the command.
    async fn fetch_state(&self, command: &C) -> Result<Option<(S, Version)>, Error> {
        self.repository.fetch_state(command).await
    }
    /// Saves state.
    async fn save(&self, state: &S, version: &Option<Version>) -> Result<(S, Version), Error> {
        self.repository.save(state, version).await
    }
}

#[cfg(not(feature = "not-send-futures"))]
impl<C, S, E, Repository, Decider, Version, Error>
    StateStoredAggregate<C, S, E, Repository, Decider, Version, Error>
where
    Repository: StateRepository<C, S, Version, Error> + Sync,
    Decider: StateComputation<C, S, E, Error> + Sync,
    C: Sync,
    S: Sync,
    E: Sync,
    Version: Sync,
    Error: Sync,
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
        let state_version = self.fetch_state(command).await?;
        match state_version {
            None => {
                let new_state = self.compute_new_state(None, command)?;
                let saved_state = self.save(&new_state, &None).await?;
                Ok(saved_state)
            }
            Some((state, version)) => {
                let new_state = self.compute_new_state(Some(state), command)?;
                let saved_state = self.save(&new_state, &Some(version)).await?;
                Ok(saved_state)
            }
        }
    }
}

#[cfg(feature = "not-send-futures")]
impl<C, S, E, Repository, Decider, Version, Error>
    StateStoredAggregate<C, S, E, Repository, Decider, Version, Error>
where
    Repository: StateRepository<C, S, Version, Error>,
    Decider: StateComputation<C, S, E, Error>,
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
        let state_version = self.fetch_state(command).await?;
        match state_version {
            None => {
                let new_state = self.compute_new_state(None, command)?;
                let saved_state = self.save(&new_state, &None).await?;
                Ok(saved_state)
            }
            Some((state, version)) => {
                let new_state = self.compute_new_state(Some(state), command)?;
                let saved_state = self.save(&new_state, &Some(version)).await?;
                Ok(saved_state)
            }
        }
    }
}

/// Orchestrating Event Sourced Aggregate.
/// It is using a [Decider] and [Saga] to compute new events based on the current events and the command.
/// If the `decider` is combined out of many deciders via `combine` function, a `saga` could be used to react on new events and send new commands to the `decider` recursively, in single transaction.
/// It is using a [EventRepository] to fetch the current events and to save the new events.
/// Generic parameters:
/// - `C` - Command
/// - `S` - State
/// - `E` - Event
/// - `Repository` - Event repository
/// - `Version` - Version/Offset/Sequence number
/// - `Error` - Error
pub struct EventSourcedOrchestratingAggregate<'a, C, S, E, Repository, Version, Error>
where
    Repository: EventRepository<C, E, Version, Error>,
{
    repository: Repository,
    decider: Decider<'a, C, S, E, Error>,
    saga: Saga<'a, E, C>,
    _marker: PhantomData<(C, S, E, Version, Error)>,
}

#[cfg(not(feature = "not-send-futures"))]
impl<C, S, E, Repository, Version, Error> EventRepository<C, E, Version, Error>
    for EventSourcedOrchestratingAggregate<'_, C, S, E, Repository, Version, Error>
where
    Repository: EventRepository<C, E, Version, Error> + Sync,
    C: Sync,
    S: Sync,
    E: Sync,
    Version: Sync,
    Error: Sync,
{
    /// Fetches current events, based on the command.
    async fn fetch_events(&self, command: &C) -> Result<Vec<(E, Version)>, Error> {
        self.repository.fetch_events(command).await
    }
    /// Saves events.
    async fn save(&self, events: &[E]) -> Result<Vec<(E, Version)>, Error> {
        self.repository.save(events).await
    }
    /// Version provider. It is used to provide the version/sequence of the event. Optimistic locking is useing this version to check if the event is already saved.
    async fn version_provider(&self, event: &E) -> Result<Option<Version>, Error> {
        self.repository.version_provider(event).await
    }
}

#[cfg(feature = "not-send-futures")]
impl<C, S, E, Repository, Version, Error> EventRepository<C, E, Version, Error>
    for EventSourcedOrchestratingAggregate<'_, C, S, E, Repository, Version, Error>
where
    Repository: EventRepository<C, E, Version, Error>,
{
    /// Fetches current events, based on the command.
    async fn fetch_events(&self, command: &C) -> Result<Vec<(E, Version)>, Error> {
        self.repository.fetch_events(command).await
    }
    /// Saves events.
    async fn save(&self, events: &[E]) -> Result<Vec<(E, Version)>, Error> {
        self.repository.save(events).await
    }
    /// Version provider. It is used to provide the version/sequence of the event. Optimistic locking is useing this version to check if the event is already saved.
    async fn version_provider(&self, event: &E) -> Result<Option<Version>, Error> {
        self.repository.version_provider(event).await
    }
}

#[cfg(not(feature = "not-send-futures"))]
impl<'a, C, S, E, Repository, Version, Error>
    EventSourcedOrchestratingAggregate<'a, C, S, E, Repository, Version, Error>
where
    Repository: EventRepository<C, E, Version, Error> + Sync,
    C: Sync,
    S: Sync,
    E: Sync + Clone,
    Version: Sync,
    Error: Sync,
{
    /// Creates a new instance of [EventSourcedAggregate].
    pub fn new(
        repository: Repository,
        decider: Decider<'a, C, S, E, Error>,
        saga: Saga<'a, E, C>,
    ) -> Self {
        EventSourcedOrchestratingAggregate {
            repository,
            decider,
            saga,
            _marker: PhantomData,
        }
    }
    /// Handles the command by fetching the events from the repository, computing new events based on the current events and the command, and saving the new events to the repository.
    pub async fn handle(&self, command: &C) -> Result<Vec<(E, Version)>, Error>
    where
        E: Identifier,
        C: Identifier,
    {
        let events: Vec<(E, Version)> = self.fetch_events(command).await?;
        let mut current_events: Vec<E> = vec![];
        for (event, _) in events {
            current_events.push(event);
        }
        let new_events = self
            .compute_new_events_dynamically(&current_events, command)
            .await?;
        let saved_events = self.save(&new_events).await?;
        Ok(saved_events)
    }
    /// Computes new events based on the current events and the command.
    /// It is using a [Decider] and [Saga] to compute new events based on the current events and the command.
    /// If the `decider` is combined out of many deciders via `combine` function, a `saga` could be used to react on new events and send new commands to the `decider` recursively, in single transaction.
    /// It is using a [EventRepository] to fetch the current events for the command that is computed by the `saga`.
    async fn compute_new_events_dynamically(
        &self,
        current_events: &[E],
        command: &C,
    ) -> Result<Vec<E>, Error>
    where
        E: Identifier,
        C: Identifier,
    {
        let current_state: S = current_events
            .iter()
            .fold((self.decider.initial_state)(), |state, event| {
                (self.decider.evolve)(&state, event)
            });

        let initial_events = (self.decider.decide)(command, &current_state)?;

        let commands: Vec<C> = initial_events
            .iter()
            .flat_map(|event: &E| self.saga.compute_new_actions(event))
            .collect();

        // Collect all events including recursively computed new events.
        let mut all_events = initial_events.clone();

        for command in commands.iter() {
            let previous_events = [
                self.repository
                    .fetch_events(command)
                    .await?
                    .iter()
                    .map(|(e, _)| e.clone())
                    .collect::<Vec<E>>(),
                initial_events
                    .clone()
                    .into_iter()
                    .filter(|e| e.identifier() == command.identifier())
                    .collect::<Vec<E>>(),
            ]
            .concat();

            // Recursively compute new events and extend the accumulated events list.
            // By wrapping the recursive call in a Box, we ensure that the future type is not self-referential.
            let new_events =
                Box::pin(self.compute_new_events_dynamically(&previous_events, command)).await?;
            all_events.extend(new_events);
        }

        Ok(all_events)
    }
}

#[cfg(feature = "not-send-futures")]
impl<'a, C, S, E, Repository, Version, Error>
    EventSourcedOrchestratingAggregate<'a, C, S, E, Repository, Version, Error>
where
    Repository: EventRepository<C, E, Version, Error>,
    E: Clone,
{
    /// Creates a new instance of [EventSourcedAggregate].
    pub fn new(
        repository: Repository,
        decider: Decider<'a, C, S, E, Error>,
        saga: Saga<'a, E, C>,
    ) -> Self {
        EventSourcedOrchestratingAggregate {
            repository,
            decider,
            saga,
            _marker: PhantomData,
        }
    }
    /// Handles the command by fetching the events from the repository, computing new events based on the current events and the command, and saving the new events to the repository.
    pub async fn handle(&self, command: &C) -> Result<Vec<(E, Version)>, Error>
    where
        E: Identifier,
        C: Identifier,
    {
        let events: Vec<(E, Version)> = self.fetch_events(command).await?;
        let mut current_events: Vec<E> = vec![];
        for (event, _) in events {
            current_events.push(event);
        }
        let new_events = self
            .compute_new_events_dynamically(&current_events, command)
            .await?;
        let saved_events = self.save(&new_events).await?;
        Ok(saved_events)
    }
    /// Computes new events based on the current events and the command.
    /// It is using a [Decider] and [Saga] to compute new events based on the current events and the command.
    /// If the `decider` is combined out of many deciders via `combine` function, a `saga` could be used to react on new events and send new commands to the `decider` recursively, in single transaction.
    /// It is using a [EventRepository] to fetch the current events for the command that is computed by the `saga`.
    async fn compute_new_events_dynamically(
        &self,
        current_events: &[E],
        command: &C,
    ) -> Result<Vec<E>, Error>
    where
        E: Identifier,
        C: Identifier,
    {
        let current_state: S = current_events
            .iter()
            .fold((self.decider.initial_state)(), |state, event| {
                (self.decider.evolve)(&state, event)
            });

        let initial_events = (self.decider.decide)(command, &current_state)?;

        let commands: Vec<C> = initial_events
            .iter()
            .flat_map(|event: &E| self.saga.compute_new_actions(event))
            .collect();

        // Collect all events including recursively computed new events.
        let mut all_events = initial_events.clone();

        for command in commands.iter() {
            let previous_events = [
                self.repository
                    .fetch_events(command)
                    .await?
                    .iter()
                    .map(|(e, _)| e.clone())
                    .collect::<Vec<E>>(),
                initial_events
                    .clone()
                    .into_iter()
                    .filter(|e| e.identifier() == command.identifier())
                    .collect::<Vec<E>>(),
            ]
            .concat();

            // Recursively compute new events and extend the accumulated events list.
            // By wrapping the recursive call in a Box, we ensure that the future type is not self-referential.
            let new_events =
                Box::pin(self.compute_new_events_dynamically(&previous_events, command)).await?;
            all_events.extend(new_events);
        }

        Ok(all_events)
    }
}

/// Orchestrating State Stored Aggregate.
///
/// It is using a [Decider] and [Saga] to compute new state based on the current state and the command.
/// If the `decider` is combined out of many deciders via `combine` function, a `saga` could be used to react on new events and send new commands to the `decider` recursively, in single transaction.
/// It is using a [StateRepository] to fetch the current state and to save the new state.
///
/// Generic parameters:
///
/// - `C` - Command
/// - `S` - State
/// - `E` - Event
/// - `Repository` - State repository
/// - `Version` - Version
/// - `Error` - Error
pub struct StateStoredOrchestratingAggregate<'a, C, S, E, Repository, Version, Error>
where
    Repository: StateRepository<C, S, Version, Error>,
{
    repository: Repository,
    decider: Decider<'a, C, S, E, Error>,
    saga: Saga<'a, E, C>,
    _marker: PhantomData<(C, S, E, Version, Error)>,
}

impl<C, S, E, Repository, Version, Error> StateComputation<C, S, E, Error>
    for StateStoredOrchestratingAggregate<'_, C, S, E, Repository, Version, Error>
where
    Repository: StateRepository<C, S, Version, Error>,
    S: Clone,
{
    /// Computes new state based on the current state and the command.
    fn compute_new_state(&self, current_state: Option<S>, command: &C) -> Result<S, Error> {
        let effective_current_state =
            current_state.unwrap_or_else(|| (self.decider.initial_state)());
        let events = (self.decider.decide)(command, &effective_current_state)?;
        let mut new_state = events.iter().fold(effective_current_state, |state, event| {
            (self.decider.evolve)(&state, event)
        });
        let commands = events
            .iter()
            .flat_map(|event: &E| self.saga.compute_new_actions(event))
            .collect::<Vec<C>>();
        for action in commands {
            new_state = self.compute_new_state(Some(new_state.clone()), &action)?;
        }
        Ok(new_state)
    }
}

#[cfg(not(feature = "not-send-futures"))]
impl<C, S, E, Repository, Version, Error> StateRepository<C, S, Version, Error>
    for StateStoredOrchestratingAggregate<'_, C, S, E, Repository, Version, Error>
where
    Repository: StateRepository<C, S, Version, Error> + Sync,
    C: Sync,
    S: Sync,
    E: Sync,
    Version: Sync,
    Error: Sync,
{
    /// Fetches current state, based on the command.
    async fn fetch_state(&self, command: &C) -> Result<Option<(S, Version)>, Error> {
        self.repository.fetch_state(command).await
    }
    /// Saves state.
    async fn save(&self, state: &S, version: &Option<Version>) -> Result<(S, Version), Error> {
        self.repository.save(state, version).await
    }
}

#[cfg(feature = "not-send-futures")]
impl<C, S, E, Repository, Version, Error> StateRepository<C, S, Version, Error>
    for StateStoredOrchestratingAggregate<'_, C, S, E, Repository, Version, Error>
where
    Repository: StateRepository<C, S, Version, Error>,
{
    /// Fetches current state, based on the command.
    async fn fetch_state(&self, command: &C) -> Result<Option<(S, Version)>, Error> {
        self.repository.fetch_state(command).await
    }
    /// Saves state.
    async fn save(&self, state: &S, version: &Option<Version>) -> Result<(S, Version), Error> {
        self.repository.save(state, version).await
    }
}

#[cfg(not(feature = "not-send-futures"))]
impl<'a, C, S, E, Repository, Version, Error>
    StateStoredOrchestratingAggregate<'a, C, S, E, Repository, Version, Error>
where
    Repository: StateRepository<C, S, Version, Error> + Sync,
    C: Sync,
    S: Sync + Clone,
    E: Sync,
    Version: Sync,
    Error: Sync,
{
    /// Creates a new instance of [StateStoredAggregate].
    pub fn new(
        repository: Repository,
        decider: Decider<'a, C, S, E, Error>,
        saga: Saga<'a, E, C>,
    ) -> Self {
        StateStoredOrchestratingAggregate {
            repository,
            decider,
            saga,
            _marker: PhantomData,
        }
    }
    /// Handles the command by fetching the state from the repository, computing new state based on the current state and the command, and saving the new state to the repository.
    pub async fn handle(&self, command: &C) -> Result<(S, Version), Error> {
        let state_version = self.fetch_state(command).await?;
        match state_version {
            None => {
                let new_state = self.compute_new_state(None, command)?;
                let saved_state = self.save(&new_state, &None).await?;
                Ok(saved_state)
            }
            Some((state, version)) => {
                let new_state = self.compute_new_state(Some(state), command)?;
                let saved_state = self.save(&new_state, &Some(version)).await?;
                Ok(saved_state)
            }
        }
    }
}

#[cfg(feature = "not-send-futures")]
impl<'a, C, S, E, Repository, Version, Error>
    StateStoredOrchestratingAggregate<'a, C, S, E, Repository, Version, Error>
where
    Repository: StateRepository<C, S, Version, Error>,
    S: Clone,
{
    /// Creates a new instance of [StateStoredAggregate].
    pub fn new(
        repository: Repository,
        decider: Decider<'a, C, S, E, Error>,
        saga: Saga<'a, E, C>,
    ) -> Self {
        StateStoredOrchestratingAggregate {
            repository,
            decider,
            saga,
            _marker: PhantomData,
        }
    }
    /// Handles the command by fetching the state from the repository, computing new state based on the current state and the command, and saving the new state to the repository.
    pub async fn handle(&self, command: &C) -> Result<(S, Version), Error> {
        let state_version = self.fetch_state(command).await?;
        match state_version {
            None => {
                let new_state = self.compute_new_state(None, command)?;
                let saved_state = self.save(&new_state, &None).await?;
                Ok(saved_state)
            }
            Some((state, version)) => {
                let new_state = self.compute_new_state(Some(state), command)?;
                let saved_state = self.save(&new_state, &Some(version)).await?;
                Ok(saved_state)
            }
        }
    }
}
