use std::marker::PhantomData;

use async_trait::async_trait;

use crate::saga::ActionComputation;

/// Publishes the action/command to some external system.
///
/// Generic parameter:
///
/// - `A`. - action
/// - `Error` - error
#[async_trait]
pub trait ActionPublisher<A, Error> {
    /// Publishes the action/command to some external system, returning either the actions that are successfully published or error.
    async fn publish(&self, action: &[A]) -> Result<Vec<A>, Error>;
}

/// Saga Manager.
///
/// It is using a `Saga` to react to the action result and to publish the new actions.
/// It is using an [ActionPublisher] to publish the new actions.
///
/// Generic parameters:
/// - `A` - Action / Command
/// - `AR` - Action Result / Event
/// - `Publisher` - Action Publisher
/// - `Error` - Error
pub struct SagaManager<A, AR, Publisher, Saga, Error>
where
    Publisher: ActionPublisher<A, Error>,
    Saga: ActionComputation<AR, A>,
{
    action_publisher: Publisher,
    saga: Saga,
    _marker: PhantomData<(A, AR, Error)>,
}

impl<A, AR, Publisher, Saga, Error> ActionComputation<AR, A>
    for SagaManager<A, AR, Publisher, Saga, Error>
where
    Publisher: ActionPublisher<A, Error>,
    Saga: ActionComputation<AR, A>,
{
    /// Computes new actions based on the action result.
    fn compute_new_actions(&self, action_result: &AR) -> Vec<A> {
        self.saga.compute_new_actions(action_result)
    }
}

#[async_trait]
impl<A, AR, Publisher, Saga, Error> ActionPublisher<A, Error>
    for SagaManager<A, AR, Publisher, Saga, Error>
where
    Publisher: ActionPublisher<A, Error> + Sync,
    Saga: ActionComputation<AR, A> + Sync,
    A: Sync,
    AR: Sync,
    Error: Sync,
{
    /// Publishes the action/command to some external system, returning either the actions that are successfully published or error.
    async fn publish(&self, action: &[A]) -> Result<Vec<A>, Error> {
        self.action_publisher.publish(action).await
    }
}

impl<A, AR, Publisher, Saga, Error> SagaManager<A, AR, Publisher, Saga, Error>
where
    Publisher: ActionPublisher<A, Error> + Sync,
    Saga: ActionComputation<AR, A> + Sync,
    A: Sync,
    AR: Sync,
    Error: Sync,
{
    /// Creates a new instance of [SagaManager].
    pub fn new(action_publisher: Publisher, saga: Saga) -> Self {
        SagaManager {
            action_publisher,
            saga,
            _marker: PhantomData,
        }
    }
    /// Handles the `action result` by computing new `actions` based on `action result`, and publishing new `actions` to the external system.
    /// In most cases:
    ///  - the `action result` is an `event` that you react,
    ///  - the `actions` are `commands` that you publish downstream.
    pub async fn handle(&self, action_result: &AR) -> Result<Vec<A>, Error> {
        let new_actions = self.compute_new_actions(action_result);
        let published_actions = self.publish(&new_actions).await?;
        Ok(published_actions)
    }
}
