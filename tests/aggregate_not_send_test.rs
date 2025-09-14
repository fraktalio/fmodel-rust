#![cfg(feature = "not-send-futures")]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use fmodel_rust::aggregate::{
    EventRepository, EventSourcedAggregate, StateRepository, StateStoredAggregate,
};
use fmodel_rust::decider::Decider;
use fmodel_rust::Identifier;
use tokio::task;

use crate::api::{CreateOrderCommand, OrderCommand, OrderCreatedEvent, OrderEvent, OrderState};
use crate::application::AggregateError;

mod api;
mod application;

/// In-memory event repository for testing
struct InMemoryOrderEventRepository {
    events: RefCell<Vec<(OrderEvent, i32)>>,
}

impl InMemoryOrderEventRepository {
    fn new() -> Self {
        Self {
            events: RefCell::new(vec![]),
        }
    }
}

impl EventRepository<OrderCommand, OrderEvent, i32, AggregateError>
    for InMemoryOrderEventRepository
{
    async fn fetch_events(
        &self,
        command: &OrderCommand,
    ) -> Result<Vec<(OrderEvent, i32)>, AggregateError> {
        let events = self.events.borrow(); // borrow the Vec immutably
        Ok(events
            .iter()
            .cloned()
            .filter(|(e, _)| e.identifier() == command.identifier())
            .collect())
    }

    async fn save(&self, events: &[OrderEvent]) -> Result<Vec<(OrderEvent, i32)>, AggregateError> {
        // Step 1: compute latest version without holding mutable borrow
        let latest_version = {
            let events_vec = self.events.borrow(); // immutable borrow
            events
                .first()
                .and_then(|first_event| {
                    events_vec
                        .iter()
                        .filter(|(e, _)| e.identifier() == first_event.identifier())
                        .map(|(_, v)| *v)
                        .last()
                })
                .unwrap_or(-1)
        };

        // Step 2: build new events
        let mut current_version = latest_version;
        let new_events: Vec<(OrderEvent, i32)> = events
            .iter()
            .map(|event| {
                current_version += 1;
                (event.clone(), current_version)
            })
            .collect();

        // Step 3: commit them under a mutable borrow
        self.events.borrow_mut().extend_from_slice(&new_events);

        Ok(new_events)
    }

    async fn version_provider(&self, event: &OrderEvent) -> Result<Option<i32>, AggregateError> {
        let events = self.events.borrow();
        Ok(events
            .iter()
            .filter(|(e, _)| e.identifier() == event.identifier())
            .map(|(_, v)| *v)
            .last())
    }
}

/// In-memory state repository for testing
struct InMemoryOrderStateRepository {
    states: RefCell<HashMap<u32, (OrderState, i32)>>,
}
impl InMemoryOrderStateRepository {
    fn new() -> Self {
        Self {
            states: RefCell::new(HashMap::new()),
        }
    }
}

impl StateRepository<OrderCommand, OrderState, i32, AggregateError>
    for InMemoryOrderStateRepository
{
    async fn fetch_state(
        &self,
        command: &OrderCommand,
    ) -> Result<Option<(OrderState, i32)>, AggregateError> {
        let states = self.states.borrow();
        Ok(states
            .get(&command.identifier().parse::<u32>().unwrap())
            .cloned())
    }

    async fn save(
        &self,
        state: &OrderState,
        version: &Option<i32>,
    ) -> Result<(OrderState, i32), AggregateError> {
        let mut states = self.states.borrow_mut();
        let version = version.unwrap_or(0);
        states.insert(state.order_id, (state.clone(), version + 1));
        Ok((state.clone(), version))
    }
}

/// Example decider
fn decider<'a>() -> Decider<'a, OrderCommand, OrderState, OrderEvent> {
    Decider {
        decide: Box::new(|command, _state| match command {
            OrderCommand::Create(cmd) => Ok(vec![OrderEvent::Created(OrderCreatedEvent {
                order_id: cmd.order_id,
                customer_name: cmd.customer_name.clone(),
                items: cmd.items.clone(),
            })]),
            OrderCommand::Update(_cmd) => Ok(vec![]),
            OrderCommand::Cancel(_cmd) => Ok(vec![]),
        }),
        evolve: Box::new(|state, _event| state.clone()),
        initial_state: Box::new(|| OrderState {
            order_id: 0,
            customer_name: "".to_string(),
            items: vec![],
            is_cancelled: false,
        }),
    }
}

#[tokio::test]
async fn es_test_not_send() {
    let repository = InMemoryOrderEventRepository::new();

    let aggregate = EventSourcedAggregate::new(
        repository,
        decider().map_error(|()| AggregateError::DomainError("Decider error".to_string())),
    );

    // Does not require `move` and `Rc`
    // The futures are created and immediately consumed in the same function where aggregate lives, so the borrow checker can verify that aggregate lives long enough.
    let task1 = async {
        let command = OrderCommand::Create(CreateOrderCommand {
            order_id: 1,
            customer_name: "Alice".to_string(),
            items: vec!["Item1".to_string()],
        });
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
    };

    let task2 = async {
        let command = OrderCommand::Create(CreateOrderCommand {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
    };

    // Run both tasks concurrently on the same thread. Awaited immediately in the same scope
    // The futures are created and immediately consumed in the same function where aggregate lives, so the borrow checker can verify that aggregate lives long enough.
    tokio::join!(task1, task2);
}

#[tokio::test]
async fn es_test_not_send_with_spawn_local() {
    // Create a LocalSet to run !Send futures
    let local = task::LocalSet::new();

    local
        .run_until(async {
            let repository = InMemoryOrderEventRepository::new();
            let aggregate = Rc::new(EventSourcedAggregate::new(
                repository,
                decider().map_error(|()| AggregateError::DomainError("Decider error".to_string())),
            ));

            // Clone the Rc for each spawned task
            let aggregate1 = Rc::clone(&aggregate);
            let aggregate2 = Rc::clone(&aggregate);

            // Spawn the first task locally - requires `move` and `Rc`
            let handle1 = task::spawn_local(async move {
                let command = OrderCommand::Create(CreateOrderCommand {
                    order_id: 1,
                    customer_name: "Alice".to_string(),
                    items: vec!["Item1".to_string()],
                });
                let result = aggregate1.handle(&command).await;
                assert!(result.is_ok());
            });

            // Spawn the second task locally - also requires `move` and `Rc`
            let handle2 = task::spawn_local(async move {
                let command = OrderCommand::Create(CreateOrderCommand {
                    order_id: 2,
                    customer_name: "Bob".to_string(),
                    items: vec!["Item2".to_string()],
                });
                let result = aggregate2.handle(&command).await;
                assert!(result.is_ok());
            });

            // Wait for both tasks to complete
            let (result1, result2) = tokio::join!(handle1, handle2);

            // Check that both tasks completed successfully
            assert!(result1.is_ok());
            assert!(result2.is_ok());
        })
        .await;
}

#[tokio::test]
async fn ss_test_not_send() {
    let repository = InMemoryOrderStateRepository::new();
    let aggregate = StateStoredAggregate::new(
        repository,
        decider().map_error(|()| AggregateError::DomainError("Decider error".to_string())),
    );

    let command = OrderCommand::Create(CreateOrderCommand {
        order_id: 1,
        customer_name: "Alice".to_string(),
        items: vec!["Item1".to_string()],
    });

    let result = aggregate.handle(&command).await;
    assert!(result.is_ok());
}
