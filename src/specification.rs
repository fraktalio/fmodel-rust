//! ## A test specification DSL for deciders and views that supports the given-when-then format.

use pretty_assertions::assert_eq;

use crate::{
    decider::{Decider, EventComputation, StateComputation},
    view::{View, ViewStateComputation},
};

// ########################################################
// ############# Decider Specification DSL ################
// ########################################################

/// A test specification DSL for deciders that supports the `given-when-then` format.
/// The DSL is used to specify the events that have already occurred (GIVEN), the command that is being executed (WHEN), and the expected events (THEN) that should be generated.
pub struct DeciderTestSpecification<'a, Command, State, Event, Error>
where
    Event: PartialEq + std::fmt::Debug,
    Error: PartialEq + std::fmt::Debug,
{
    events: Vec<Event>,
    state: Option<State>,
    command: Option<Command>,
    decider: Option<Decider<'a, Command, State, Event, Error>>,
}

impl<Command, State, Event, Error> Default
    for DeciderTestSpecification<'_, Command, State, Event, Error>
where
    Event: PartialEq + std::fmt::Debug,
    Error: PartialEq + std::fmt::Debug,
{
    fn default() -> Self {
        Self {
            events: Vec::new(),
            state: None,
            command: None,
            decider: None,
        }
    }
}

impl<'a, Command, State, Event, Error> DeciderTestSpecification<'a, Command, State, Event, Error>
where
    Command: std::fmt::Debug,
    Event: PartialEq + std::fmt::Debug,
    State: PartialEq + std::fmt::Debug,
    Error: PartialEq + std::fmt::Debug,
{
    #[allow(dead_code)]
    /// Specify the decider you want to test
    pub fn for_decider(mut self, decider: Decider<'a, Command, State, Event, Error>) -> Self {
        self.decider = Some(decider);
        self
    }

    #[allow(dead_code)]
    /// Given preconditions / previous events
    pub fn given(mut self, events: Vec<Event>) -> Self {
        self.events = events;
        self
    }

    #[allow(dead_code)]
    /// Given preconditions / previous state
    pub fn given_state(mut self, state: Option<State>) -> Self {
        self.state = state;
        self
    }

    #[allow(dead_code)]
    /// When action/command
    pub fn when(mut self, command: Command) -> Self {
        self.command = Some(command);
        self
    }

    #[allow(dead_code)]
    #[track_caller]
    /// Then expect result / new events
    pub fn then(self, expected_events: Vec<Event>) {
        let decider = self
            .decider
            .expect("Decider must be initialized. Did you forget to call `for_decider`?");
        let command = self
            .command
            .expect("Command must be initialized. Did you forget to call `when`?");
        let events = self.events;

        let new_events_result = decider.compute_new_events(&events, &command);
        let new_events = match new_events_result {
            Ok(events) => events,
            Err(error) => {
                panic!("Events were expected but the decider returned an error instead: {error:?}")
            }
        };
        assert_eq!(
            new_events, expected_events,
            "Actual and Expected events do not match!\nCommand: {command:?}\n",
        );
    }

    #[allow(dead_code)]
    #[track_caller]
    /// Then expect result / new events
    pub fn then_state(self, expected_state: State) {
        let decider = self
            .decider
            .expect("Decider must be initialized. Did you forget to call `for_decider`?");
        let command = self
            .command
            .expect("Command must be initialized. Did you forget to call `when`?");
        let state = self.state;

        let new_state_result = decider.compute_new_state(state, &command);
        let new_state = match new_state_result {
            Ok(state) => state,
            Err(error) => {
                panic!("State was expected but the decider returned an error instead: {error:?}")
            }
        };
        assert_eq!(
            new_state, expected_state,
            "Actual and Expected states do not match.\nCommand: {command:?}\n"
        );
    }

    #[allow(dead_code)]
    #[track_caller]
    /// Then expect error result / these are not events
    pub fn then_error(self, expected_error: Error) {
        let decider = self
            .decider
            .expect("Decider must be initialized. Did you forget to call `for_decider`?");
        let command = self
            .command
            .expect("Command must be initialized. Did you forget to call `when`?");
        let events = self.events;

        let error_result = decider.compute_new_events(&events, &command);
        let error = match error_result {
            Ok(events) => {
                panic!("An error was expected but the decider returned events instead: {events:?}")
            }
            Err(error) => error,
        };
        assert_eq!(
            error, expected_error,
            "Actual and Expected errors do not match.\nCommand: {command:?}\n"
        );
    }
}

// ########################################################
// ############### View Specification DSL #################
// ########################################################

/// A test specification DSL for views that supports the `given-then`` format.
/// The DSL is used to specify the events that have already occurred (GIVEN), and the expected view state (THEN) that should be generated based on these events.
pub struct ViewTestSpecification<'a, State, Event>
where
    State: PartialEq + std::fmt::Debug,
{
    events: Vec<Event>,
    view: Option<View<'a, State, Event>>,
}

impl<State, Event> Default for ViewTestSpecification<'_, State, Event>
where
    State: PartialEq + std::fmt::Debug,
{
    fn default() -> Self {
        Self {
            events: Vec::new(),
            view: None,
        }
    }
}

impl<'a, State, Event> ViewTestSpecification<'a, State, Event>
where
    State: PartialEq + std::fmt::Debug,
    Event: std::fmt::Debug,
{
    #[allow(dead_code)]
    /// Specify the view you want to test
    pub fn for_view(mut self, view: View<'a, State, Event>) -> Self {
        self.view = Some(view);
        self
    }

    #[allow(dead_code)]
    /// Given preconditions / events
    pub fn given(mut self, events: Vec<Event>) -> Self {
        self.events = events;
        self
    }

    #[allow(dead_code)]
    #[track_caller]
    /// Then expect evolving new state of the view
    pub fn then(self, expected_state: State) {
        let view = self
            .view
            .expect("View must be initialized. Did you forget to call `for_view`?");

        let events = self.events;

        let initial_state = (view.initial_state)();
        let event_refs: Vec<&Event> = events.iter().collect();
        let new_state_result = view.compute_new_state(Some(initial_state), &event_refs);

        assert_eq!(
            new_state_result, expected_state,
            "Actual and Expected states do not match.\nEvents: {events:?}\n"
        );
    }
}
