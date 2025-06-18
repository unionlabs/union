use std::{
    fmt::{self, Display},
    time::Duration,
};

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
use lz4_flex::compress_prepend_size;
use tracing::{debug, info};

use crate::indexer::{
    api::{IndexerError, UniversalChainId},
    event::MessageHash,
};

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

        let config = jetstream::stream::Config {
            name: "hubble".to_string(),
            subjects: vec!["hubble.>".to_string()],
            storage: StorageType::File,
            num_replicas: 2,
            max_bytes: 75 * 1024 * 1024 * 1024, // 75GiB
            discard: jetstream::stream::DiscardPolicy::New,
            max_age: Duration::from_secs(7 * 24 * 60 * 60), // 1 week
            ..Default::default()
        };

        info!("ensure 'hubble' stream exists");
        let stream = context.get_or_create_stream(&config).await?;
        info!("found: {stream:?}");

        Ok(Self {
            consumer: consumer.to_string(),
            context,
            stream,
        })
    }

    pub async fn create_consumer(
        &self,
        universal_chain_id: &UniversalChainId,
    ) -> Result<Consumer<Config>, IndexerError> {
        let durable_name = durable_name(&self.consumer, universal_chain_id);
        let consumer_config = jetstream::consumer::pull::Config {
            durable_name: Some(durable_name.clone()),
            description: Some("indexing chain events".to_string()),
            ack_policy: jetstream::consumer::AckPolicy::Explicit,
            filter_subject: subject_for_block(universal_chain_id),
            ..Default::default()
        };

        Ok(self
            .stream
            .get_or_create_consumer(&durable_name, consumer_config)
            .await?)
    }

    pub async fn publish(
        &self,
        universal_chain_id: &UniversalChainId,
        message: &Message,
    ) -> Result<Ack, IndexerError> {
        info!("{}: sending", message.id);

        let mut headers = message.headers.clone();
        headers.append("Content-Encoding", "lz4");
        headers.append("Message-Sequence", message.id.to_string());
        headers.append("Universal-Chain-Id", universal_chain_id.to_string());

        let data = compress_prepend_size(&message.data);

        let ack_future = self
            .context
            .publish_with_headers(message.subject.clone(), headers, data.into())
            .await?;

        debug!("{}: acking", message.id);
        let ack = ack_future.await?;

        debug!("{}: acked (sequence: {})", message.id, ack.sequence);

        Ok(Ack {
            sequence: ack.sequence,
        })
    }
}

pub struct MessageMeta {
    pub subject: String,
    pub universal_chain_id: UniversalChainId,
    pub message_sequence: u64,
    pub message_hash: MessageHash,
    pub nats_stream_sequence: u64,
    pub nats_consumer_sequence: u64,
}

impl fmt::Display for MessageMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [m{}|s{}|c{}] ({})",
            self.subject,
            self.message_sequence,
            self.nats_stream_sequence,
            self.nats_consumer_sequence,
            self.message_hash,
        )
    }
}

pub struct Ack {
    sequence: u64,
}

impl fmt::Display for Ack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ack(sequence: {})", self.sequence)
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

pub fn subject_for_block(universal_chain_id: &UniversalChainId) -> String {
    format!("hubble.block.{}", universal_chain_id)
}

pub fn durable_name(consumer_id: &str, universal_chain_id: &UniversalChainId) -> String {
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
