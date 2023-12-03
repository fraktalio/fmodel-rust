use fmodel_rust::view::{View, ViewStateComputation};
use fmodel_rust::view_combined::combine;
use fmodel_rust::{Sum::First as Order, Sum::Second as Shipment};

use crate::api::{
    OrderCancelledEvent, OrderCreatedEvent, OrderEvent, OrderUpdatedEvent, ShipmentCreatedEvent,
    ShipmentEvent,
};

mod api;

#[derive(Debug, Clone, PartialEq)]
struct OrderViewState {
    order_id: u32,
    customer_name: String,
    items: Vec<String>,
    is_cancelled: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct ShipmentViewState {
    shipment_id: u32,
    order_id: u32,
    customer_name: String,
    items: Vec<String>,
}

fn order_view<'a>() -> View<'a, OrderViewState, OrderEvent> {
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

fn shipment_view<'a>() -> View<'a, ShipmentViewState, ShipmentEvent> {
    View {
        evolve: Box::new(|state, event| {
            let mut new_state = state.clone();
            match event {
                ShipmentEvent::Created(created_event) => {
                    new_state.shipment_id = created_event.shipment_id;
                    new_state.order_id = created_event.order_id;
                    new_state.customer_name = created_event.customer_name.to_owned();
                    new_state.items = created_event.items.to_owned();
                }
            }
            new_state
        }),
        initial_state: Box::new(|| ShipmentViewState {
            shipment_id: 0,
            order_id: 0,
            customer_name: "".to_string(),
            items: Vec::new(),
        }),
    }
}

#[test]
fn test() {
    let order_view: View<OrderViewState, OrderEvent> = order_view();
    let shipment_view: View<ShipmentViewState, ShipmentEvent> = shipment_view();
    let combined_view = combine(&order_view, &shipment_view);

    let order_created_event = OrderEvent::Created(OrderCreatedEvent {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });
    let shipment_created_event = ShipmentEvent::Created(ShipmentCreatedEvent {
        shipment_id: 1,
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });

    let new_state = order_view.compute_new_state(None, &[&order_created_event]);
    assert_eq!(
        new_state,
        OrderViewState {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
            is_cancelled: false,
        }
    );
    let new_combined_state2 = combined_view.compute_new_state(None, &[&Order(order_created_event)]);
    assert_eq!(
        new_combined_state2,
        (
            OrderViewState {
                order_id: 1,
                customer_name: "John Doe".to_string(),
                items: vec!["Item 1".to_string(), "Item 2".to_string()],
                is_cancelled: false,
            },
            ShipmentViewState {
                shipment_id: 0,
                order_id: 0,
                customer_name: "".to_string(),
                items: Vec::new(),
            }
        )
    );

    let new_combined_state3 =
        combined_view.compute_new_state(None, &[&Shipment(shipment_created_event)]);
    assert_eq!(
        new_combined_state3,
        (
            OrderViewState {
                order_id: 0,
                customer_name: "".to_string(),
                items: Vec::new(),
                is_cancelled: false,
            },
            ShipmentViewState {
                shipment_id: 1,
                order_id: 1,
                customer_name: "John Doe".to_string(),
                items: vec!["Item 1".to_string(), "Item 2".to_string()],
            }
        )
    );

    let order_updated_event = OrderEvent::Updated(OrderUpdatedEvent {
        order_id: 1,
        updated_items: vec![
            "Item 11".to_string(),
            "Item 22".to_string(),
            "Item 33".to_string(),
        ],
    });
    let new_state = order_view.compute_new_state(Some(new_state), &[&order_updated_event]);
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
    let new_state = order_view.compute_new_state(Some(new_state), &[&order_canceled_event]);
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
