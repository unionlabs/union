use serde::Serialize;
use serde_json::Value;
use sqlx::{error::BoxDynError, migrate::Migrator, query, types::Json, Acquire, Postgres};

pub static MIGRATOR: Migrator = sqlx::migrate!(); // defaults to "./migrations"

/// A fifo queue backed by a postgres table. Not suitable for high-throughput, but enough for ~1k items/sec.
///
/// The queue assumes the following database schema:
///     
///     id SERIAL AUTO INCREMENT
///     status 0..2
///     item JSONB
///     error TEXT
pub struct Queue {}

impl Queue {
    /// Enqueues a new item for processing. The item's processing status is set to 0, indicating that it is ready
    /// for processing.
    pub async fn enqueue<'a, A, T: Serialize + Send + Sync>(
        conn: A,
        item: T,
    ) -> Result<i64, BoxDynError>
    where
        A: Acquire<'a, Database = Postgres>,
    {
        let mut tx = conn.begin().await?;
        let row = query!(
            "INSERT into queue (item) VALUES ($1) RETURNING id",
            Json(item) as _
        )
        .fetch_one(tx.as_mut())
        .await?;
        tx.commit().await?;
        Ok(row.id)
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
    pub async fn process<'a, A>(
        conn: A,
        f: impl FnOnce((i64, Value)) -> Result<ProcessFlow, BoxDynError>,
    ) -> Result<(), BoxDynError>
    where
        A: Acquire<'a, Database = Postgres>,
    {
        let mut tx = conn.begin().await?;

        let row = query!(
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
            RETURNING id, item;",
        )
        .fetch_one(tx.as_mut())
        .await?;

        match f((row.id, row.item))? {
            ProcessFlow::Fail(error) => {
                // Insert error message in the queue
                query!(
                    "
                    UPDATE queue
                    SET status = 'failed'::status, message = $1
                    WHERE id = $2",
                    error,
                    row.id,
                )
                .execute(tx.as_mut())
                .await?;
                tx.commit().await?;
            }
            ProcessFlow::Success => {
                tx.commit().await?;
            }
            ProcessFlow::Requeue => {
                tx.rollback().await?;
            }
        }
        Ok(())
    }
}

pub enum ProcessFlow {
    Success,
    Requeue,
    Fail(String),
}
