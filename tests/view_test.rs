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

#[test]
fn test() {
    let view: View<OrderViewState, OrderEvent> = view();
    let order_created_event = OrderEvent::Created(OrderCreatedEvent {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });
    let new_state = (view.evolve)(&(view.initial_state)(), &order_created_event);
    assert_eq!(
        new_state,
        OrderViewState {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
            is_cancelled: false,
        }
    );

    let order_updated_event = OrderEvent::Updated(OrderUpdatedEvent {
        order_id: 1,
        updated_items: vec![
            "Item 11".to_string(),
            "Item 22".to_string(),
            "Item 33".to_string(),
        ],
    });
    let new_state = (view.evolve)(&new_state, &order_updated_event);
    assert_eq!(
        new_state,
        OrderViewState {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec![
                "Item 11".to_string(),
                "Item 22".to_string(),
                "Item 33".to_string()
            ],
            is_cancelled: false,
        }
    );

    let order_canceled_event = OrderEvent::Cancelled(OrderCancelledEvent { order_id: 1 });
    let new_state = (view.evolve)(&new_state, &order_canceled_event);
    assert_eq!(
        new_state,
        OrderViewState {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec![
                "Item 11".to_string(),
                "Item 22".to_string(),
                "Item 33".to_string()
            ],
            is_cancelled: true,
        }
    );
}
