use fmodel_rust::saga::Saga;
use fmodel_rust::saga_manager::{ActionPublisher, SagaManager};

use crate::api::{
    CreateShipmentCommand, OrderCommand, OrderCreatedEvent, ShipmentCommand, UpdateOrderCommand,
};
use crate::application::{sum_to_command2, Command, Event, SagaManagerError};

mod api;
mod application;

fn order_saga<'a>() -> Saga<'a, Event, ShipmentCommand> {
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

fn shipment_saga<'a>() -> Saga<'a, Event, OrderCommand> {
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
/// Simple action publisher that just returns the action/command.
/// It is used for testing. In real life, it would publish the action/command to some external system. or to an aggregate that is able to handel the action/command.
struct SimpleActionPublisher;

impl SimpleActionPublisher {
    fn new() -> Self {
        SimpleActionPublisher {}
    }
}

impl ActionPublisher<Command, SagaManagerError> for SimpleActionPublisher {
    async fn publish(&self, action: &[Command]) -> Result<Vec<Command>, SagaManagerError> {
        Ok(Vec::from(action))
    }
}

#[tokio::test]
async fn test() {
    let order_created_event = Event::OrderCreated(OrderCreatedEvent {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });

    let saga_manager = SagaManager::new(
        SimpleActionPublisher::new(),
        shipment_saga()
            .merge(order_saga())
            .map_action(sum_to_command2),
    );
    let result = saga_manager.handle(&order_created_event).await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![Command::ShipmentCreate(CreateShipmentCommand {
            shipment_id: 1,
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        })]
    );
}
