use fmodel_rust::decider::{Decider, EventComputation, StateComputation};

use crate::api::{
    CancelOrderCommand, CreateOrderCommand, CreateShipmentCommand, OrderCancelledEvent,
    OrderCommand, OrderCreatedEvent, OrderEvent, OrderState, OrderUpdatedEvent, ShipmentCommand,
    ShipmentCreatedEvent, ShipmentEvent, ShipmentState,
};
use crate::application::Command::{OrderCreate, ShipmentCreate};
use crate::application::Event::{OrderCreated, ShipmentCreated};
use crate::application::{command_from_sum, event_from_sum, sum_to_event, Command, Event};

mod api;
mod application;

fn order_decider<'a>() -> Decider<'a, OrderCommand, OrderState, OrderEvent> {
    Decider {
        decide: Box::new(|command, state| match command {
            OrderCommand::Create(cmd) => {
                vec![OrderEvent::Created(OrderCreatedEvent {
                    order_id: cmd.order_id,
                    customer_name: cmd.customer_name.to_owned(),
                    items: cmd.items.to_owned(),
                })]
            }
            OrderCommand::Update(cmd) => {
                if state.order_id == cmd.order_id {
                    vec![OrderEvent::Updated(OrderUpdatedEvent {
                        order_id: cmd.order_id,
                        updated_items: cmd.new_items.to_owned(),
                    })]
                } else {
                    vec![]
                }
            }
            OrderCommand::Cancel(cmd) => {
                if state.order_id == cmd.order_id {
                    vec![OrderEvent::Cancelled(OrderCancelledEvent {
                        order_id: cmd.order_id,
                    })]
                } else {
                    vec![]
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

fn shipment_decider<'a>() -> Decider<'a, ShipmentCommand, ShipmentState, ShipmentEvent> {
    Decider {
        decide: Box::new(|command, _state| match command {
            ShipmentCommand::Create(cmd) => {
                vec![ShipmentEvent::Created(ShipmentCreatedEvent {
                    shipment_id: cmd.shipment_id,
                    order_id: cmd.order_id,
                    customer_name: cmd.customer_name.to_owned(),
                    items: cmd.items.to_owned(),
                })]
            }
        }),
        evolve: Box::new(|state, event| {
            let mut new_state = state.clone();
            match event {
                ShipmentEvent::Created(evt) => {
                    new_state.shipment_id = evt.shipment_id;
                    new_state.order_id = evt.order_id;
                    new_state.customer_name = evt.customer_name.to_owned();
                    new_state.items = evt.items.to_owned();
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
    let order_decider_clone: Decider<OrderCommand, OrderState, OrderEvent> = crate::order_decider();
    let shipment_decider: Decider<ShipmentCommand, ShipmentState, ShipmentEvent> =
        shipment_decider();
    let combined_decider: Decider<Command, (OrderState, ShipmentState), Event> =
        order_decider_clone
            .combine(shipment_decider) // Decider<Sum<OrderCommand, ShipmentCommand>, (OrderState, ShipmentState), Sum<OrderEvent, ShipmentEvent>>
            .map_command(&command_from_sum) // Decider<Command, (OrderState, ShipmentState), Sum<OrderEvent, ShipmentEvent>>
            .map_event(&event_from_sum, &sum_to_event); // Decider<Command, (OrderState, ShipmentState), Event>

    let create_order_command = CreateOrderCommand {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    };

    let create_shipment_command = CreateShipmentCommand {
        shipment_id: 1,
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    };

    // Test the OrderDecider
    let new_events =
        order_decider.compute_new_events(&[], &OrderCommand::Create(create_order_command.clone()));
    assert_eq!(
        new_events,
        [OrderEvent::Created(OrderCreatedEvent {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        })]
    );
    // Test the Decider that combines OrderDecider and ShipmentDecider and can handle both OrderCommand and ShipmentCommand and produce Event
    let new_events2 =
        combined_decider.compute_new_events(&[], &OrderCreate(create_order_command.clone()));
    assert_eq!(
        new_events2,
        [OrderCreated(OrderCreatedEvent {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        })]
    );
    // Test the Decider that combines OrderDecider and ShipmentDecider and can handle both OrderCommand and ShipmentCommand and produce Event
    let new_events3 =
        combined_decider.compute_new_events(&[], &ShipmentCreate(create_shipment_command.clone()));
    assert_eq!(
        new_events3,
        [ShipmentCreated(ShipmentCreatedEvent {
            shipment_id: 1,
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        })]
    );

    // Test the OrderDecider
    let new_state =
        order_decider.compute_new_state(None, &OrderCommand::Create(create_order_command.clone()));
    assert_eq!(
        new_state,
        OrderState {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
            is_cancelled: false,
        }
    );
    // Test the Decider that combines OrderDecider and ShipmentDecider and can handle both OrderCommand and ShipmentCommand and produce a tuple of (OrderState, ShipmentState)
    let new_state2 =
        combined_decider.compute_new_state(None, &ShipmentCreate(create_shipment_command.clone()));
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

    // Test the OrderDecider
    let cancel_command = OrderCommand::Cancel(CancelOrderCommand { order_id: 1 });
    let new_events = order_decider.compute_new_events(&new_events, &cancel_command);
    assert_eq!(
        new_events,
        [OrderEvent::Cancelled(OrderCancelledEvent { order_id: 1 })]
    );
    // Test the OrderDecider
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
