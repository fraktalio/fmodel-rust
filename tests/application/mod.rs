use derive_more::Display;
use fmodel_rust::{Identifier, Sum};
use std::error::Error;

use crate::api::{
    CancelOrderCommand, CreateOrderCommand, CreateShipmentCommand, OrderCancelledEvent,
    OrderCommand, OrderCreatedEvent, OrderEvent, OrderUpdatedEvent, ShipmentCommand,
    ShipmentCreatedEvent, ShipmentEvent, UpdateOrderCommand,
};

/// The command enum for all the domain commands (shipment and order)
/// It is convenient to have a single enum for all the command variants in your system to make it easy to combine all deciders into a single decider
/// Consider exposing this API to the outside world, instead of exposing the Order or Shipment commands individually. It is on you!
#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum Command {
    ShipmentCreate(CreateShipmentCommand),
    OrderCreate(CreateOrderCommand),
    OrderUpdate(UpdateOrderCommand),
    OrderCancel(CancelOrderCommand),
}

/// A mapping function to contra map the domain command to the inconvenient Sum<OrderCommand, ShipmentCommand>
#[allow(dead_code)]
pub fn command_from_sum(command: &Command) -> Sum<OrderCommand, ShipmentCommand> {
    match command {
        Command::ShipmentCreate(c) => Sum::Second(ShipmentCommand::Create(c.to_owned())),
        Command::OrderCreate(c) => Sum::First(OrderCommand::Create(c.to_owned())),
        Command::OrderUpdate(c) => Sum::First(OrderCommand::Update(c.to_owned())),
        Command::OrderCancel(c) => Sum::First(OrderCommand::Cancel(c.to_owned())),
    }
}
/// A mapping function to map the inconvenient Sum<OrderCommand, ShipmentCommand> to the domain command
#[allow(dead_code)]
pub fn sum_to_command(command: &Sum<OrderCommand, ShipmentCommand>) -> Command {
    match command {
        Sum::First(c) => match c {
            OrderCommand::Create(c) => Command::OrderCreate(c.to_owned()),
            OrderCommand::Update(c) => Command::OrderUpdate(c.to_owned()),
            OrderCommand::Cancel(c) => Command::OrderCancel(c.to_owned()),
        },
        Sum::Second(c) => match c {
            ShipmentCommand::Create(c) => Command::ShipmentCreate(c.to_owned()),
        },
    }
}
#[allow(dead_code)]
pub fn sum_to_command2(command: &Sum<ShipmentCommand, OrderCommand>) -> Command {
    match command {
        Sum::First(c) => match c {
            ShipmentCommand::Create(c) => Command::ShipmentCreate(c.to_owned()),
        },
        Sum::Second(c) => match c {
            OrderCommand::Create(c) => Command::OrderCreate(c.to_owned()),
            OrderCommand::Update(c) => Command::OrderUpdate(c.to_owned()),
            OrderCommand::Cancel(c) => Command::OrderCancel(c.to_owned()),
        },
    }
}

/// The event enum for all the domain events (shipment and order)
/// It is convenient to have a single enum for all the event variants in your system to make it easy to combine all deciders/sagas/views into a single decider/saga/view
/// Consider exposing this API to the outside world, instead of exposing the Order or Shipment events individually. It is on you!
#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum Event {
    ShipmentCreated(ShipmentCreatedEvent),
    OrderCreated(OrderCreatedEvent),
    OrderUpdated(OrderUpdatedEvent),
    OrderCancelled(OrderCancelledEvent),
}

/// A mapping function to contra map the domain event to the inconvenient Sum<OrderEvent, ShipmentEvent>
#[allow(dead_code)]
pub fn event_from_sum(event: &Event) -> Sum<OrderEvent, ShipmentEvent> {
    match event {
        Event::ShipmentCreated(c) => Sum::Second(ShipmentEvent::Created(c.to_owned())),
        Event::OrderCreated(c) => Sum::First(OrderEvent::Created(c.to_owned())),
        Event::OrderUpdated(c) => Sum::First(OrderEvent::Updated(c.to_owned())),
        Event::OrderCancelled(c) => Sum::First(OrderEvent::Cancelled(c.to_owned())),
    }
}
#[allow(dead_code)]
pub fn event_from_sum2(event: &Event) -> Sum<ShipmentEvent, OrderEvent> {
    match event {
        Event::ShipmentCreated(c) => Sum::First(ShipmentEvent::Created(c.to_owned())),
        Event::OrderCreated(c) => Sum::Second(OrderEvent::Created(c.to_owned())),
        Event::OrderUpdated(c) => Sum::Second(OrderEvent::Updated(c.to_owned())),
        Event::OrderCancelled(c) => Sum::Second(OrderEvent::Cancelled(c.to_owned())),
    }
}
/// A mapping function to map the inconvenient Sum<OrderEvent, ShipmentEvent> to the domain event
#[allow(dead_code)]
pub fn sum_to_event(event: &Sum<OrderEvent, ShipmentEvent>) -> Event {
    match event {
        Sum::First(e) => match e {
            OrderEvent::Created(c) => Event::OrderCreated(c.to_owned()),
            OrderEvent::Updated(c) => Event::OrderUpdated(c.to_owned()),
            OrderEvent::Cancelled(c) => Event::OrderCancelled(c.to_owned()),
        },
        Sum::Second(e) => match e {
            ShipmentEvent::Created(c) => Event::ShipmentCreated(c.to_owned()),
        },
    }
}

impl Identifier for Event {
    fn identifier(&self) -> String {
        match self {
            Event::ShipmentCreated(evt) => evt.shipment_id.to_string(),
            Event::OrderCreated(evt) => evt.order_id.to_string(),
            Event::OrderUpdated(evt) => evt.order_id.to_string(),
            Event::OrderCancelled(evt) => evt.order_id.to_string(),
        }
    }
}

impl Identifier for Command {
    fn identifier(&self) -> String {
        match self {
            Command::OrderCreate(cmd) => cmd.order_id.to_string(),
            Command::OrderUpdate(cmd) => cmd.order_id.to_string(),
            Command::OrderCancel(cmd) => cmd.order_id.to_string(),
            Command::ShipmentCreate(cmd) => cmd.shipment_id.to_string(),
        }
    }
}

/// Error type for the application/aggregate
#[derive(Debug, Display)]
#[allow(dead_code)]
pub enum AggregateError {
    DomainError(String),
    FetchEvents(String),
    SaveEvents(String),
    FetchState(String),
    SaveState(String),
}

impl Error for AggregateError {}

/// Error type for the application/materialized view
#[derive(Debug, Display)]
#[allow(dead_code)]
pub enum MaterializedViewError {
    FetchState(String),
    SaveState(String),
}

impl Error for MaterializedViewError {}

/// Error type for the saga manager
#[derive(Debug, Display)]
#[allow(dead_code)]
pub enum SagaManagerError {
    PublishAction(String),
}

impl Error for SagaManagerError {}
