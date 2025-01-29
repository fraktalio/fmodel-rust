# **f`(`model`)`** - Functional Domain Modeling with Rust

> Publicly available at [crates.io](https://crates.io/crates/fmodel-rust) and [docs.rs](https://docs.rs/fmodel-rust/latest/fmodel_rust/)

> From version 0.7.0+, the library is using [`async fn` in Traits](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html) feature, which is currently available only in stable Rust 1.75.0+.

> If you are using older version of Rust, please use version 0.6.0 of the library. It depends on `async-trait` crate. Version 0.6.0 is not maintained anymore, only patched for security issues and bugs.

<!-- TOC -->
* [**f`(`model`)`** - Functional Domain Modeling with Rust](#fmodel---functional-domain-modeling-with-rust)
  * [`IOR<Library, Inspiration>`](#iorlibrary-inspiration)
  * [Abstraction and generalization](#abstraction-and-generalization)
  * [`Box<dyn Fn(&C, &S) -> Vec<E>>`](#boxdyn-fnc-s---vece)
  * [`Box<dyn Fn(&S, &E) -> S>`](#boxdyn-fns-e---s)
  * [Decider](#decider)
    * [Event-sourcing aggregate](#event-sourcing-aggregate)
    * [State-stored aggregate](#state-stored-aggregate)
  * [View](#view)
    * [Materialized View](#materialized-view)
  * [Algebraic Data Types](#algebraic-data-types)
    * [`C` / Command / Intent to change the state of the system](#c--command--intent-to-change-the-state-of-the-system)
    * [`E` / Event / Fact](#e--event--fact)
    * [`S` / State / Current state of the system/aggregate/entity](#s--state--current-state-of-the-systemaggregateentity)
  * [Modeling the Behaviour of our domain](#modeling-the-behaviour-of-our-domain)
  * [The Application layer](#the-application-layer)
  * [Fearless Concurrency](#fearless-concurrency)
  * [Install the crate as a dependency of your project](#install-the-crate-as-a-dependency-of-your-project)
  * [Examples](#examples)
  * [FModel in other languages](#fmodel-in-other-languages)
  * [Further reading](#further-reading)
  * [Credits](#credits)
<!-- TOC -->

When you’re developing an information system to automate the activities of the business, you are modeling the business.
The abstractions that you design, the behaviors that you implement, and the UI interactions that you build all reflect
the business — together, they constitute the model of the domain.

![event-modeling](https://github.com/fraktalio/fmodel-ts/raw/main/.assets/event-modeling.png)

## `IOR<Library, Inspiration>`

This project can be used as a library, or as an inspiration, or both. It provides just enough tactical Domain-Driven
Design patterns, optimised for Event Sourcing and CQRS.

## Abstraction and generalization

Abstractions can hide irrelevant details and use names to reference objects. It emphasizes what an object is or does
rather than how it is represented or how it works.

Generalization reduces complexity by replacing multiple entities which perform similar functions with a single
construct.

Abstraction and generalization are often used together. Abstracts are generalized through parameterization to provide
more excellent utility.

## `Box<dyn Fn(&C, &S) -> Result<Vec<E>, Error>`

`type DecideFunction<'a, C, S, E> = Box<dyn Fn(&C, &S) -> Result<Vec<E>, Error> + 'a + Send + Sync>`

On a higher level of abstraction, any information system is responsible for handling the intent (`Command`) and based on
the current `State`, produce new facts (`Events`):

- given the current `State/S` *on the input*,
- when `Command/C` is handled *on the input*,
- expect `Vec` of new `Events/E` to be published/emitted *on the output*

## `Box<dyn Fn(&S, &E) -> S>`

`type EvolveFunction<'a, S, E> = Box<dyn Fn(&S, &E) -> S + 'a + Send + Sync>`

The new state is always evolved out of the current state `S` and the current event `E`:

- given the current `State/S` *on the input*,
- when `Event/E` is handled *on the input*,
- expect new `State/S` to be published *on the output*

Two functions are wrapped in a datatype class (algebraic data structure), which is generalized with three generic
parameters:

```rust
pub struct Decider<'a, C: 'a, S: 'a, E: 'a> {
    pub decide: DecideFunction<'a, C, S, E>,
    pub evolve: EvolveFunction<'a, S, E>,
    pub initial_state: InitialStateFunction<'a, S>,
}
```

`Decider` is the most important datatype, but it is not the only one. There are others:

![onion architecture image](https://github.com/fraktalio/fmodel/blob/d8643a7d0de30b79f0b904f7a40233419c463fc8/.assets/onion.png?raw=true)

## Decider

`Decider` is a datatype/struct that represents the main decision-making algorithm. It belongs to the Domain layer. It
has three
generic parameters `C`, `S`, `E` , representing the type of the values that `Decider` may contain or use.
`Decider` can be specialized for any type `C` or `S` or `E` because these types do not affect its
behavior. `Decider` behaves the same for `C`=`Int` or `C`=`YourCustomType`, for example.

`Decider` is a pure domain component.

- `C` - Command
- `S` - State
- `E` - Event

```rust
pub type DecideFunction<'a, C, S, E> = Box<dyn Fn(&C, &S) -> Vec<E> + 'a + Send + Sync>;
pub type EvolveFunction<'a, S, E> = Box<dyn Fn(&S, &E) -> S + 'a + Send + Sync>;
pub type InitialStateFunction<'a, S> = Box<dyn Fn() -> S + 'a + Send + Sync>;

pub struct Decider<'a, C: 'a, S: 'a, E: 'a> {
    pub decide: DecideFunction<'a, C, S, E>,
    pub evolve: EvolveFunction<'a, S, E>,
    pub initial_state: InitialStateFunction<'a, S>,
}
```

Additionally, `initialState` of the Decider is introduced to gain more control over the initial state of the Decider.

### Event-sourcing aggregate

[Event sourcing aggregate](src/aggregate.rs) is using/delegating a `Decider` to handle commands and produce new events.
It belongs to the
Application layer. In order to
handle the command, aggregate needs to fetch the current state (represented as a list/vector of events)
via `EventRepository.fetchEvents` async function, and then delegate the command to the decider which can produce new
events as
a result. Produced events are then stored via `EventRepository.save` async function.

It is a formalization of the event sourced information system.

### State-stored aggregate

[State stored aggregate](src/aggregate.rs) is using/delegating a `Decider` to handle commands and produce new state. It
belongs to the
Application layer. In order to
handle the command, aggregate needs to fetch the current state via `StateRepository.fetchState` async function first,
and then
delegate the command to the decider which can produce new state as a result. New state is then stored
via `StateRepository.save` async function.

It is a formalization of the state stored information system.


## View

`View`  is a datatype that represents the event handling algorithm, responsible for translating the events into
denormalized state, which is more adequate for querying. It belongs to the Domain layer. It is usually used to create
the view/query side of the CQRS pattern. Obviously, the command side of the CQRS is usually event-sourced aggregate.

It has two generic parameters `S`, `E`, representing the type of the values that `View` may contain or use.
`View` can be specialized for any type of `S`, `E` because these types do not affect its behavior.
`View` behaves the same for `E`=`Int` or `E`=`YourCustomType`, for example.

`View` is a pure domain component.

- `S` - State
- `E` - Event

```rust
pub struct View<'a, S: 'a, E: 'a> {
    pub evolve: EvolveFunction<'a, S, E>,
    pub initial_state: InitialStateFunction<'a, S>,
}
```

### Materialized View

[Materialized view](src/materialized_view.rs) is using/delegating a `View` to handle events of type `E` and to maintain
a state of denormalized
projection(s) as a
result. Essentially, it represents the query/view side of the CQRS pattern. It belongs to the Application layer.

In order to handle the event, materialized view needs to fetch the current state via `ViewStateRepository.fetchState`
suspending function first, and then delegate the event to the view, which can produce new state as a result. New state
is then stored via `ViewStateRepository.save` suspending function.

## Algebraic Data Types

In Rust, we can use ADTs to model our application's domain entities and relationships in a functional way, clearly defining the set of possible values and states.
Rust has two main types of ADTs: `enum` and `struct`. 

 - `enum` is used to define a type that can take on one of several possible variants - modeling a `sum/OR` type.
 - `struct` is used to express a type that has named fields - modeling a `product/AND` type.

ADTs will help with

 - representing the business domain in the code accurately
 - enforcing correctness
 - reducing the likelihood of bugs.

In FModel, we extensively use ADTs to model the data.

### `C` / Command / Intent to change the state of the system

```rust
// models Sum/Or type / multiple possible variants
pub enum OrderCommand {
    Create(CreateOrderCommand),
    Update(UpdateOrderCommand),
    Cancel(CancelOrderCommand),
}
// models Product/And type / a concrete variant, consisting of named fields
pub struct CreateOrderCommand {
    pub order_id: u32,
    pub customer_name: String,
    pub items: Vec<String>,
}
// models Product/And type / a concrete variant, consisting of named fields
pub struct UpdateOrderCommand {
    pub order_id: u32,
    pub new_items: Vec<String>,
}
// models Product/And type / a concrete variant, consisting of named fields
#[derive(Debug)]
pub struct CancelOrderCommand {
    pub order_id: u32,
}
```

### `E` / Event / Fact

```rust
// models Sum/Or type / multiple possible variants
pub enum OrderEvent {
    Created(OrderCreatedEvent),
    Updated(OrderUpdatedEvent),
    Cancelled(OrderCancelledEvent),
}
// models Product/And type / a concrete variant, consisting of named fields
pub struct OrderCreatedEvent {
    pub order_id: u32,
    pub customer_name: String,
    pub items: Vec<String>,
}
// models Product/And type / a concrete variant, consisting of named fields
pub struct OrderUpdatedEvent {
    pub order_id: u32,
    pub updated_items: Vec<String>,
}
// models Product/And type / a concrete variant, consisting of named fields
pub struct OrderCancelledEvent {
    pub order_id: u32,
}

```

### `S` / State / Current state of the system/aggregate/entity

```rust
struct OrderState {
    order_id: u32,
    customer_name: String,
    items: Vec<String>,
    is_cancelled: bool,
}
```

## Modeling the Behaviour of our domain

 - algebraic data types form the structure of our entities (commands, state, and events).
 - functions/lambda offers the algebra of manipulating the entities in a compositional manner, effectively modeling the behavior.

This leads to modularity in design and a clear separation of the entity’s structure and functions/behaviour of the entity.

Fmodel library offers generic and abstract components to specialize in for your specific case/expected behavior:

 - Decider - data type that represents the main decision-making algorithm.

```rust
fn decider<'a>() -> Decider<'a, OrderCommand, OrderState, OrderEvent> {
    Decider {
        // Your decision logic goes here.
        decide: Box::new(|command, state| match command {
            // Exhaustive pattern matching on the command
            OrderCommand::Create(create_cmd) => {
                vec![OrderEvent::Created(OrderCreatedEvent {
                    order_id: create_cmd.order_id,
                    customer_name: create_cmd.customer_name.to_owned(),
                    items: create_cmd.items.to_owned(),
                })]
            }
            OrderCommand::Update(update_cmd) => {
                // Your validation logic goes here
                if state.order_id == update_cmd.order_id {
                    vec![OrderEvent::Updated(OrderUpdatedEvent {
                        order_id: update_cmd.order_id,
                        updated_items: update_cmd.new_items.to_owned(),
                    })]
                } else {
                    // In case of validation failure, return empty list of events or error event
                    vec![]
                }
            }
            OrderCommand::Cancel(cancel_cmd) => {
                // Your validation logic goes here
                if state.order_id == cancel_cmd.order_id {
                    vec![OrderEvent::Cancelled(OrderCancelledEvent {
                        order_id: cancel_cmd.order_id,
                    })]
                } else {
                    // In case of validation failure, return empty list of events or error event
                    vec![]
                }
            }
        }),
        // Evolve the state based on the event(s)
        evolve: Box::new(|state, event| {
            let mut new_state = state.clone();
            // Exhaustive pattern matching on the event
            match event {
                OrderEvent::Created(created_event) => {
                    new_state.order_id = created_event.order_id;
                    new_state.customer_name = created_event.customer_name.to_owned();
                    new_state.items = created_event.items.to_owned();
                }
                OrderEvent::Updated(updated_event) => {
                    new_state.items = updated_event.updated_items.to_owned();
                }
                OrderEvent::Cancelled(_) => {
                    new_state.is_cancelled = true;
                }
            }
            new_state
        }),
        // Initial state
        initial_state: Box::new(|| OrderState {
            order_id: 0,
            customer_name: "".to_string(),
            items: Vec::new(),
            is_cancelled: false,
        }),
    }
}
```
- View - represents the event handling algorithm responsible for translating the events into the denormalized state, which is adequate for querying.

```rust
// The state of the view component
struct OrderViewState {
    order_id: u32,
    customer_name: String,
    items: Vec<String>,
    is_cancelled: bool,
}

fn view<'a>() -> View<'a, OrderViewState, OrderEvent> {
    View {
        // Evolve the state of the `view` based on the event(s)
        evolve: Box::new(|state, event| {
            let mut new_state = state.clone();
            // Exhaustive pattern matching on the event
            match event {
                OrderEvent::Created(created_event) => {
                    new_state.order_id = created_event.order_id;
                    new_state.customer_name = created_event.customer_name.to_owned();
                    new_state.items = created_event.items.to_owned();
                }
                OrderEvent::Updated(updated_event) => {
                    new_state.items = updated_event.updated_items.to_owned();
                }
                OrderEvent::Cancelled(_) => {
                    new_state.is_cancelled = true;
                }
            }
            new_state
        }),
        // Initial state
        initial_state: Box::new(|| OrderViewState {
            order_id: 0,
            customer_name: "".to_string(),
            items: Vec::new(),
            is_cancelled: false,
        }),
    }
}

```

## The Application layer

The logic execution will be orchestrated by the outside components that use the domain components (decider, view) to do the computations. These components will be responsible for fetching and saving the data (repositories).


The arrows in the image (adapters->application->domain) show the direction of the dependency. Notice that all dependencies point inward and that Domain does not depend on anybody or anything.

Pushing these decisions from the core domain model is very valuable. Being able to postpone them is a sign of good architecture.

**Event-sourcing aggregate**

```rust
    let repository = InMemoryOrderEventRepository::new();
    let aggregate = EventSourcedAggregate::new(repository, decider());

    let command = OrderCommand::Create(CreateOrderCommand {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });

    let result = aggregate.handle(&command).await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        [(
            OrderEvent::Created(OrderCreatedEvent {
                order_id: 1,
                customer_name: "John Doe".to_string(),
                items: vec!["Item 1".to_string(), "Item 2".to_string()],
            }),
            0
        )]
    );
```

**State-stored aggregate**
```rust
    let repository = InMemoryOrderStateRepository::new();
    let aggregate = StateStoredAggregate::new(repository, decider());

    let command = OrderCommand::Create(CreateOrderCommand {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });
    let result = aggregate.handle(&command).await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        (
            OrderState {
                order_id: 1,
                customer_name: "John Doe".to_string(),
                items: vec!["Item 1".to_string(), "Item 2".to_string()],
                is_cancelled: false,
            },
            0
        )
    );
```

## Fearless Concurrency

Splitting the computation in your program into multiple threads to run multiple tasks at the same time can improve performance.
However, programming with threads has a reputation for being difficult. Rust’s type system and ownership model guarantee thread safety.

Example of the concurrent execution of the aggregate:

```rust
async fn es_test() {
    let repository = InMemoryOrderEventRepository::new();
    let aggregate = Arc::new(EventSourcedAggregate::new(repository, decider()));
    // Makes a clone of the Arc pointer. This creates another pointer to the same allocation, increasing the strong reference count.
    let aggregate2 = Arc::clone(&aggregate);

    // Lets spawn two threads to simulate two concurrent requests
    let handle1 = thread::spawn(|| async move {
        let command = OrderCommand::Create(CreateOrderCommand {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });

        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                OrderEvent::Created(OrderCreatedEvent {
                    order_id: 1,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 1".to_string(), "Item 2".to_string()],
                }),
                0
            )]
        );
        let command = OrderCommand::Update(UpdateOrderCommand {
            order_id: 1,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                OrderEvent::Updated(OrderUpdatedEvent {
                    order_id: 1,
                    updated_items: vec!["Item 3".to_string(), "Item 4".to_string()],
                }),
                1
            )]
        );
        let command = OrderCommand::Cancel(CancelOrderCommand { order_id: 1 });
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                OrderEvent::Cancelled(OrderCancelledEvent { order_id: 1 }),
                2
            )]
        );
    });

    let handle2 = thread::spawn(|| async move {
        let command = OrderCommand::Create(CreateOrderCommand {
            order_id: 2,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                OrderEvent::Created(OrderCreatedEvent {
                    order_id: 2,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 1".to_string(), "Item 2".to_string()],
                }),
                0
            )]
        );
        let command = OrderCommand::Update(UpdateOrderCommand {
            order_id: 2,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                OrderEvent::Updated(OrderUpdatedEvent {
                    order_id: 2,
                    updated_items: vec!["Item 3".to_string(), "Item 4".to_string()],
                }),
                1
            )]
        );
        let command = OrderCommand::Cancel(CancelOrderCommand { order_id: 2 });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                OrderEvent::Cancelled(OrderCancelledEvent { order_id: 2 }),
                2
            )]
        );
    });

    handle1.join().unwrap().await;
    handle2.join().unwrap().await;
}
```

You might wonder why all primitive types in Rust aren’t atomic and why standard library types aren’t implemented to use `Arc<T>` by default.
The reason is that thread safety comes with a performance penalty that you only want to pay when you really need to.

**You choose how to run it!** You can run it in a single-threaded, multi-threaded, or distributed environment.

## Install the crate as a dependency of your project

Run the following Cargo command in your project directory:
```shell
cargo add fmodel-rust
```
Or add the following line to your `Cargo.toml` file:

```toml
fmodel-rust = "0.8.0"
```

## Examples

- [Restaurant Demo - with Postgres](https://github.com/fraktalio/fmodel-rust-demo)
- [Gift Card Demo - with Axon](https://github.com/AxonIQ/axon-rust/tree/main/gift-card-rust)
- [Tests](tests)


## FModel in other languages

 - [FModel Kotlin](https://github.com/fraktalio/fmodel/)
 - [FModel TypeScript](https://github.com/fraktalio/fmodel-ts/)
 - [FModel Java](https://github.com/fraktalio/fmodel-java/)

## Further reading

- [https://doc.rust-lang.org/book/](https://doc.rust-lang.org/book/)
- [https://fraktalio.com/fmodel/](https://fraktalio.com/fmodel/)
- [https://fraktalio.com/fmodel-ts/](https://fraktalio.com/fmodel-ts/)
- [https://xebia.com/blog/functional-domain-modeling-in-rust-part-1/](https://xebia.com/blog/functional-domain-modeling-in-rust-part-1/)


## Credits

Special credits to `Jérémie Chassaing` for sharing his [research](https://www.youtube.com/watch?v=kgYGMVDHQHs)
and `Adam Dymitruk` for hosting the meetup.

---
Created with :heart: by [Fraktalio](https://fraktalio.com/)
