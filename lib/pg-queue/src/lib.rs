use core::f64;
use std::{
    borrow::Borrow, cmp::Eq, collections::HashMap, fmt::Write, future::Future, hash::Hash,
    marker::PhantomData, time::Duration,
};

use futures_util::TryStreamExt;
use itertools::Itertools;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::{
    postgres::PgPoolOptions, prelude::FromRow, types::Json, Either, Executor, PgPool, Postgres,
    Transaction,
};
use tracing::{debug, debug_span, error, info, info_span, instrument, trace, warn, Instrument};
use voyager_vm::{
    filter::{FilterResult, InterestFilter},
    pass::{Pass, PassResult},
    BoxDynError, Captures, EnqueueResult, ItemId, Op, QueueError, QueueMessage,
};

use crate::metrics::{ITEM_PROCESSING_DURATION, OPTIMIZE_ITEM_COUNT, OPTIMIZE_PROCESSING_DURATION};

pub mod metrics;

/// A fifo queue backed by a postgres table. Not suitable for high-throughput, but enough for ~1k items/sec.
///
/// The queue assumes the following database schema:
///
/// ```ignore
/// id SERIAL AUTO INCREMENT
/// status 0..2
/// item JSONB
/// error TEXT
/// ```
#[derive(Debug, Clone)]
pub struct PgQueue<T> {
    client: PgPool,
    optimize_batch_limit: Option<i64>,
    retryable_error_expo_backoff_max: f64,
    retryable_error_expo_backoff_multiplier: f64,
    __marker: PhantomData<fn() -> T>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PgQueueConfig {
    pub database_url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    #[serde(default)]
    pub idle_timeout: Option<Duration>,
    #[serde(default)]
    pub max_lifetime: Option<Duration>,
    #[serde(default)]
    pub optimize_batch_limit: Option<i64>,
    #[serde(default = "default_retryable_error_expo_backoff_max")]
    pub retryable_error_expo_backoff_max: f64,
    #[serde(default = "default_retryable_error_expo_backoff_multiplier")]
    pub retryable_error_expo_backoff_multiplier: f64,
}

pub const fn default_max_connections() -> u32 {
    10
}

pub const fn default_min_connections() -> u32 {
    0
}

pub const fn default_retryable_error_expo_backoff_max() -> f64 {
    60.0 * 5.0
}

pub const fn default_retryable_error_expo_backoff_multiplier() -> f64 {
    2.0
}

impl PgQueueConfig {
    pub async fn into_pg_pool(self) -> sqlx::Result<PgPool> {
        PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .idle_timeout(self.idle_timeout)
            .max_lifetime(self.max_lifetime)
            .connect(&self.database_url)
            .await
    }
}

#[derive(FromRow)]
struct Id {
    id: i64,
}

#[derive(Debug, FromRow)]
struct QueueRecord {
    id: i64,
    parents: Vec<i64>,
    item: String,
    created_at: time::OffsetDateTime,
    attempt: i64,
}

#[derive(Debug, FromRow)]
struct OptimizeRecord {
    id: i64,
    #[allow(dead_code)]
    parents: Vec<i64>,
    item: String,
    #[allow(dead_code)]
    created_at: time::OffsetDateTime,
}

#[derive(Debug, FromRow, Serialize)]
#[serde(bound(serialize = ""))]
pub struct FailedRecord<T: QueueMessage> {
    pub id: i64,
    pub parents: Vec<i64>,
    pub item: Json<Op<T>>,
    pub message: String,
    // pub created_at: sqlx::types::time::OffsetDateTime,
}

impl<T: QueueMessage> PgQueue<T> {
    pub async fn query_failed(
        &self,
        page: i64,
        per_page: i64,
        mut item_filters: Vec<String>,
        mut message_filters: Vec<String>,
    ) -> Result<Vec<FailedRecord<T>>, sqlx::Error> {
        // default to all-inclusive filter if none are provided
        if item_filters.is_empty() {
            item_filters.push("%".to_owned())
        }

        if message_filters.is_empty() {
            message_filters.push("%".to_owned())
        }

        sqlx::query(
            r#"
            SELECT
                id,
                parents,
                item,
                message
            FROM
                failed 
            WHERE
                item::TEXT LIKE ANY($1) 
                AND message LIKE ANY($2) 
            ORDER BY
                id DESC
            LIMIT
                $3
            OFFSET
                $4
            "#,
        )
        .bind(item_filters)
        .bind(message_filters)
        .bind(per_page)
        .bind((page - 1) * per_page)
        .map(|row| FailedRecord::<T>::from_row(&row))
        .fetch_all(&self.client)
        .await?
        .into_iter()
        .collect()
    }

