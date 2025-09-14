#![cfg(not(feature = "not-send-futures"))]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;

use fmodel_rust::materialized_view::{MaterializedView, ViewStateRepository};
use fmodel_rust::view::View;
use fmodel_rust::Identifier;

use crate::api::{
    OrderCancelledEvent, OrderCreatedEvent, OrderEvent, OrderUpdatedEvent, OrderViewState,
};
use crate::application::MaterializedViewError;

mod api;
mod application;

fn view<'a>() -> View<'a, OrderViewState, OrderEvent> {
    View {
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
        initial_state: Box::new(|| OrderViewState {
            order_id: 0,
            customer_name: "".to_string(),
            items: Vec::new(),
            is_cancelled: false,
        }),
    }
}

struct InMemoryViewOrderStateRepository {
    states: RwLock<HashMap<u32, OrderViewState>>,
}

impl InMemoryViewOrderStateRepository {
    fn new() -> Self {
        InMemoryViewOrderStateRepository {
            states: RwLock::new(HashMap::new()),
        }
    }
}

// Implementation of [ViewStateRepository] for [InMemoryViewOrderStateRepository]
impl ViewStateRepository<OrderEvent, OrderViewState, MaterializedViewError>
    for InMemoryViewOrderStateRepository
{
    async fn fetch_state(
        &self,
        event: &OrderEvent,
    ) -> Result<Option<OrderViewState>, MaterializedViewError> {
        Ok(self
            .states
            .read()
            .unwrap()
            .get(&event.identifier().parse::<u32>().unwrap())
            .cloned())
    }

    async fn save(&self, state: &OrderViewState) -> Result<OrderViewState, MaterializedViewError> {
        self.states
            .write()
            .unwrap()
            .insert(state.order_id, state.clone());
        Ok(state.clone())
    }
}

#[tokio::test]
async fn test() {
    let repository = InMemoryViewOrderStateRepository::new();
    let materialized_view = Arc::new(MaterializedView::new(repository, view()));
    let materialized_view1 = Arc::clone(&materialized_view);
    let materialized_view2 = Arc::clone(&materialized_view);

    // Let's spawn two threads to simulate two concurrent requests
    let handle1 = thread::spawn(|| async move {
        let event = OrderEvent::Created(OrderCreatedEvent {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
        let result = materialized_view1.handle(&event).await;
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
        let result = materialized_view1.handle(&event).await;
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
        let result = materialized_view1.handle(&event).await;
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
    });

    let handle2 = thread::spawn(|| async move {
        let event = OrderEvent::Created(OrderCreatedEvent {
            order_id: 2,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
        let result = materialized_view2.handle(&event).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            OrderViewState {
                order_id: 2,
                customer_name: "John Doe".to_string(),
                items: vec!["Item 1".to_string(), "Item 2".to_string()],
                is_cancelled: false,
            }
        );
        let event = OrderEvent::Updated(OrderUpdatedEvent {
            order_id: 2,
            updated_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
        let result = materialized_view2.handle(&event).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            OrderViewState {
                order_id: 2,
                customer_name: "John Doe".to_string(),
                items: vec!["Item 3".to_string(), "Item 4".to_string()],
                is_cancelled: false,
            }
        );
        let event = OrderEvent::Cancelled(OrderCancelledEvent { order_id: 2 });
        let result = materialized_view2.handle(&event).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            OrderViewState {
                order_id: 2,
                customer_name: "John Doe".to_string(),
                items: vec!["Item 3".to_string(), "Item 4".to_string()],
                is_cancelled: true,
            }
        );
    });

    handle1.join().unwrap().await;
    handle2.join().unwrap().await;
}
