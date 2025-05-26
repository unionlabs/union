#![allow(clippy::type_complexity)]

use std::fmt::Debug;

use futures::Future;
use pg_queue::{PgQueue, PgQueueConfig};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{error, trace};
use voyager_message::VoyagerMessage;
use voyager_vm::{
    filter::InterestFilter, in_memory::InMemoryQueue, pass::Pass, Captures, EnqueueResult, ItemId,
    Op, Queue, QueueError,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum QueueConfig {
    InMemory,
    PgQueue(PgQueueConfig),
}

#[derive(Debug, Clone)]
pub enum QueueImpl {
    InMemory(InMemoryQueue<VoyagerMessage>),
    PgQueue(PgQueue<VoyagerMessage>),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum AnyQueueError {
    InMemory(std::convert::Infallible),
    PgQueue(sqlx::Error),
}

impl Queue<VoyagerMessage> for QueueImpl {
    type Error = AnyQueueError;
    type Config = QueueConfig;

    fn new(cfg: Self::Config) -> impl Future<Output = Result<Self, Self::Error>> {
        async move {
            Ok(match cfg {
                QueueConfig::InMemory => Self::InMemory(
                    InMemoryQueue::new(())
                        .await
                        .map_err(AnyQueueError::InMemory)?,
                ),
                QueueConfig::PgQueue(cfg) => {
                    Self::PgQueue(PgQueue::new(cfg).await.map_err(AnyQueueError::PgQueue)?)
                }
            })
        }
    }

    fn enqueue<'a, Filter: InterestFilter<VoyagerMessage>>(
        &'a self,
        item: Op<VoyagerMessage>,
        filter: &'a Filter,
    ) -> impl Future<Output = Result<EnqueueResult, Self::Error>> + Send + 'a {
        async move {
            let res = match self {
                QueueImpl::InMemory(queue) => queue
                    .enqueue(item, filter)
                    .await
                    .map_err(AnyQueueError::InMemory)?,
                QueueImpl::PgQueue(queue) => queue
                    .enqueue(item, filter)
                    .await
                    .map_err(AnyQueueError::PgQueue)?,
            };

            trace!("queued");

            Ok(res)
        }
    }

    fn process<'a, F, Fut, R, Filter: InterestFilter<VoyagerMessage>>(
        &'a self,
        filter: &'a Filter,
        f: F,
    ) -> impl Future<Output = Result<Option<R>, Self::Error>> + Send + Captures<'a>
    where
        F: (FnOnce(Op<VoyagerMessage>, ItemId) -> Fut) + Send + Captures<'a>,
        Fut:
            Future<Output = (R, Result<Vec<Op<VoyagerMessage>>, QueueError>)> + Send + Captures<'a>,
        R: Send + Sync + 'static,
    {
        async move {
            let res = match self {
                QueueImpl::InMemory(queue) => queue
                    .process(filter, f)
                    .await
                    .map_err(AnyQueueError::InMemory),
                QueueImpl::PgQueue(queue) => queue
                    .process(filter, f)
                    .await
                    .map_err(AnyQueueError::PgQueue),
            };

            trace!("processed");

            res
        }
    }

    async fn optimize<'a, O: Pass<VoyagerMessage>, Filter: InterestFilter<VoyagerMessage>>(
        &'a self,
        tag: &'a str,
        filter: &'a Filter,
        optimizer: &'a O,
    ) -> Result<(), sqlx::Either<Self::Error, O::Error>> {
        match self {
            QueueImpl::InMemory(queue) => queue
                .optimize(tag, filter, optimizer)
                .await
                .map_err(|e| e.map_left(AnyQueueError::InMemory)),
            QueueImpl::PgQueue(queue) => queue
                .optimize(tag, filter, optimizer)
                .await
                .map_err(|e| e.map_left(AnyQueueError::PgQueue)),
        }
    }
}
