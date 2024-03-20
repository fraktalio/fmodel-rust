use fmodel_rust::saga::Saga;
use fmodel_rust::saga_manager::{ActionPublisher, SagaManager};

use crate::api::{CreateShipmentCommand, OrderCreatedEvent, OrderEvent, ShipmentCommand};
use crate::application::SagaManagerError;

mod api;
mod application;

fn saga<'a>() -> Saga<'a, OrderEvent, ShipmentCommand> {
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

/// Simple action publisher that just returns the action/command.
/// It is used for testing. In real life, it would publish the action/command to some external system. or to an aggregate that is able to handel the action/command.
struct SimpleActionPublisher;

impl SimpleActionPublisher {
    fn new() -> Self {
        SimpleActionPublisher {}
    }
}

impl ActionPublisher<ShipmentCommand, SagaManagerError> for SimpleActionPublisher {
    async fn publish(
        &self,
        action: &[ShipmentCommand],
    ) -> Result<Vec<ShipmentCommand>, SagaManagerError> {
        Ok(Vec::from(action))
    }
}

#[tokio::test]
async fn test() {
    let saga: Saga<OrderEvent, ShipmentCommand> = saga();
    let order_created_event = OrderEvent::Created(OrderCreatedEvent {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });

    let saga_manager = SagaManager::new(SimpleActionPublisher::new(), saga);
    let result = saga_manager.handle(&order_created_event).await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![ShipmentCommand::Create(CreateShipmentCommand {
            shipment_id: 1,
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        })]
    );
}
