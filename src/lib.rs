//! # FModel Rust
//!
//! When you’re developing an information system to automate the activities of the business, you are modeling the business.
//! The abstractions that you design, the behaviors that you implement, and the UI interactions that you build all reflect
//! the business — together, they constitute the model of the domain.
//!
//! ![event-modeling](https://github.com/fraktalio/fmodel-ts/raw/main/.assets/event-modeling.png)
//
//! ## `IOR<Library, Inspiration>`
//!
//! This crate can be used as a library, or as an inspiration, or both. It provides just enough tactical Domain-Driven
//! Design patterns, optimised for Event Sourcing and CQRS.
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
//! ## Examples
//!
//! - [Gift Card Demo - with Axon](https://github.com/AxonIQ/axon-rust/tree/main/gift-card-rust)
//! - [FModel Rust Tests](https://github.com/fraktalio/fmodel-rust/tree/main/tests)
//!
//! ## GitHub
//!
//! - [FModel Rust](https://github.com/fraktalio/fmodel-rust)
//!
//! ## FModel in other languages
//!
//!  - [FModel Kotlin](https://github.com/fraktalio/fmodel/)
//!  - [FModel TypeScript](https://github.com/fraktalio/fmodel-ts/)
//!  - [FModel Java](https://github.com/fraktalio/fmodel-java/)
//!
//! ## Credits
//!
//! Special credits to `Jérémie Chassaing` for sharing his [research](https://www.youtube.com/watch?v=kgYGMVDHQHs)
//! and `Adam Dymitruk` for hosting the meetup.
//!
//! ---
//! Created with `love` by [Fraktalio](https://fraktalio.com/)

pub mod aggregate;
pub mod decider;
pub mod materialized_view;
pub mod saga;
pub mod saga_manager;
pub mod view;

/// The [DecideFunction] function is used to decide which events to produce based on the command and the current state.
pub type DecideFunction<'a, C, S, E> = Box<dyn Fn(&C, &S) -> Vec<E> + 'a + Send + Sync>;
/// The [EvolveFunction] function is used to evolve the state based on the current state and the event.
pub type EvolveFunction<'a, S, E> = Box<dyn Fn(&S, &E) -> S + 'a + Send + Sync>;
/// The [InitialStateFunction] function is used to produce the initial state.
pub type InitialStateFunction<'a, S> = Box<dyn Fn() -> S + 'a + Send + Sync>;
/// The [ReactFunction] function is used to decide what actions/A to execute next based on the action result/AR.
pub type ReactFunction<'a, AR, A> = Box<dyn Fn(&AR) -> Vec<A> + 'a + Send + Sync>;
