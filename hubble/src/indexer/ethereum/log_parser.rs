use alloy::{
    dyn_abi::{DynSolValue, EventExt},
    json_abi::JsonAbi,
    rpc::types::Log,
};
use alloy_primitives::{FixedBytes, B256};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

pub struct Parser<'a> {
    abi: &'a JsonAbi,
}

/// A decoded event which is self-describing through String keys.
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyedEvent {
    /// The name of the event.
    name: String,

    /// The data of the emitted event, both indexed and body.
    data: serde_json::Value,
}

#[derive(Error, Debug)]
pub enum ParsingError {
    /// The name of the decoded event is not found in the ABI. This might
    /// indicate an ABI mismatch.
    #[error("event not found for given abi")]
    UnknownEvent { selector: FixedBytes<32> },
    /// The name of the event IS found in the ABI, yet decoding still failed.
    /// This might indicate an out-of-date ABI.
    #[error("could not decode, abi might mismatch data")]
    DecodingError(#[from] alloy::dyn_abi::Error),
}

impl<'a> Parser<'a> {
    pub fn new(abi: &'a JsonAbi) -> Self {
        Self { abi }
    }

    pub fn parse(&self, log: &Log) -> Result<KeyedEvent, ParsingError> {
        let selector = log.topics().first().unwrap();
        let definition = self
            .abi
            .events()
            .find(|e| e.selector().0 == selector.0)
            .ok_or(ParsingError::UnknownEvent {
                selector: *selector,
            })?;

        let topics = log.topics().iter().map(|t| B256::from_slice(&t.0));
        let decoded = definition
            .decode_log_parts(topics, &log.data().data)
            .map_err(ParsingError::DecodingError)?;
        let indexed = definition.inputs.iter().filter(|e| e.indexed);
        let body = definition.inputs.iter().filter(|e| !e.indexed);

        let indexed = indexed.zip(decoded.indexed);
        let body = body.zip(decoded.body);

        let values: Map<String, Value> = indexed
            .chain(body)
            .map(|(k, v)| (k.name.clone(), dyn_sol_to_json(v)))
            .collect();

        Ok(KeyedEvent {
            name: definition.name.clone(),
            data: Value::Object(values),
        })
    }
}

pub fn dyn_sol_to_json(val: DynSolValue) -> Value {
    use base64::prelude::*;

    match val {
        DynSolValue::Bool(b) => Value::Bool(b),
        DynSolValue::Int(i, _) => Value::String(i.to_dec_string()),
        DynSolValue::Uint(i, _) => Value::String(i.to_string()),
        DynSolValue::FixedBytes(v, _) => Value::String(BASE64_STANDARD.encode(v.0)),
        DynSolValue::Address(a) => Value::String(a.to_string()),
        DynSolValue::Function(p) => Value::String(p.to_string()),
        DynSolValue::Bytes(b) => Value::String(BASE64_STANDARD.encode(b)),
        DynSolValue::String(s) => Value::String(s),
        DynSolValue::Array(a) => Value::Array(a.into_iter().map(dyn_sol_to_json).collect()),
        DynSolValue::FixedArray(a) => Value::Array(a.into_iter().map(dyn_sol_to_json).collect()),
        DynSolValue::Tuple(a) => Value::Array(a.into_iter().map(dyn_sol_to_json).collect()),
        DynSolValue::CustomStruct {
            name: _,
            prop_names,
            tuple,
        } => {
            let map = prop_names
                .into_iter()
                .zip(tuple.into_iter().map(dyn_sol_to_json))
                .collect();
            Value::Object(map)
        }
    }
}