    pub async fn query_failed_by_id(
        &self,
        id: i64,
    ) -> Result<Option<FailedRecord<T>>, sqlx::Error> {
        sqlx::query(
            r#"
            SELECT
               id,
               parents,
               item,
               message
            FROM
               failed 
            WHERE
               id = $1
            "#,
        )
        .bind(id)
        .map(|row| FailedRecord::<T>::from_row(&row))
        .fetch_optional(&self.client)
        .await?
        .transpose()
    }
}

impl<T: QueueMessage> voyager_vm::Queue<T> for PgQueue<T> {
    type Config = PgQueueConfig;
    // type Error = tokio_postgres::Error;
    type Error = sqlx::Error;

    async fn new(config: Self::Config) -> Result<Self, Self::Error> {
        // // Connect to the database.
        // let (client, connection) = tokio_postgres::connect(&config.database_url, NoTls).await?;

        // // The connection object performs the actual communication with the database,
        // // so spawn it off to run on its own.
        // tokio::spawn(async move {
        //     if let Err(e) = connection.await {
        //         eprintln!("connection error: {}", e);
        //     }
        // });

        let optimize_batch_limit = config.optimize_batch_limit;
        let retryable_error_expo_backoff_multiplier =
            config.retryable_error_expo_backoff_multiplier;
        let retryable_error_expo_backoff_max = config.retryable_error_expo_backoff_max;

        let pool = config.into_pg_pool().await?;

        pool.execute_many(
            r#"
            CREATE TABLE IF NOT EXISTS
              queue (
                id BIGSERIAL PRIMARY KEY,
                item JSONB NOT NULL,
                parents BIGINT[] DEFAULT '{}',
                created_at timestamptz NOT NULL DEFAULT now(),
                handle_at timestamptz NOT NULL DEFAULT now(),
                attempt INT8 NOT NULL DEFAULT 0
              );

            CREATE TABLE IF NOT EXISTS
              optimize (
                -- TODO: Figure out how to do this properly
                id BIGINT PRIMARY KEY DEFAULT nextval('queue_id_seq'::regclass),
                item JSONB NOT NULL,
                tag text NOT NULL,
                parents BIGINT[] DEFAULT '{}',
                created_at timestamptz NOT NULL DEFAULT now()
              );

            CREATE TABLE IF NOT EXISTS
              done (
                id BIGINT,
                item JSONB NOT NULL,
                parents BIGINT[] DEFAULT '{}',
                created_at timestamptz NOT NULL DEFAULT now(),
                PRIMARY KEY (id, created_at)
              );

            CREATE TABLE IF NOT EXISTS
              failed (
                id BIGINT PRIMARY KEY,
                item JSONB NOT NULL,
                parents BIGINT[] DEFAULT '{}',
                message TEXT,
                created_at timestamptz NOT NULL DEFAULT now()
              );

            CREATE INDEX IF NOT EXISTS index_queue_id ON queue (id);

            CREATE INDEX IF NOT EXISTS index_queue_created_at ON queue (created_at ASC) INCLUDE (id);

            CREATE INDEX IF NOT EXISTS index_queue_handle_at ON queue(handle_at DESC) INCLUDE (id);
            "#,
        )
        .try_for_each(|result| async move {
            trace!("rows affected: {}", result.rows_affected());
            Ok(())
        })
        .instrument(info_span!("init"))
        .await?;

        Ok(Self {
            client: pool,
            optimize_batch_limit,
            retryable_error_expo_backoff_max,
            retryable_error_expo_backoff_multiplier,
            __marker: PhantomData,
        })
    }

