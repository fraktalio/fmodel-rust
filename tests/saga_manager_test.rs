#![cfg(not(feature = "not-send-futures"))]

use std::sync::Arc;
use std::thread;

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
    let saga_manager = Arc::new(SagaManager::new(SimpleActionPublisher::new(), saga()));
    let saga_manager1 = saga_manager.clone();
    let saga_manager2 = saga_manager.clone();

    let handle1 = thread::spawn(|| async move {
        let order_created_event = OrderEvent::Created(OrderCreatedEvent {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });

        let result = saga_manager1.handle(&order_created_event).await;
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
    });

    let handle2 = thread::spawn(|| async move {
        let order_created_event = OrderEvent::Created(OrderCreatedEvent {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 21".to_string(), "Item 22".to_string()],
        });

        let result = saga_manager2.handle(&order_created_event).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            vec![ShipmentCommand::Create(CreateShipmentCommand {
                shipment_id: 1,
                order_id: 1,
                customer_name: "John Doe".to_string(),
                items: vec!["Item 21".to_string(), "Item 22".to_string()],
            })]
        );
    });

    handle1.join().unwrap().await;
    handle2.join().unwrap().await;
}

#[cfg(feature = "not-send-futures")]
#[tokio::test]
async fn test2() {
    use std::rc::Rc;

    let saga_manager = Rc::new(SagaManager::new(SimpleActionPublisher::new(), saga()));
    let saga_manager1 = saga_manager.clone();
    let saga_manager2 = saga_manager.clone();

    let task1 = async move {
        let order_created_event = OrderEvent::Created(OrderCreatedEvent {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });

        let result = saga_manager1.handle(&order_created_event).await;
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
    };

    let task2 = async move {
        let order_created_event = OrderEvent::Created(OrderCreatedEvent {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 21".to_string(), "Item 22".to_string()],
        });

        let result = saga_manager2.handle(&order_created_event).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            vec![ShipmentCommand::Create(CreateShipmentCommand {
                shipment_id: 1,
                order_id: 1,
                customer_name: "John Doe".to_string(),
                items: vec!["Item 21".to_string(), "Item 22".to_string()],
            })]
        );
    };

    // Run both tasks concurrently on the same thread.
    let _ = tokio::join!(task1, task2);
}
