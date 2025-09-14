#![deny(missing_docs)]
//! # FModel Rust
//!
//! When you’re developing an information system to automate the activities of the business, you are modeling the business.
//! The abstractions that you design, the behaviors that you implement, and the UI interactions that you build all reflect
//! the business — together, they constitute the model of the domain.
//!
//! ![event-modeling](https://github.com/fraktalio/fmodel-ts/raw/main/.assets/event-modeling.png)
//!
//! ## `IOR<Library, Inspiration>`
//!
//! This crate can be used as a library, or as an inspiration, or both. It provides just enough tactical Domain-Driven
//! Design patterns, optimised for Event Sourcing and CQRS.
//!
//! ![onion architecture image](https://github.com/fraktalio/fmodel/blob/d8643a7d0de30b79f0b904f7a40233419c463fc8/.assets/onion.png?raw=true)
//!
//!## Decider
//!
//! `Decider` is a datatype/struct that represents the main decision-making algorithm. It belongs to the Domain layer. It
//! has three
//! generic parameters `C`, `S`, `E` , representing the type of the values that `Decider` may contain or use.
//! `Decider` can be specialized for any type `C` or `S` or `E` because these types do not affect its
//! behavior. `Decider` behaves the same for `C`=`Int` or `C`=`YourCustomType`, for example.
//!
//! `Decider` is a pure domain component.
//!
//! - `C` - Command
//! - `S` - State
//! - `E` - Event
//!
//! ```rust
//! pub type DecideFunction<'a, C, S, E> = Box<dyn Fn(&C, &S) -> Vec<E> + 'a + Send + Sync>;
//! pub type EvolveFunction<'a, S, E> = Box<dyn Fn(&S, &E) -> S + 'a + Send + Sync>;
//! pub type InitialStateFunction<'a, S> = Box<dyn Fn() -> S + 'a + Send + Sync>;
//!
//! pub struct Decider<'a, C: 'a, S: 'a, E: 'a> {
//!     pub decide: DecideFunction<'a, C, S, E>,
//!     pub evolve: EvolveFunction<'a, S, E>,
//!     pub initial_state: InitialStateFunction<'a, S>,
//! }
//! ```
//!
//! Additionally, `initialState` of the Decider is introduced to gain more control over the initial state of the Decider.
//!
//! ### Event-sourcing aggregate
//!
//! [aggregate::EventSourcedAggregate]  is using/delegating a `Decider` to handle commands and produce new events.
//!
//! It belongs to the Application layer.
//!
//! In order to handle the command, aggregate needs to fetch the current state (represented as a list/vector of events)
//! via `EventRepository.fetchEvents` async function, and then delegate the command to the decider which can produce new
//! events as
//! a result. Produced events are then stored via `EventRepository.save` async function.
//!
//! It is a formalization of the event sourced information system.
//!
//! ### State-stored aggregate
//!
//! [aggregate::StateStoredAggregate] is using/delegating a `Decider` to handle commands and produce new state.
//!
//! It belongs to the Application layer.
//!
//! In order to handle the command, aggregate needs to fetch the current state via `StateRepository.fetchState` async function first,
//! and then
//! delegate the command to the decider which can produce new state as a result. New state is then stored
//! via `StateRepository.save` async function.
//!
//! ## View
//!
//! `View`  is a datatype that represents the event handling algorithm, responsible for translating the events into
//! denormalized state, which is more adequate for querying. It belongs to the Domain layer. It is usually used to create
//! the view/query side of the CQRS pattern. Obviously, the command side of the CQRS is usually event-sourced aggregate.
//!
//! It has two generic parameters `S`, `E`, representing the type of the values that `View` may contain or use.
//! `View` can be specialized for any type of `S`, `E` because these types do not affect its behavior.
//! `View` behaves the same for `E`=`Int` or `E`=`YourCustomType`, for example.
//!
//! `View` is a pure domain component.
//!
//! - `S` - State
//! - `E` - Event
//!
//! ```rust
//! pub type EvolveFunction<'a, S, E> = Box<dyn Fn(&S, &E) -> S + 'a + Send + Sync>;
//! pub type InitialStateFunction<'a, S> = Box<dyn Fn() -> S + 'a + Send + Sync>;
//!
//! pub struct View<'a, S: 'a, E: 'a> {
//!     pub evolve: EvolveFunction<'a, S, E>,
//!     pub initial_state: InitialStateFunction<'a, S>,
//! }
//! ```
//!
//! ### Materialized View
//!
//! [materialized_view::MaterializedView] is using/delegating a `View` to handle events of type `E` and to maintain
//! a state of denormalized
//! projection(s) as a
//! result. Essentially, it represents the query/view side of the CQRS pattern.
//!
//! It belongs to the Application layer.
//!
//! In order to handle the event, materialized view needs to fetch the current state via `ViewStateRepository.fetchState`
//! suspending function first, and then delegate the event to the view, which can produce new state as a result. New state
//! is then stored via `ViewStateRepository.save` suspending function.
//!
//!
//! ## Saga
//!
//! `Saga` is a datatype that represents the central point of control, deciding what to execute next (`A`), based on the action result (`AR`).
//! It has two generic parameters `AR`/Action Result, `A`/Action , representing the type of the values that Saga may contain or use.
//! `'a` is used as a lifetime parameter, indicating that all references contained within the struct (e.g., references within the function closures) must have a lifetime that is at least as long as 'a.
//!
//! `Saga` is a pure domain component.
//!
//! - `AR` - Action Result/Event
//! - `A` - Action/Command
//!
//! ```rust
//! pub type ReactFunction<'a, AR, A> = Box<dyn Fn(&AR) -> Vec<A> + 'a + Send + Sync>;
//! pub struct Saga<'a, AR: 'a, A: 'a> {
//!     pub react: ReactFunction<'a, AR, A>,
//! }
//! ```
//!
//! ### Saga Manager
//!
//! [saga_manager::SagaManager] is using/delegating a `Saga` to react to the action result and to publish the new actions.
//!
//! It belongs to the Application layer.
//!
//! It is using a [saga::Saga] to react to the action result and to publish the new actions.
//! It is using an [saga_manager::ActionPublisher] to publish the new actions.
//!
//! ## Clear separation between data and behaviour
//!
//!```rust
//! use fmodel_rust::decider::Decider;
//! // ## Algebraic Data Types
//! //
//! // In Rust, we can use ADTs to model our application's domain entities and relationships in a functional way, clearly defining the set of possible values and states.
//! // Rust has two main types of ADTs: `enum` and `struct`.
//! //
//! // - `enum` is used to define a type that can take on one of several possible variants - modeling a `sum/OR` type.
//! // - `struct` is used to express a type that has named fields - modeling a `product/AND` type.
//! //
//! // ADTs will help with
//! //
//! // - representing the business domain in the code accurately
//! // - enforcing correctness
//! // - reducing the likelihood of bugs.
//!
//!
//! // ### `C` / Command / Intent to change the state of the system
//!
//! // models Sum/Or type / multiple possible variants
//! pub enum OrderCommand {
//!     Create(CreateOrderCommand),
//!     Update(UpdateOrderCommand),
//!     Cancel(CancelOrderCommand),
//! }
//! // models Product/And type / a concrete variant, consisting of named fields
//! pub struct CreateOrderCommand {
//!     pub order_id: u32,
//!     pub customer_name: String,
//!     pub items: Vec<String>,
//! }
//! // models Product/And type / a concrete variant, consisting of named fields
//! pub struct UpdateOrderCommand {
//!     pub order_id: u32,
//!     pub new_items: Vec<String>,
//! }
//! // models Product/And type / a concrete variant, consisting of named fields
//! pub struct CancelOrderCommand {
//!     pub order_id: u32,
//! }
//!
//! // ### `E` / Event / Fact
//!
//! // models Sum/Or type / multiple possible variants
//! pub enum OrderEvent {
//!     Created(OrderCreatedEvent),
//!     Updated(OrderUpdatedEvent),
//!     Cancelled(OrderCancelledEvent),
//! }
//! // models Product/And type / a concrete variant, consisting of named fields
//! pub struct OrderCreatedEvent {
//!     pub order_id: u32,
//!     pub customer_name: String,
//!     pub items: Vec<String>,
//! }
//! // models Product/And type / a concrete variant, consisting of named fields
//! pub struct OrderUpdatedEvent {
//!     pub order_id: u32,
//!     pub updated_items: Vec<String>,
//! }
//! // models Product/And type / a concrete variant, consisting of named fields
//! pub struct OrderCancelledEvent {
//!     pub order_id: u32,
//! }
//!
//! // ### `S` / State / Current state of the system/aggregate/entity
//! #[derive(Clone)]
//! struct OrderState {
//!     order_id: u32,
//!     customer_name: String,
//!     items: Vec<String>,
//!     is_cancelled: bool,
//! }
//!
//! // ## Modeling the Behaviour of our domain
//! //
//! //  - algebraic data types form the structure of our entities (commands, state, and events).
//! //  - functions/lambda offers the algebra of manipulating the entities in a compositional manner, effectively modeling the behavior.
//! //
//! // This leads to modularity in design and a clear separation of the entity’s structure and functions/behaviour of the entity.
//! //
//! // Fmodel library offers generic and abstract components to specialize in for your specific case/expected behavior
//!
//! fn decider<'a>() -> Decider<'a, OrderCommand, OrderState, OrderEvent> {
//!     Decider {
//!         // Your decision logic goes here.
//!         decide: Box::new(|command, state| match command {
//!             // Exhaustive pattern matching on the command
//!             OrderCommand::Create(create_cmd) => {
//!                 Ok(vec![OrderEvent::Created(OrderCreatedEvent {
//!                     order_id: create_cmd.order_id,
//!                     customer_name: create_cmd.customer_name.to_owned(),
//!                     items: create_cmd.items.to_owned(),
//!                 })])
//!             }
//!             OrderCommand::Update(update_cmd) => {
//!                 // Your validation logic goes here
//!                 if state.order_id == update_cmd.order_id {
//!                     Ok(vec![OrderEvent::Updated(OrderUpdatedEvent {
//!                         order_id: update_cmd.order_id,
//!                         updated_items: update_cmd.new_items.to_owned(),
//!                     })])
//!                 } else {
//!                     // In case of validation failure, return empty list of events or error event
//!                     Ok(vec![])
//!                 }
//!             }
//!             OrderCommand::Cancel(cancel_cmd) => {
//!                 // Your validation logic goes here
//!                 if state.order_id == cancel_cmd.order_id {
//!                     Ok(vec![OrderEvent::Cancelled(OrderCancelledEvent {
//!                         order_id: cancel_cmd.order_id,
//!                     })])
//!                 } else {
//!                     // In case of validation failure, return empty list of events or error event
//!                     Ok(vec![])
//!                 }
//!             }
//!         }),
//!         // Evolve the state based on the event(s)
//!         evolve: Box::new(|state, event| {
//!             let mut new_state = state.clone();
//!             // Exhaustive pattern matching on the event
//!             match event {
//!                 OrderEvent::Created(created_event) => {
//!                     new_state.order_id = created_event.order_id;
//!                     new_state.customer_name = created_event.customer_name.to_owned();
//!                     new_state.items = created_event.items.to_owned();
//!                 }
//!                 OrderEvent::Updated(updated_event) => {
//!                     new_state.items = updated_event.updated_items.to_owned();
//!                 }
//!                 OrderEvent::Cancelled(_) => {
//!                     new_state.is_cancelled = true;
//!                 }
//!             }
//!             new_state
//!         }),
//!         // Initial state
//!         initial_state: Box::new(|| OrderState {
//!             order_id: 0,
//!             customer_name: "".to_string(),
//!             items: Vec::new(),
//!             is_cancelled: false,
//!         }),
//!     }
//! }
//! ```
//!
//! ## Examples
//!
//! - [Restaurant Demo - with Postgres](https://github.com/fraktalio/fmodel-rust-demo)
//! - [Gift Card Demo - with Axon](https://!github.com/AxonIQ/axon-rust/tree/main/gift-card-rust)
//! - [FModel Rust Tests](https://!github.com/fraktalio/fmodel-rust/tree/main/tests)
//!
//! ## GitHub
//!
//! - [FModel Rust](https://!github.com/fraktalio/fmodel-rust)
//!
//! ## FModel in other languages
//!
//!  - [FModel Kotlin](https://!github.com/fraktalio/fmodel/)
//!  - [FModel TypeScript](https://!github.com/fraktalio/fmodel-ts/)
//!  - [FModel Java](https://!github.com/fraktalio/fmodel-java/)
//!
//! ## Credits
//!
//! Special credits to `Jérémie Chassaing` for sharing his [research](https://!www.youtube.com/watch?v=kgYGMVDHQHs)
//! and `Adam Dymitruk` for hosting the meetup.
//!
//! ---
//! Created with `love` by [Fraktalio](https://!fraktalio.com/)