    async fn enqueue<'a>(
        &'a self,
        op: Op<T>,
        filter: &'a T::Filter,
    ) -> Result<EnqueueResult, Self::Error> {
        trace!("enqueue");

        let (optimize, ready): (Vec<_>, Vec<_>) =
            op.normalize()
                .into_iter()
                .partition_map(|op| match filter.check_interest(&op) {
                    FilterResult::Interest(interest) => Either::Left((op, interest)),
                    FilterResult::NoInterest => Either::Right(op),
                });

        let mut tx = self.client.begin().await?;

        let ready_ids = sqlx::query(
            "
            INSERT INTO queue (item)
            SELECT * FROM UNNEST($1::JSONB[])
            RETURNING id
            ",
        )
        .bind(ready.into_iter().map(Json).collect::<Vec<_>>())
        .try_map(|x| Id::from_row(&x))
        .fetch_all(tx.as_mut())
        .await?;

        for ready in &ready_ids {
            debug!(id = ready.id, "enqueued ready item");
        }

        let optimize_further_ids = sqlx::query(
            "
            INSERT INTO optimize (item, tag)
            SELECT * FROM UNNEST($1::JSONB[], $2::TEXT[])
            RETURNING id
            ",
        )
        .bind(
            optimize
                .iter()
                .map(|x| Json(x.0.clone()))
                .collect::<Vec<_>>(),
        )
        .bind(
            optimize
                .iter()
                .flat_map(|x| x.1.tags.clone())
                .collect::<Vec<_>>(),
        )
        .try_map(|x| Id::from_row(&x))
        .fetch_all(tx.as_mut())
        .await?;

        for ready in &optimize_further_ids {
            debug!(id = ready.id, "enqueued optimize item");
        }

        tx.commit().await?;

        Ok(EnqueueResult {
            queue: ready_ids
                .into_iter()
                .map(|id| ItemId::new(id.id).expect("invalid id returned from database"))
                .collect(),
            optimize: optimize_further_ids
                .into_iter()
                .map(|id| ItemId::new(id.id).expect("invalid id returned from database"))
                .collect(),
        })
    }

    #[instrument(skip_all)]
    async fn process<'a, F, Fut, R>(
        &'a self,
        filter: &'a T::Filter,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: (FnOnce(Op<T>, ItemId) -> Fut) + Send + Captures<'a>,
        Fut: Future<Output = (R, Result<Vec<Op<T>>, QueueError>)> + Send + Captures<'a>,
        R: Send + Sync + 'static,
    {
        trace!("process");

        let mut tx = self.client.begin().await?;

        let row = sqlx::query(
            r#"
            DELETE FROM
              queue
            WHERE
              id = (
                SELECT
                  id
                FROM
                  queue
                WHERE
                  handle_at < now()
                ORDER BY
                  handle_at ASC
                FOR UPDATE
                  SKIP LOCKED
                LIMIT 1)
            RETURNING
              id,
              parents,
              item::text,
              attempt,
              created_at
            "#,
        )
        .try_map(|x| QueueRecord::from_row(&x))
        .fetch_optional(tx.as_mut())
        .await?;

        let res = match row {
            Some(record) => {
                process_item(
                    &mut tx,
                    record,
                    f,
                    filter,
                    self.retryable_error_expo_backoff_max,
                    self.retryable_error_expo_backoff_multiplier,
                )
                .await?
            }
            None => None,
        };

        tx.commit().await?;

        Ok(res)
    }

