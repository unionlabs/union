use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use alloy::{
    dyn_abi::{DynSolValue, EventExt},
    json_abi::JsonAbi,
    primitives::Address,
};
use alloy_primitives::B256;
use itertools::Itertools;
use serde_json::Value;

use crate::{
    github_client::GitCommitHash,
    indexer::{
        api::{AbiParsingError, IndexerError},
        ethereum::log_parser::Parser,
        record::InternalChainId,
    },
};

pub struct Abi {
    pub internal_chain_id: InternalChainId,
    pub address: Address,
    pub description: String,
    pub definition: String,
    pub commit: GitCommitHash,
}

impl Abi {
    pub fn decode_to_json(&self, log: &alloy::rpc::types::Log) -> Result<Value, IndexerError> {
        let abi: JsonAbi =
            serde_json::from_str(&self.definition).expect("deserializing json abi failed");
        let parser = Parser::new(&abi);
        let result = parser
            .parse(log)
            .expect("could not parse log into keyed data");
        let json = serde_json::to_value(result).expect("could not convert keyed events to json");

        Ok(json)
    }

    pub fn parse(&self, log: &alloy::rpc::types::Log) -> Result<SolEvent, IndexerError> {
        let abi: JsonAbi = serde_json::from_str(&self.definition).map_err(|err| {
            IndexerError::AbiCannotParse(
                Box::new(AbiParsingError::DeserializingJsonAbiFailed(err)),
                self.internal_chain_id,
                self.address,
                self.description.clone(),
                self.commit.clone(),
            )
        })?;

        let selector = log.topics().first().unwrap();
        let definition = abi
            .events()
            .find(|e| e.selector().0 == selector.0)
            .ok_or(AbiParsingError::UnknownEvent {
                selector: *selector,
            })
            .map_err(|err| {
                IndexerError::AbiCannotParse(
                    Box::new(err),
                    self.internal_chain_id,
                    self.address,
                    self.description.clone(),
                    self.commit.clone(),
                )
            })?;

        let topics = log.topics().iter().map(|t| B256::from_slice(&t.0));
        let decoded = definition
            .decode_log_parts(topics, &log.data().data)
            .map_err(|err| {
                IndexerError::AbiCannotParse(
                    Box::new(AbiParsingError::DecodingError(err)),
                    self.internal_chain_id,
                    self.address,
                    self.description.clone(),
                    self.commit.clone(),
                )
            })?;
        let indexed = definition.inputs.iter().filter(|e| e.indexed);
        let body = definition.inputs.iter().filter(|e| !e.indexed);

        let indexed = indexed.zip(decoded.indexed);
        let body = body.zip(decoded.body);

        let values = indexed
            .chain(body)
            .map(|(k, v)| (k.name.clone(), v))
            .collect();

        Ok(SolEvent {
            name: definition.name.clone(),
            attributes: values,
        })
    }
}

pub struct SolEvent {
    pub name: String,
    pub attributes: HashMap<String, DynSolValue>,
}

impl Display for SolEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.keys_as_string())
    }
}

pub struct AbiRegistration {
    pub administration: HashMap<Address, Abi>,
}

impl AbiRegistration {
    pub fn addresses(&self) -> Vec<Address> {
        self.administration.keys().cloned().collect_vec()
    }

    pub fn get_abi_for_address<'a>(&'a self, address: &Address) -> Option<&'a Abi> {
        self.administration.get(address)
    }
}

impl Display for AbiRegistration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let addresses: Vec<String> = self
            .administration
            .keys()
            .map(|addr| addr.to_string())
            .collect();
        write!(f, "{}", addresses.join(", "))
    }
}

pub struct GeneratedAbi {
    pub abi: String,
    pub command: String,
}
