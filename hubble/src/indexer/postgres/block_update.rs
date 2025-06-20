use itertools::Itertools;
use sqlx::Postgres;

use crate::indexer::{
    api::{BlockHeight, UniversalChainId},
    consumer::BlockUpdate,
    event::Range,
};

pub async fn get_block_updates(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    universal_chain_id: &UniversalChainId,
    range: &Range,
) -> sqlx::Result<Vec<BlockUpdate>> {
    Ok(sqlx::query!(
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
        universal_chain_id,
        i64::try_from(range.start_inclusive).expect("start fits"),
        i64::try_from(range.end_exclusive - 1).expect("end fits"), // BETWEEN is inclusive
    )
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|r| BlockUpdate {
        universal_chain_id: r.universal_chain_id,
        height: BlockHeight::try_from(r.height).expect("height fits"),
        message_sequence: u64::try_from(r.message_sequence).expect("message_sequence fits"),
        delete: r.delete,
        message_hash: r.message_hash.into(),
        nats_stream_sequence: u64::try_from(r.nats_stream_sequence)
            .expect("nats_stream_sequence fits"),
        nats_consumer_sequence: u64::try_from(r.nats_consumer_sequence)
            .expect("nats_consumer_sequence fits"),
    })
    .collect_vec())
}

pub async fn insert_block_update(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    block_update: &BlockUpdate,
) -> sqlx::Result<()> {
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
        block_update.universal_chain_id,
        i64::try_from(block_update.height).expect("height fits"),
        i64::try_from(block_update.message_sequence).expect("message_sequence fits"),
        block_update.delete,
        Vec::<u8>::from(block_update.message_hash.clone()),
        i64::try_from(block_update.nats_stream_sequence).expect("nats_stream_sequence fits"),
        i64::try_from(block_update.nats_consumer_sequence).expect("nats_consumer_sequence fits"),
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}

pub async fn update_block_update(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    block_update: BlockUpdate,
) -> sqlx::Result<()> {
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
        block_update.universal_chain_id,
        i64::try_from(block_update.height).expect("height fits"),
        i64::try_from(block_update.message_sequence).expect("message_sequence fits"),
        block_update.delete,
        Vec::<u8>::from(block_update.message_hash),
        i64::try_from(block_update.nats_stream_sequence).expect("nats_stream_sequence fits"),
        i64::try_from(block_update.nats_consumer_sequence).expect("nats_consumer_sequence fits"),
    )
    .execute(tx.as_mut())
    .await?;

    Ok(())
}