    async fn optimize<'a, O: Pass<T>>(
        &'a self,
        tag: &'a str,
        optimizer: &'a O,
    ) -> Result<(), Either<Self::Error, O::Error>> {
        trace!(%tag, "optimize");

        let mut tx = self.client.begin().await.map_err(Either::Left)?;

        let msgs = sqlx::query(
            r#"
            DELETE FROM
              optimize
            WHERE
              id = ANY(
                SELECT
                  id
                FROM
                  optimize
                WHERE
                  tag = $1
                ORDER BY
                  id ASC
                FOR UPDATE
                  SKIP LOCKED
                LIMIT $2)
            RETURNING
              id,
              parents,
              item::text,
              created_at
            "#,
        )
        .bind(tag)
        .bind(self.optimize_batch_limit)
        .try_map(|x| OptimizeRecord::from_row(&x))
        .fetch_all(tx.as_mut())
        .await
        .map_err(Either::Left)?;

        if msgs.is_empty() {
            trace!("optimizer queue is empty");
            tokio::time::sleep(Duration::from_millis(100)).await;
            return Ok(());
        }

        let (ids, msgs) = msgs
            .into_iter()
            .map(|r| {
                Ok((
                    r.id,
                    de(&r.item).map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                ))
            })
            .collect::<Result<(Vec<_>, Vec<_>), sqlx::Error>>()
            .map_err(Either::Left)?;

        OPTIMIZE_ITEM_COUNT.observe(msgs.len() as f64);
        let timer = OPTIMIZE_PROCESSING_DURATION.start_timer();

        let PassResult {
            optimize_further,
            ready,
        } = optimizer
            .run_pass(msgs.clone())
            .instrument(debug_span!(
                "optimizing items",
                ids = ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ))
            .await
            .map_err(Either::Right)?;
        let _ = timer.stop_and_record();

        trace!(
            ready = ready.len(),
            optimize_further = optimize_further.len(),
            "optimized items"
        );

        let get_parent_ids = |parent_idxs: &[usize]| {
            ids.iter()
                .enumerate()
                .filter_map(|(idx, id)| parent_idxs.contains(&idx).then_some(*id))
                .collect::<Vec<_>>()
        };

        for (parent_idxs, new_msg, tag) in optimize_further {
            let parents = get_parent_ids(&parent_idxs);
            trace!(parent_idxs = ?&parent_idxs, parents = ?&parents);

            let new_row = sqlx::query(
                "
                INSERT INTO optimize (item, parents, tag)
                VALUES
                    ($1::JSONB, $2, $3)
                RETURNING id
                ",
            )
            .bind(Json(new_msg))
            .bind(&parents)
            .bind(tag)
            .try_map(|row| Id::from_row(&row))
            .fetch_one(tx.as_mut())
            .await
            .map_err(Either::Left)?;

            debug!(id = new_row.id, "inserted new optimizer message");
        }

        for (parent_idxs, new_msg) in ready {
            let parents = get_parent_ids(&parent_idxs);
            trace!(parent_idxs = ?&parent_idxs, parents = ?&parents);

            let new_row = sqlx::query(
                "
                INSERT INTO queue (item, parents)
                VALUES
                    ($1::JSONB, $2)
                RETURNING id
                ",
            )
            .bind(Json(new_msg))
            .bind(&parents)
            .try_map(|x| Id::from_row(&x))
            .fetch_one(tx.as_mut())
            .await
            .map_err(Either::Left)?;

            debug!(id = new_row.id, "inserted new message");
        }

        tx.commit().await.map_err(Either::Left)?;

        Ok(())
    }
}

