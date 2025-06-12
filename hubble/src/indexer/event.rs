use serde::Serializer;
use serde_json::Value;
use sqlx::Postgres;
use tracing::debug;

use crate::indexer::{
    api::{BlockReference, FetcherClient, IndexerError},
    nats::subject_for_block,
    postgres::schedule,
    Indexer,
};

#[derive(serde::Serialize)]
pub struct HubbleEvent {
    pub version: u8,
    pub universal_chain_id: String,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk: Option<Chunk>,
    #[serde(skip_serializing_if = "Value::is_null")]
    pub events: Value,
}

fn serialize_as_str<S, T>(x: &T, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToString,
{
    s.serialize_str(&x.to_string())
}

#[derive(serde::Serialize)]
pub struct Range {
    #[serde(serialize_with = "serialize_as_str")]
    pub start: u64,
    #[serde(serialize_with = "serialize_as_str")]
    pub end: u64,
}

#[derive(serde::Serialize)]
pub struct Chunk {
    pub index: u8,
    pub total: u8,
}

impl<T: FetcherClient> Indexer<T> {
    pub async fn schedule_event(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        reference: &BlockReference,
        events: Value,
    ) -> Result<(), IndexerError> {
        if self.nats.is_some() {
            debug!("scheduling: {reference}");

            let subject = subject_for_block(&self.indexer_id);

            let data = serde_json::to_vec(&HubbleEvent {
                version: 1,
                universal_chain_id: self.indexer_id.clone(),
                range: Range {
                    start: reference.height,
                    end: reference.height,
                },
                chunk: None,
                events,
            })
            .map_err(|e| IndexerError::InternalError(e.into()))?;

            let id = schedule(tx, &subject, data.into()).await?;

            debug!("scheduled: {reference} - {id}");
        }

        Ok(())
    }
}