use decider::Decider;
use saga::Saga;
use serde::{Deserialize, Serialize};
use view::View;

/// Aggregate module - belongs to the `Application` layer - composes pure logic and effects (fetching, storing)
pub mod aggregate;
/// Decider module - belongs to the `Domain` layer - pure decision making component - pure logic
pub mod decider;
/// Materialized View module - belongs to the `Application` layer - composes pure event handling algorithm and effects (fetching, storing)
pub mod materialized_view;
/// Saga module - belongs to the `Domain` layer - pure mapper of action results/events into new actions/commands
pub mod saga;
/// Saga Manager module - belongs to the `Application` layer - composes pure saga and effects (publishing)
pub mod saga_manager;
/// Given-When-Then Test specificatin domain specific language - unit testing
pub mod specification;
/// View module - belongs to the `Domain` layer - pure event handling algorithm
pub mod view;

/// The [DecideFunction] function is used to decide which events to produce based on the command and the current state.
#[cfg(not(feature = "not-send-futures"))]
pub type DecideFunction<'a, C, S, E, Error> =
    Box<dyn Fn(&C, &S) -> Result<Vec<E>, Error> + 'a + Send + Sync>;
/// The [EvolveFunction] function is used to evolve the state based on the current state and the event.
#[cfg(not(feature = "not-send-futures"))]
pub type EvolveFunction<'a, S, E> = Box<dyn Fn(&S, &E) -> S + 'a + Send + Sync>;
/// The [InitialStateFunction] function is used to produce the initial state.
#[cfg(not(feature = "not-send-futures"))]
pub type InitialStateFunction<'a, S> = Box<dyn Fn() -> S + 'a + Send + Sync>;
/// The [ReactFunction] function is used to decide what actions/A to execute next based on the action result/AR.
#[cfg(not(feature = "not-send-futures"))]
pub type ReactFunction<'a, AR, A> = Box<dyn Fn(&AR) -> Vec<A> + 'a + Send + Sync>;

