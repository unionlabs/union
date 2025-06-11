use async_nats::header::HeaderMap;
use bytes::Bytes;

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
