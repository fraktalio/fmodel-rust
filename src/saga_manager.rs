use std::marker::PhantomData;

use async_trait::async_trait;

use crate::saga::Saga;

/// Publishes the action/command to some external system.
///
/// Generic parameter:
///
/// - `A`. - action
/// - `Error` - error
#[async_trait]
pub trait ActionPublisher<A, Error> {
    async fn publish(&self, action: &[A]) -> Result<Vec<A>, Error>;
}

/// Saga Manager.
///
/// It is using a [Saga] to react to the action result and to publish the new actions.
/// It is using an [ActionPublisher] to publish the new actions.
///
/// Generic parameters:
/// - `A` - Action / Command
/// - `AR` - Action Result / Event
/// - `Publisher` - Action Publisher
/// - `Error` - Error
pub struct SagaManager<'a, A, AR, Publisher, Error>
    where
        Publisher: ActionPublisher<A, Error>,
{
    action_publisher: Publisher,
    saga: Saga<'a, AR, A>,
    _marker: PhantomData<(A, AR, Error)>,
}

impl<'a, A, AR, Publisher, Error> SagaManager<'a, A, AR, Publisher, Error>
    where
        Publisher: ActionPublisher<A, Error>,
{
    pub fn new(action_publisher: Publisher, saga: Saga<'a, AR, A>) -> Self {
        SagaManager {
            action_publisher,
            saga,
            _marker: PhantomData,
        }
    }
    /// Handles the action result by publishing it to the external system.
    pub async fn handle(&self, action_result: &AR) -> Result<Vec<A>, Error> {
        let new_actions = (self.saga.react)(action_result);
        let published_actions = self.action_publisher.publish(&new_actions).await?;
        Ok(published_actions)
    }
}