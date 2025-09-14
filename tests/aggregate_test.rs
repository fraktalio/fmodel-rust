#![cfg(not(feature = "not-send-futures"))]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use fmodel_rust::aggregate::{
    EventRepository, EventSourcedAggregate, StateRepository, StateStoredAggregate,
};
use fmodel_rust::decider::Decider;
use fmodel_rust::Identifier;

use crate::api::{
    CreateOrderCommand, OrderCancelledEvent, OrderCommand, OrderCreatedEvent, OrderEvent,
    OrderState, OrderUpdatedEvent,
};
use crate::application::AggregateError;
use std::thread;

mod api;
mod application;

/// A simple in-memory event repository - infrastructure
struct InMemoryOrderEventRepository {
    events: RwLock<Vec<(OrderEvent, i32)>>,
}

impl InMemoryOrderEventRepository {
    fn new() -> Self {
        InMemoryOrderEventRepository {
            events: RwLock::new(vec![]),
        }
    }
}

/// Implementation of [EventRepository] for [InMemoryOrderEventRepository] - infrastructure
impl EventRepository<OrderCommand, OrderEvent, i32, AggregateError>
    for InMemoryOrderEventRepository
{
    async fn fetch_events(
        &self,
        command: &OrderCommand,
    ) -> Result<Vec<(OrderEvent, i32)>, AggregateError> {
        Ok(self
            .events
            .read()
            .unwrap()
            .clone()
            .into_iter()
            .filter(|(event, _)| event.identifier() == command.identifier())
            .collect())
    }

    async fn save(&self, events: &[OrderEvent]) -> Result<Vec<(OrderEvent, i32)>, AggregateError> {
        let mut latest_version = self
            .version_provider(events.first().unwrap())
            .await?
            .unwrap_or(-1);
        let events = events
            .iter()
            .map(|event| {
                latest_version += 1;
                (event.clone(), latest_version)
            })
            .collect::<Vec<(OrderEvent, i32)>>();

        self.events
            .write()
            .unwrap()
            .extend_from_slice(&events.clone());
        Ok(events)
    }

    async fn version_provider(&self, event: &OrderEvent) -> Result<Option<i32>, AggregateError> {
        Ok(self
            .events
            .read()
            .unwrap()
            .clone()
            .into_iter()
            .filter(|(e, _)| e.identifier() == event.identifier())
            .map(|(_, version)| version)
            .last())
    }
}

struct InMemoryOrderStateRepository {
    states: Mutex<HashMap<u32, (OrderState, i32)>>,
}

impl InMemoryOrderStateRepository {
    fn new() -> Self {
        InMemoryOrderStateRepository {
            states: Mutex::new(HashMap::new()),
        }
    }
}

// Implementation of [StateRepository] for [InMemoryOrderStateRepository]
impl StateRepository<OrderCommand, OrderState, i32, AggregateError>
    for InMemoryOrderStateRepository
{
    async fn fetch_state(
        &self,
        command: &OrderCommand,
    ) -> Result<Option<(OrderState, i32)>, AggregateError> {
        Ok(self
            .states
            .lock()
            .unwrap()
            .get(&command.identifier().parse::<u32>().unwrap())
            .cloned())
    }

    async fn save(
        &self,
        state: &OrderState,
        version: &Option<i32>,
    ) -> Result<(OrderState, i32), AggregateError> {
        let version = version.to_owned().unwrap_or(0);
        self.states
            .lock()
            .unwrap()
            .insert(state.order_id, (state.clone(), version + 1));
        Ok((state.clone(), version))
    }
}

/// Decider for the Order aggregate - Domain logic
fn decider<'a>() -> Decider<'a, OrderCommand, OrderState, OrderEvent> {
    Decider {
        decide: Box::new(|command, state| match command {
            OrderCommand::Create(cmd) => Ok(vec![OrderEvent::Created(OrderCreatedEvent {
                order_id: cmd.order_id,
                customer_name: cmd.customer_name.to_owned(),
                items: cmd.items.to_owned(),
            })]),
            OrderCommand::Update(cmd) => {
                if state.order_id == cmd.order_id {
                    Ok(vec![OrderEvent::Updated(OrderUpdatedEvent {
                        order_id: cmd.order_id,
                        updated_items: cmd.new_items.to_owned(),
                    })])
                } else {
                    Ok(vec![])
                }
            }
            OrderCommand::Cancel(cmd) => {
                if state.order_id == cmd.order_id {
                    Ok(vec![OrderEvent::Cancelled(OrderCancelledEvent {
                        order_id: cmd.order_id,
                    })])
                } else {
                    Ok(vec![])
                }
            }
        }),
        evolve: Box::new(|state, event| {
            let mut new_state = state.clone();
            match event {
                OrderEvent::Created(evt) => {
                    new_state.order_id = evt.order_id;
                    new_state.customer_name = evt.customer_name.to_owned();
                    new_state.items = evt.items.to_owned();
                }
                OrderEvent::Updated(evt) => {
                    new_state.items = evt.updated_items.to_owned();
                }
                OrderEvent::Cancelled(_) => {
                    new_state.is_cancelled = true;
                }
            }
            new_state
        }),
        initial_state: Box::new(|| OrderState {
            order_id: 0,
            customer_name: "".to_string(),
            items: Vec::new(),
            is_cancelled: false,
        }),
    }
}

#[tokio::test]
async fn es_test() {
    let repository = InMemoryOrderEventRepository::new();
    let aggregate = Arc::new(EventSourcedAggregate::new(
        repository,
        decider().map_error(|()| AggregateError::DomainError("Decider error".to_string())),
    ));
    let aggregate1 = Arc::clone(&aggregate);
    let aggregate2 = Arc::clone(&aggregate);

    // Spawn two async tasks instead of threads
    let handle1 = thread::spawn(|| async move {
        let command = OrderCommand::Create(CreateOrderCommand {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
        let result = aggregate1.handle(&command).await;
        assert!(result.is_ok());
    });

    let handle2 = thread::spawn(|| async move {
        let command = OrderCommand::Create(CreateOrderCommand {
            order_id: 2,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
    });

    handle1.join().unwrap().await;
    handle2.join().unwrap().await;
}

#[tokio::test]
async fn ss_test() {
    let repository = InMemoryOrderStateRepository::new();
    let aggregate = Arc::new(StateStoredAggregate::new(
        repository,
        decider().map_error(|()| AggregateError::DomainError("Decider error".to_string())),
    ));
    let aggregate1 = Arc::clone(&aggregate);
    let aggregate2 = Arc::clone(&aggregate);

    let handle1 = thread::spawn(|| async move {
        let command = OrderCommand::Create(CreateOrderCommand {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
        let result = aggregate1.handle(&command).await;
        assert!(result.is_ok());
    });

    let handle2 = thread::spawn(|| async move {
        let command = OrderCommand::Create(CreateOrderCommand {
            order_id: 2,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
    });

    handle1.join().unwrap().await;
    handle2.join().unwrap().await;
}
