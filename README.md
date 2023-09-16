# **f`(`model`)`** - Functional Domain Modeling with Rust

>Publicly available at [crates.io](https://crates.io/crates/fmodel-rust) and [docs.rs](https://docs.rs/fmodel-rust/latest/fmodel_rust/)

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

## `type DecideFunction<'a, C, S, E> = Box<dyn Fn(&C, &S) -> Vec<E> + 'a + Send + Sync>`

On a higher level of abstraction, any information system is responsible for handling the intent (`Command`) and based on
the current `State`, produce new facts (`Events`):

- given the current `State/S` *on the input*,
- when `Command/C` is handled *on the input*,
- expect `flow` of new `Events/E` to be published/emitted *on the output*

## `type EvolveFunction<'a, S, E> = Box<dyn Fn(&S, &E) -> S + 'a + Send + Sync>`

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

## Testing the crate

Cargo is Rust’s build system and package manager. We can use it to test and build the library:

```shell
cargo test
```

## Install the crate as a dependency of your project

Run the following Cargo command in your project directory:
```shell
cargo add fmodel-rust
```
Or add the following line to your `Cargo.toml` file:

```toml
fmodel-rust = "0.1.0"
```

## FModel in other languages

 - [FModel Kotlin](https://github.com/fraktalio/fmodel/)
 - [FModel TypeScript](https://github.com/fraktalio/fmodel-ts/)
 - [FModel Java](https://github.com/fraktalio/fmodel-java/)

## Further reading

- [https://doc.rust-lang.org/book/](https://doc.rust-lang.org/book/)
- [https://fraktalio.com/fmodel/](https://fraktalio.com/fmodel/)
- [https://fraktalio.com/fmodel-ts/](https://fraktalio.com/fmodel-ts/)
- [https://xebia.com/blog/functional-domain-modeling-in-rust-part-1/](https://xebia.com/blog/functional-domain-modeling-in-rust-part-1/)
- [https://xebia.com/blog/functional-domain-modeling-in-rust-part-2/](https://xebia.com/blog/functional-domain-modeling-in-rust-part-2/)


## Credits

Special credits to `Jérémie Chassaing` for sharing his [research](https://www.youtube.com/watch?v=kgYGMVDHQHs)
and `Adam Dymitruk` for hosting the meetup.

---
Created with :heart: by [Fraktalio](https://fraktalio.com/)