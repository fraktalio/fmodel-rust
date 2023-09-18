use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;

use async_trait::async_trait;
use derive_more::Display;

use fmodel_rust::materialized_view::{MaterializedView, ViewStateRepository};
use fmodel_rust::view::View;

use crate::api::{OrderCancelledEvent, OrderCreatedEvent, OrderEvent, OrderUpdatedEvent};

mod api;

#[derive(Debug, Clone, PartialEq)]
struct OrderViewState {
    order_id: u32,
    customer_name: String,
    items: Vec<String>,
    is_cancelled: bool,
}

fn view<'a>() -> View<'a, OrderViewState, OrderEvent> {
    View {
        evolve: Box::new(|state, event| {
            let mut new_state = state.clone();
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
        initial_state: Box::new(|| OrderViewState {
            order_id: 0,
            customer_name: "".to_string(),
            items: Vec::new(),
            is_cancelled: false,
        }),
    }
}

/// Error type for the application/materialized view
#[derive(Debug, Display)]
#[allow(dead_code)]
enum MaterializedViewError {
    FetchState(String),
    SaveState(String),
}

impl Error for MaterializedViewError {}

struct InMemoryViewOrderStateRepository {
    states: Mutex<HashMap<u32, OrderViewState>>,
}

impl InMemoryViewOrderStateRepository {
    fn new() -> Self {
        InMemoryViewOrderStateRepository {
            states: Mutex::new(HashMap::new()),
        }
    }
}

// Implementation of [ViewStateRepository] for [InMemoryViewOrderStateRepository]
#[async_trait]
impl ViewStateRepository<OrderEvent, OrderViewState, MaterializedViewError>
    for InMemoryViewOrderStateRepository
{
    async fn fetch_state(
        &self,
        event: &OrderEvent,
    ) -> Result<Option<OrderViewState>, MaterializedViewError> {
        Ok(self.states.lock().unwrap().get(&event.id()).cloned())
    }

    async fn save(&self, state: &OrderViewState) -> Result<OrderViewState, MaterializedViewError> {
        self.states
            .lock()
            .unwrap()
            .insert(state.order_id, state.clone());
        Ok(state.clone())
    }
}

#[tokio::test]
async fn test() {
    let repository = InMemoryViewOrderStateRepository::new();
    let materialized_view = MaterializedView::new(repository, view());
    let event = OrderEvent::Created(OrderCreatedEvent {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });
    let result = materialized_view.handle(&event).await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        OrderViewState {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
            is_cancelled: false,
        }
    );
    let event = OrderEvent::Updated(OrderUpdatedEvent {
        order_id: 1,
        updated_items: vec!["Item 3".to_string(), "Item 4".to_string()],
    });
    let result = materialized_view.handle(&event).await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        OrderViewState {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 3".to_string(), "Item 4".to_string()],
            is_cancelled: false,
        }
    );
    let event = OrderEvent::Cancelled(OrderCancelledEvent { order_id: 1 });
    let result = materialized_view.handle(&event).await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        OrderViewState {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 3".to_string(), "Item 4".to_string()],
            is_cancelled: true,
        }
    );
}