#[instrument(
    skip_all,
    fields(
        item_id = record.id,
        attempt = record.attempt
    )
)]
async fn process_item<'a, T: QueueMessage, F, Fut, R>(
    tx: &mut Transaction<'static, Postgres>,
    record: QueueRecord,
    f: F,
    filter: &'a T::Filter,
    retryable_error_expo_backoff_max: f64,
    retryable_error_expo_backoff_multiplier: f64,
) -> Result<Option<R>, sqlx::Error>
where
    F: (FnOnce(Op<T>, ItemId) -> Fut) + Send + Captures<'a>,
    Fut: Future<Output = (R, Result<Vec<Op<T>>, QueueError>)> + Send + Captures<'a>,
    R: Send + Sync + 'static,
{
    trace!(%record.item);

    // really don't feel like defining a new error type right now
    let op = de::<Op<T>>(&record.item).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

    let timer = ITEM_PROCESSING_DURATION.start_timer();
    let (r, res) = f(op.clone(), ItemId::new(record.id).unwrap()).await;
    let _ = timer.stop_and_record();

    match res {
        Err(QueueError::Fatal(error)) => {
            let error = full_error_string(error);
            error!(%error, "fatal error");
            insert_error(record, error, tx).await?;
        }
        Err(QueueError::Unprocessable(error)) => {
            let error = full_error_string(error);
            info!(%error, "unprocessable message");
            insert_error(record, error, tx).await?;
        }
        Err(QueueError::Retry(error)) => {
            warn!(error = %full_error_string(error), "retryable error");
            sqlx::query(
                "
                INSERT INTO
                queue  (id, item,      parents, attempt, handle_at, created_at)
                VALUES ($1, $2::JSONB, $3,      $4,      $5,        $6        )
                ",
            )
            .bind(record.id)
            .bind(record.item)
            .bind(record.parents)
            .bind(record.attempt.saturating_add(1))
            .bind(
                time::OffsetDateTime::now_utc().saturating_add(
                    Duration::try_from_secs_f64(
                        (record.attempt as f64)
                            .powf(retryable_error_expo_backoff_multiplier)
                            .clamp(f64::MIN, retryable_error_expo_backoff_max),
                    )
                    .unwrap_or(Duration::MAX)
                    .try_into()
                    .unwrap_or(time::Duration::MAX),
                ),
            )
            .bind(record.created_at)
            .execute(tx.as_mut())
            .await?;

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Ok(ops) => {
            'block: {
                // insert the op we just processed into done
                sqlx::query(
                    "
                    INSERT INTO
                    done   (id, parents, item,      created_at)
                    VALUES ($1, $2,      $3::JSONB, $4        )
                    ",
                )
                .bind(record.id)
                .bind(record.parents)
                .bind(record.item)
                .bind(record.created_at)
                .execute(tx.as_mut())
                .await?;

                if ops.is_empty() {
                    break 'block;
                }

                let (optimize, ready): (Vec<_>, Vec<_>) = ops
                    .into_iter()
                    .flat_map(Op::normalize)
                    .partition_map(|op| match filter.check_interest(&op) {
                        FilterResult::Interest(tag) => Either::Left((op, tag)),
                        FilterResult::NoInterest => Either::Right(op),
                    });

                sqlx::query(
                    "
                    INSERT INTO queue (item, parents)
                    SELECT *, $1 as parents FROM UNNEST($2::JSONB[])
                    ",
                )
                .bind(vec![record.id])
                .bind(ready.into_iter().map(Json).collect::<Vec<_>>())
                .execute(tx.as_mut())
                .await?;

                sqlx::query(
                    "
                    INSERT INTO optimize (item, tag, parents)
                    SELECT *, $1 as parents FROM UNNEST($2::JSONB[], $3::TEXT[])
                    ",
                )
                .bind(vec![record.id])
                .bind(
                    optimize
                        .iter()
                        .flat_map(|(op, interest)| {
                            interest.tags.iter().map(|_| Json(op.clone())).clone()
                        })
                        .collect::<Vec<_>>(),
                )
                .bind(
                    optimize
                        .iter()
                        .flat_map(|(_, interest)| &interest.tags)
                        .copied()
                        .collect::<Vec<_>>(),
                )
                .execute(tx.as_mut())
                .await?;
            }
        }
    }

    Ok(Some(r))
}

async fn insert_error(
    record: QueueRecord,
    error: String,
    tx: &mut Transaction<'static, Postgres>,
) -> Result<(), sqlx::Error> {
    // insert error message and the op into failed

    sqlx::query(
        r#"
        INSERT INTO
        failed (id, parents, item,      created_at, message)
        VALUES ($1, $2,      $3::JSONB, $4,         $5     )
        "#,
    )
    .bind(record.id)
    .bind(record.parents)
    .bind(record.item)
    .bind(record.created_at)
    .bind(error)
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

// copied from unionlabs::ErrorReporter
fn full_error_string(error: BoxDynError) -> String {
    let mut s = String::new();

    write!(s, "{}", error).unwrap();

    for e in core::iter::successors(error.source(), |e| (*e).source()) {
        write!(s, ": {e}").unwrap();
    }

    s
}

fn de<T: DeserializeOwned>(s: &str) -> Result<T, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_str(s);
    deserializer.disable_recursion_limit();
    // let deserializer = serde_stacker::Deserializer::new(&mut deserializer);
    let json = T::deserialize(&mut deserializer)?;
    Ok(json)
}

pub trait MapExt<K, V> {
    fn get_many<'a, Q>(&'a self, ks: impl IntoIterator<Item = &'a Q>) -> Vec<&'a V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq + 'a;
}

impl<K: Hash + Eq, V> MapExt<K, V> for HashMap<K, V> {
    fn get_many<'a, Q>(&'a self, ks: impl IntoIterator<Item = &'a Q>) -> Vec<&'a V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq + 'a,
    {
        let mut out = vec![];

        for k in ks {
            out.extend(self.get(k));
        }

        out
    }
}
