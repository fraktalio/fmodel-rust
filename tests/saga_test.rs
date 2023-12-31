use fmodel_rust::saga::{ActionComputation, Saga};
use fmodel_rust::Sum;

use crate::api::{
    CreateShipmentCommand, OrderCommand, OrderCreatedEvent, OrderEvent, ShipmentCommand,
    ShipmentEvent,
};

mod api;

fn order_saga<'a>() -> Saga<'a, OrderEvent, ShipmentCommand> {
    Saga {
        react: Box::new(|event| match event {
            OrderEvent::Created(created_event) => {
                vec![ShipmentCommand::Create(CreateShipmentCommand {
                    shipment_id: created_event.order_id,
                    order_id: created_event.order_id,
                    customer_name: created_event.customer_name.to_owned(),
                    items: created_event.items.to_owned(),
                })]
            }
            OrderEvent::Updated(_updated_event) => {
                vec![]
            }
            OrderEvent::Cancelled(_cancelled_event) => {
                vec![]
            }
        }),
    }
}

fn shipment_saga<'a>() -> Saga<'a, ShipmentEvent, OrderCommand> {
    Saga {
        react: Box::new(|event| match event {
            ShipmentEvent::Created(created_event) => {
                vec![OrderCommand::Update(api::UpdateOrderCommand {
                    order_id: created_event.order_id,
                    new_items: created_event.items.to_owned(),
                })]
            }
        }),
    }
}

#[test]
fn test() {
    let order_saga: Saga<OrderEvent, ShipmentCommand> = order_saga();
    let order_saga2: Saga<OrderEvent, ShipmentCommand> = crate::order_saga();
    let shipment_saga: Saga<ShipmentEvent, OrderCommand> = shipment_saga();
    let combined_saga = order_saga2.combine(shipment_saga);
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
    let combined_commands = combined_saga.compute_new_actions(&Sum::First(order_created_event));
    assert_eq!(
        combined_commands,
        [Sum::Second(ShipmentCommand::Create(
            CreateShipmentCommand {
                shipment_id: 1,
                order_id: 1,
                customer_name: "John Doe".to_string(),
                items: vec!["Item 1".to_string(), "Item 2".to_string()],
            }
        ))]
    );
}
