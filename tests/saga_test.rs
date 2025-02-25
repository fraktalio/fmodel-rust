use fmodel_rust::saga::{ActionComputation, Saga};

use crate::api::{
    CreateShipmentCommand, OrderCommand, OrderCreatedEvent, OrderEvent, ShipmentCommand,
    UpdateOrderCommand,
};
use crate::application::{sum_to_command, Command, Event};

mod api;
mod application;

fn order_saga<'a>() -> Saga<'a, OrderEvent, ShipmentCommand> {
    Saga {
        react: Box::new(|event| match event {
            OrderEvent::Created(evt) => {
                vec![ShipmentCommand::Create(CreateShipmentCommand {
                    shipment_id: evt.order_id,
                    order_id: evt.order_id,
                    customer_name: evt.customer_name.to_owned(),
                    items: evt.items.to_owned(),
                })]
            }
            OrderEvent::Updated(_) => {
                vec![]
            }
            OrderEvent::Cancelled(_) => {
                vec![]
            }
        }),
    }
}

fn order_saga_2<'a>() -> Saga<'a, Event, ShipmentCommand> {
    Saga {
        react: Box::new(|event| match event {
            Event::OrderCreated(evt) => {
                vec![ShipmentCommand::Create(CreateShipmentCommand {
                    shipment_id: evt.order_id,
                    order_id: evt.order_id,
                    customer_name: evt.customer_name.to_owned(),
                    items: evt.items.to_owned(),
                })]
            }
            Event::OrderUpdated(_) => {
                vec![]
            }
            Event::OrderCancelled(_) => {
                vec![]
            }
            Event::ShipmentCreated(_) => {
                vec![]
            }
        }),
    }
}

fn shipment_saga_2<'a>() -> Saga<'a, Event, OrderCommand> {
    Saga {
        react: Box::new(|event| match event {
            Event::ShipmentCreated(evt) => {
                vec![OrderCommand::Update(UpdateOrderCommand {
                    order_id: evt.order_id,
                    new_items: evt.items.to_owned(),
                })]
            }

            Event::OrderCreated(_) => {
                vec![]
            }
            Event::OrderUpdated(_) => {
                vec![]
            }
            Event::OrderCancelled(_) => {
                vec![]
            }
        }),
    }
}

#[test]
fn test() {
    let order_saga: Saga<OrderEvent, ShipmentCommand> = order_saga();
    let order_saga_2: Saga<Event, ShipmentCommand> = crate::order_saga_2();
    let shipment_saga_2: Saga<Event, OrderCommand> = crate::shipment_saga_2();
    let merged_saga = order_saga_2
        .merge(shipment_saga_2)
        .map_action(sum_to_command);

    let order_created_event = OrderEvent::Created(OrderCreatedEvent {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });
    let commands = order_saga.compute_new_actions(&order_created_event);
    assert_eq!(
        commands,
        [ShipmentCommand::Create(CreateShipmentCommand {
            shipment_id: 1,
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        })]
    );
    let order_created_event2 = Event::OrderCreated(OrderCreatedEvent {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });

    let merged_commands = merged_saga.compute_new_actions(&order_created_event2);
    assert_eq!(
        merged_commands,
        [Command::ShipmentCreate(CreateShipmentCommand {
            shipment_id: 1,
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        })]
    );
}
