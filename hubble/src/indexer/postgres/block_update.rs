use sqlx::{Postgres, Transaction};
use tracing::debug;

use crate::indexer::{
    api::IndexerError,
    consumer::BlockUpdate,
    event::types::{BlockHeight, Range, UniversalChainId},
    record::PgValue,
};

pub async fn get_block_updates(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    universal_chain_id: &UniversalChainId,
    range: &Range,
) -> Result<Vec<BlockUpdate>, IndexerError> {
    sqlx::query!(
        "
        SELECT b.universal_chain_id,
               b.height,
               b.message_sequence,
               b.delete,
               b.message_hash,
               b.nats_stream_sequence,
               b.nats_consumer_sequence
        FROM   hubble.block_update b
        WHERE  b.universal_chain_id = $1
        AND    b.height BETWEEN $2 AND $3
        FOR UPDATE
        ",
        universal_chain_id.clone().pg_value()?,
        i64::try_from(range.start_inclusive).expect("start fits"),
        i64::try_from(range.end_exclusive - 1).expect("end fits"), // BETWEEN is inclusive
    )
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|r| {
        Ok(BlockUpdate {
            universal_chain_id: r.universal_chain_id.into(),
            height: r.height.try_into()?,
            message_sequence: r.message_sequence.try_into()?,
            delete: r.delete,
            message_hash: r.message_hash.into(),
            nats_stream_sequence: r.nats_stream_sequence.try_into()?,
            nats_consumer_sequence: r.nats_consumer_sequence.try_into()?,
        })
    })
    .collect::<Result<Vec<_>, IndexerError>>()
}

pub async fn insert_block_update(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    block_update: &BlockUpdate,
) -> Result<(), IndexerError> {
    sqlx::query!(
        "
        INSERT INTO hubble.block_update (
            universal_chain_id, 
            height,
            message_sequence,
            delete,
            message_hash, 
            nats_stream_sequence, 
            nats_consumer_sequence
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        ",
        block_update.universal_chain_id.pg_value()?,
        block_update.height.pg_value()?,
        block_update.message_sequence.pg_value()?,
        block_update.delete,
        block_update.message_hash.pg_value()?,
        block_update.nats_stream_sequence.pg_value()?,
        block_update.nats_consumer_sequence.pg_value()?,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

pub async fn update_block_update(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    block_update: BlockUpdate,
) -> Result<(), IndexerError> {
    sqlx::query!(
        "
        UPDATE hubble.block_update
            SET message_sequence = $3
              , delete = $4
              , message_hash = $5
              , nats_stream_sequence = $6
              , nats_consumer_sequence = $7
        WHERE universal_chain_id = $1 
          AND height = $2
        ",
        block_update.universal_chain_id.pg_value()?,
        block_update.height.pg_value()?,
        block_update.message_sequence.pg_value()?,
        block_update.delete,
        block_update.message_hash.pg_value()?,
        block_update.nats_stream_sequence.pg_value()?,
        block_update.nats_consumer_sequence.pg_value()?,
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

pub async fn max_event_height(
    tx: &mut Transaction<'_, Postgres>,
    universal_chain_id: &UniversalChainId,
) -> Result<BlockHeight, IndexerError> {
    debug!("max_event_height: {universal_chain_id}");

    sqlx::query!(
        "
        SELECT GREATEST(
            (SELECT MAX(height) FROM hubble.block_update WHERE universal_chain_id = $1),
            (SELECT MAX(height) FROM v2_cosmos.events WHERE internal_chain_id = (SELECT id FROM config.chains c WHERE c.family || '.' || c.chain_id = $1)),
            (SELECT MAX(height) FROM v2_cosmos.transactions WHERE internal_chain_id = (SELECT id FROM config.chains c WHERE c.family || '.' || c.chain_id = $1)),
            (SELECT MAX(height) FROM v2_cosmos.blocks WHERE internal_chain_id = (SELECT id FROM config.chains c WHERE c.family || '.' || c.chain_id = $1)),
            (SELECT MAX(height) FROM v2_evm.logs WHERE internal_chain_id = (SELECT id FROM config.chains c WHERE c.family || '.' || c.chain_id = $1))
        ) AS max_height
         ",
        universal_chain_id.pg_value()?,
    )
    .fetch_optional(tx.as_mut())
    .await?
    .map(|record| record.max_height.unwrap_or_default().try_into())
    .unwrap_or(Ok(0.into()))
}
