use std::{
    future::Future,
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use serde::{de::DeserializeOwned, Serialize};
use sqlx::{migrate::Migrator, query, query_as, types::Json, Acquire, Postgres};

pub static MIGRATOR: Migrator = sqlx::migrate!(); // defaults to "./migrations"

/// A fifo queue backed by a postgres table. Not suitable for high-throughput, but enough for ~1k items/sec.
///
/// The queue assumes the following database schema:
///
///     id SERIAL AUTO INCREMENT
///     status 0..2
///     item JSONB
///     error TEXT
#[derive(Debug, Clone)]
pub struct Queue<T> {
    lock: Arc<AtomicBool>,
    __marker: PhantomData<fn() -> T>,
}

impl<T> Queue<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            lock: Arc::new(AtomicBool::new(false)),
            __marker: PhantomData,
        }
    }
}

impl<T: DeserializeOwned + Serialize + Unpin + Send + Sync> Queue<T> {
    /// Enqueues a new item for processing. The item's processing status is set to 0, indicating that it is ready
    /// for processing.
    pub async fn enqueue<'a, A>(&self, conn: A, item: T) -> Result<(), sqlx::Error>
    where
        A: Acquire<'a, Database = Postgres>,
    {
        let mut tx = conn.begin().await?;

        let row = query!(
            "INSERT INTO queue (item) VALUES ($1) RETURNING id",
            Json(item) as _
        )
        .fetch_one(tx.as_mut())
        .await?;

        tx.commit().await?;

        tracing::info!(id = row.id, "enqueued item");

        self.lock.store(false, Ordering::SeqCst);

        Ok(())
    }

    /// Processes the next value from the queue, calling `f` on the value. Dequeueing has the following properties:
    /// - if `f` returns an error, the item is requeued.
    /// - if `f` returns Ok(ProcessFlow::Fail), the item is permanently marked as failed.
    /// - if `f` returns Ok(ProcessFlow::Continue), the item is requeued, but process returns with Ok(()).
    /// - if `f` returns Ok(ProcessFlow::Success), the item is marked as processed.
    ///
    /// Database atomicity is used to ensure that the queue is always in a consistent state, meaning that an item
    /// process will always be retried until it reaches ProcessFlow::Fail or ProcessFlow::Success. `f` is responsible for
    /// storing metadata in the job to determine if retrying should fail permanently.
    pub async fn process<'a, F, Fut, A>(&self, conn: A, f: F) -> Result<(), sqlx::Error>
    where
        F: (FnOnce(T) -> Fut) + 'a,
        Fut: Future<Output = ProcessFlow<T>> + 'a,
        A: Acquire<'a, Database = Postgres>,
    {
        if self.lock.swap(false, Ordering::SeqCst) {
            tracing::debug!("queue is locked");
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }

        let mut tx = conn.begin().await?;

        #[derive(Debug)]
        struct Record<T> {
            id: i64,
            item: Json<T>,
        }

        let row = query_as!(
            Record::<T>,
            "
            UPDATE queue
            SET status = 'done'::status
            WHERE id = (
              SELECT id
              FROM queue
              WHERE status = 'ready'::status
              ORDER BY id ASC
              FOR UPDATE SKIP LOCKED
              LIMIT 1
            )
            RETURNING id, item as \"item: Json<T>\"",
        )
        .fetch_optional(tx.as_mut())
        .await?;

        match row {
            Some(row) => {
                tracing::info!(id = row.id, "processing item");

                match f(row.item.0).await {
                    ProcessFlow::Fail(error) => {
                        // Insert error message in the queue
                        query!(
                            "UPDATE queue
                            SET status = 'failed'::status, message = $1
                            WHERE id = $2",
                            error,
                            row.id,
                        )
                        .execute(tx.as_mut())
                        .await?;
                        tx.commit().await?;
                    }
                    ProcessFlow::Success(new_msgs) => {
                        if !new_msgs.is_empty() {
                            let new_ids = query!(
                                "INSERT INTO queue (item)
                                SELECT * FROM UNNEST($1::JSONB[])
                                RETURNING id",
                                &*new_msgs
                                    .into_iter()
                                    .map(|t| serde_json::to_value(t).expect(
                                        "queue message should have infallible serialization"
                                    ))
                                    .collect::<Vec<_>>(),
                            )
                            .fetch_all(tx.as_mut())
                            .await?;

                            for row in new_ids {
                                tracing::debug!(id = row.id, "inserted new messages");
                            }
                        }

                        tx.commit().await?;
                    }
                    ProcessFlow::Requeue => {
                        tx.rollback().await?;
                    }
                }
            }
            None => {
                tracing::debug!("queue is empty");
                self.lock.store(true, Ordering::SeqCst);
                tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
            }
        }
        Ok(())
    }
}

pub enum ProcessFlow<T> {
    Success(Vec<T>),
    Requeue,
    Fail(String),
}
