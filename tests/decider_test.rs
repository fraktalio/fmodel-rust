use fmodel_rust::decider::{Decider, EventComputation, StateComputation};
use fmodel_rust::{Sum::First as Order, Sum::Second as Shipment};

use crate::api::{
    CancelOrderCommand, CreateOrderCommand, CreateShipmentCommand, OrderCancelledEvent,
    OrderCommand, OrderCreatedEvent, OrderEvent, OrderUpdatedEvent, ShipmentCommand,
    ShipmentCreatedEvent, ShipmentEvent,
};

mod api;

#[derive(Debug, Clone, PartialEq)]
struct OrderState {
    order_id: u32,
    customer_name: String,
    items: Vec<String>,
    is_cancelled: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct ShipmentState {
    shipment_id: u32,
    order_id: u32,
    customer_name: String,
    items: Vec<String>,
}

fn order_decider<'a>() -> Decider<'a, OrderCommand, OrderState, OrderEvent> {
    Decider {
        decide: Box::new(|command, state| match command {
            OrderCommand::Create(create_cmd) => {
                vec![OrderEvent::Created(OrderCreatedEvent {
                    order_id: create_cmd.order_id,
                    customer_name: create_cmd.customer_name.to_owned(),
                    items: create_cmd.items.to_owned(),
                })]
            }
            OrderCommand::Update(update_cmd) => {
                if state.order_id == update_cmd.order_id {
                    vec![OrderEvent::Updated(OrderUpdatedEvent {
                        order_id: update_cmd.order_id,
                        updated_items: update_cmd.new_items.to_owned(),
                    })]
                } else {
                    vec![]
                }
            }
            OrderCommand::Cancel(cancel_cmd) => {
                if state.order_id == cancel_cmd.order_id {
                    vec![OrderEvent::Cancelled(OrderCancelledEvent {
                        order_id: cancel_cmd.order_id,
                    })]
                } else {
                    vec![]
                }
            }
        }),
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
        initial_state: Box::new(|| OrderState {
            order_id: 0,
            customer_name: "".to_string(),
            items: Vec::new(),
            is_cancelled: false,
        }),
    }
}

fn shipment_decider<'a>() -> Decider<'a, ShipmentCommand, ShipmentState, ShipmentEvent> {
    Decider {
        decide: Box::new(|command, _state| match command {
            ShipmentCommand::Create(create_cmd) => {
                vec![ShipmentEvent::Created(ShipmentCreatedEvent {
                    shipment_id: create_cmd.shipment_id,
                    order_id: create_cmd.order_id,
                    customer_name: create_cmd.customer_name.to_owned(),
                    items: create_cmd.items.to_owned(),
                })]
            }
        }),
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
        initial_state: Box::new(|| ShipmentState {
            shipment_id: 0,
            order_id: 0,
            customer_name: "".to_string(),
            items: Vec::new(),
        }),
    }
}

#[test]
fn test() {
    let order_decider: Decider<OrderCommand, OrderState, OrderEvent> = order_decider();
    let order_decider2: Decider<OrderCommand, OrderState, OrderEvent> = crate::order_decider();
    let shpiment_decider2: Decider<ShipmentCommand, ShipmentState, ShipmentEvent> =
        shipment_decider();
    let combined_decider = order_decider2.combine(shpiment_decider2);

    let create_order_command = OrderCommand::Create(CreateOrderCommand {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });
    let create_shipment_command = ShipmentCommand::Create(CreateShipmentCommand {
        shipment_id: 1,
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });
    let new_events = order_decider.compute_new_events(&[], &create_order_command);
    assert_eq!(
        new_events,
        [OrderEvent::Created(OrderCreatedEvent {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        })]
    );
    let new_events2 =
        combined_decider.compute_new_events(&[], &Order(create_order_command.clone()));
    assert_eq!(
        new_events2,
        [Order(OrderEvent::Created(OrderCreatedEvent {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        }))]
    );
    let new_events3 =
        combined_decider.compute_new_events(&[], &Shipment(create_shipment_command.clone()));
    assert_eq!(
        new_events3,
        [Shipment(ShipmentEvent::Created(ShipmentCreatedEvent {
            shipment_id: 1,
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        }))]
    );

    let new_state = order_decider.compute_new_state(None, &create_order_command);
    assert_eq!(
        new_state,
        OrderState {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
            is_cancelled: false,
        }
    );
    let new_state2 =
        combined_decider.compute_new_state(None, &Shipment(create_shipment_command.clone()));
    assert_eq!(
        new_state2,
        (
            OrderState {
                order_id: 0,
                customer_name: "".to_string(),
                items: Vec::new(),
                is_cancelled: false,
            },
            ShipmentState {
                shipment_id: 1,
                order_id: 1,
                customer_name: "John Doe".to_string(),
                items: vec!["Item 1".to_string(), "Item 2".to_string()],
            }
        )
    );

    let cancel_command = OrderCommand::Cancel(CancelOrderCommand { order_id: 1 });
    let new_events = order_decider.compute_new_events(&new_events, &cancel_command);
    assert_eq!(
        new_events,
        [OrderEvent::Cancelled(OrderCancelledEvent { order_id: 1 })]
    );
    let new_state = order_decider.compute_new_state(Some(new_state), &cancel_command);
    assert_eq!(
        new_state,
        OrderState {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
            is_cancelled: true,
        }
    );
}
