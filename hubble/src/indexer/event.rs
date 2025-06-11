use serde::Serializer;
use serde_json::Value;

#[derive(serde::Serialize)]
pub struct HubbleEvent {
    pub version: u8,
    pub universal_chain_id: String,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk: Option<Chunk>,
    #[serde(skip_serializing_if = "Value::is_null")]
    pub details: Value,
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
