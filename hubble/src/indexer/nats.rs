use std::fmt::Display;

use async_nats::{
    header::HeaderMap,
    jetstream::{
        self,
        consumer::{pull::Config, Consumer},
        stream::StorageType,
    },
    ConnectOptions,
};
use bytes::Bytes;
use tracing::debug;

use crate::indexer::api::IndexerError;

#[derive(Clone)]
pub struct NatsConnection {
    pub consumer: String,
    pub context: async_nats::jetstream::context::Context,
    pub stream: async_nats::jetstream::stream::Stream,
}

impl Display for NatsConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let info = self.stream.cached_info();
        write!(
            f,
            "NatsConnection(consumer: {}, stream: {}, subjects: {:?})",
            self.consumer, info.config.name, info.config.subjects
        )
    }
}

impl NatsConnection {
    pub async fn create(
        url: &str,
        username: &str,
        password: &str,
        consumer: &str,
    ) -> color_eyre::eyre::Result<NatsConnection> {
        debug!("creating client for user {username} to {url}");
        let client = ConnectOptions::new()
            .user_and_password(username.to_string(), password.to_string())
            .connect(url)
            .await?;

        debug!("creating context");
        let context = jetstream::new(client);
        debug!("created context: {context:?}");

        debug!("ensure 'hubble' stream exists");
        let stream = context
            .get_or_create_stream(jetstream::stream::Config {
                name: "hubble".to_string(),
                subjects: vec!["hubble.>".to_string()],
                storage: StorageType::File,
                num_replicas: 2,
                ..Default::default()
            })
            .await?;
        debug!("found stream: {stream:?}");

        Ok(Self {
            consumer: consumer.to_string(),
            context,
            stream,
        })
    }

    pub async fn create_consumer(
        &self,
        indexer_id: &str,
    ) -> Result<Consumer<Config>, IndexerError> {
        let durable_name = durable_name(&self.consumer, indexer_id);
        let consumer_config = jetstream::consumer::pull::Config {
            durable_name: Some(durable_name.clone()),
            description: Some("indexing chain events".to_string()),
            ack_policy: jetstream::consumer::AckPolicy::Explicit,
            filter_subject: subject_for_block(indexer_id),
            ..Default::default()
        };

        Ok(self
            .stream
            .get_or_create_consumer(&durable_name, consumer_config)
            .await?)
    }
}

pub struct Message {
    pub id: i64,
    pub subject: String,
    pub headers: HeaderMap,
    pub data: bytes::Bytes,
}

impl Message {
    pub fn new(id: i64, subject: String, headers: HeaderMap, data: Bytes) -> Self {
        Self {
            id,
            subject,
            headers,
            data,
        }
    }
}

pub fn subject_for_block(universal_chain_id: &str) -> String {
    format!("hubble.block.{}", universal_chain_id)
}

pub fn durable_name(consumer_id: &str, universal_chain_id: &str) -> String {
    sanitize_consumer_name(&format!(
        "{}:{}",
        consumer_id,
        subject_for_block(universal_chain_id)
    ))
}

pub fn sanitize_consumer_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c == '.'
                || c == '*'
                || c == '>'
                || c == '/'
                || c == '\\'
                || c.is_whitespace()
                || c.is_control()
            {
                '-'
            } else {
                c
            }
        })
        .collect()
}