/// The [DecideFunction] function is used to decide which events to produce based on the command and the current state.
#[cfg(feature = "not-send-futures")]
pub type DecideFunction<'a, C, S, E, Error> = Box<dyn Fn(&C, &S) -> Result<Vec<E>, Error> + 'a>;
/// The [EvolveFunction] function is used to evolve the state based on the current state and the event.
#[cfg(feature = "not-send-futures")]
pub type EvolveFunction<'a, S, E> = Box<dyn Fn(&S, &E) -> S + 'a>;
/// The [InitialStateFunction] function is used to produce the initial state.
#[cfg(feature = "not-send-futures")]
pub type InitialStateFunction<'a, S> = Box<dyn Fn() -> S + 'a>;
/// The [ReactFunction] function is used to decide what actions/A to execute next based on the action result/AR.
#[cfg(feature = "not-send-futures")]
pub type ReactFunction<'a, AR, A> = Box<dyn Fn(&AR) -> Vec<A> + 'a>;

/// Generic Combined/Sum Enum of two variants
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Sum<A, B> {
    /// First variant
    First(A),
    /// Second variant
    Second(B),
}

/// Generic Combined/Sum Enum of three variants
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Sum3<A, B, C> {
    /// First variant
    First(A),
    /// Second variant
    Second(B),
    /// Third variant
    Third(C),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
/// Generic Combined/Sum Enum of four variants
pub enum Sum4<A, B, C, D> {
    /// First variant
    First(A),
    /// Second variant
    Second(B),
    /// Third variant
    Third(C),
    /// Fourth variant
    Fourth(D),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
/// Generic Combined/Sum Enum of five variants
pub enum Sum5<A, B, C, D, E> {
    /// First variant
    First(A),
    /// Second variant
    Second(B),
    /// Third variant
    Third(C),
    /// Fourth variant
    Fourth(D),
    /// Fifth variant
    Fifth(E),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
/// Generic Combined/Sum Enum of six variants
pub enum Sum6<A, B, C, D, E, F> {
    /// First variant
    First(A),
    /// Second variant
    Second(B),
    /// Third variant
    Third(C),
    /// Fourth variant
    Fourth(D),
    /// Fifth variant
    Fifth(E),
    /// Sixth variant
    Sixth(F),
}

/// Convenient type alias that represents 3 combined Deciders
type Decider3<'a, C1, C2, C3, S1, S2, S3, E1, E2, E3, Error> =
    Decider<'a, Sum3<C1, C2, C3>, (S1, S2, S3), Sum3<E1, E2, E3>, Error>;

/// Convenient type alias that represents 4 combined Deciders
type Decider4<'a, C1, C2, C3, C4, S1, S2, S3, S4, E1, E2, E3, E4, Error> =
    Decider<'a, Sum4<C1, C2, C3, C4>, (S1, S2, S3, S4), Sum4<E1, E2, E3, E4>, Error>;

/// Convenient type alias that represents 5 combined Deciders
type Decider5<'a, C1, C2, C3, C4, C5, S1, S2, S3, S4, S5, E1, E2, E3, E4, E5, Error> =
    Decider<'a, Sum5<C1, C2, C3, C4, C5>, (S1, S2, S3, S4, S5), Sum5<E1, E2, E3, E4, E5>, Error>;

/// Convenient type alias that represents 6 combined Deciders
type Decider6<'a, C1, C2, C3, C4, C5, C6, S1, S2, S3, S4, S5, S6, E1, E2, E3, E4, E5, E6, Error> =
    Decider<
        'a,
        Sum6<C1, C2, C3, C4, C5, C6>,
        (S1, S2, S3, S4, S5, S6),
        Sum6<E1, E2, E3, E4, E5, E6>,
        Error,
    >;

/// Convenient type alias that represents 3 merged Views
type View3<'a, S1, S2, S3, E> = View<'a, (S1, S2, S3), E>;

/// Convenient type alias that represents 4 merged Deciders
type View4<'a, S1, S2, S3, S4, E> = View<'a, (S1, S2, S3, S4), E>;

/// Convenient type alias that represents 5 merged Deciders
type View5<'a, S1, S2, S3, S4, S5, E> = View<'a, (S1, S2, S3, S4, S5), E>;

/// Convenient type alias that represents 6 merged Deciders
type View6<'a, S1, S2, S3, S4, S5, S6, E> = View<'a, (S1, S2, S3, S4, S5, S6), E>;

/// Convenient type alias that represents 3 merged Sagas
type Saga3<'a, AR, A1, A2, A3> = Saga<'a, AR, Sum3<A1, A2, A3>>;

/// Convenient type alias that represents 4 merged Sagas
type Saga4<'a, AR, A1, A2, A3, A4> = Saga<'a, AR, Sum4<A1, A2, A3, A4>>;

/// Convenient type alias that represents 5 merged Sagas
type Saga5<'a, AR, A1, A2, A3, A4, A5> = Saga<'a, AR, Sum5<A1, A2, A3, A4, A5>>;

/// Convenient type alias that represents 6 merged Sagas
type Saga6<'a, AR, A1, A2, A3, A4, A5, A6> = Saga<'a, AR, Sum6<A1, A2, A3, A4, A5, A6>>;

/// Identify the state/command/event.
/// It is used to identify the concept to what the state/command/event belongs to. For example, the `order_id` or `restaurant_id`.
pub trait Identifier {
    /// Returns the identifier of the state/command/event
    fn identifier(&self) -> String;
}

impl<A, B> Identifier for Sum<A, B>
where
    A: Identifier,
    B: Identifier,
{
    fn identifier(&self) -> String {
        match self {
            Sum::First(a) => a.identifier(),
            Sum::Second(b) => b.identifier(),
        }
    }
}
