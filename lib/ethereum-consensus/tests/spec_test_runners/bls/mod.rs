use crate::test_utils::{load_yaml, TestCase};
use ethereum_consensus::crypto::{
    aggregate, aggregate_verify, eth_aggregate_public_keys, eth_fast_aggregate_verify,
    fast_aggregate_verify, verify_signature, PublicKey, SecretKey, Signature,
};
use ethereum_consensus::primitives::Bytes32;
use ethereum_consensus::serde as eth_serde;
use serde::Deserialize;
use serde_with::{serde_as, DefaultOnError};

#[derive(Debug, Deserialize)]
pub struct AggregateTestCase {
    input: Vec<Signature>,
    output: Option<Signature>,
}

impl AggregateTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/data.yaml";
        load_yaml(&path)
    }
}

impl TestCase for AggregateTestCase {
    fn should_succeed(&self) -> bool {
        self.output.is_some()
    }

    fn verify_success(&self) -> bool {
        let aggregation = aggregate(&self.input).unwrap();
        self.output.as_ref().unwrap() == &aggregation
    }

    fn verify_failure(&self) -> bool {
        let aggregation = aggregate(&self.input);
        aggregation.is_err()
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct AggregateVerifyInput {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pubkeys: Vec<Option<PublicKey>>,
    messages: Vec<Bytes32>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    signature: Option<Signature>,
}

#[derive(Debug, Deserialize)]
pub struct AggregateVerifyTestCase {
    input: AggregateVerifyInput,
    output: bool,
}

impl AggregateVerifyTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/data.yaml";
        load_yaml(&path)
    }

    pub fn run(&self) -> bool {
        let public_keys = self
            .input
            .pubkeys
            .iter()
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        let messages = self
            .input
            .messages
            .iter()
            .map(|m| m.as_ref())
            .collect::<Vec<_>>();
        let signature = self.input.signature.as_ref().unwrap();
        aggregate_verify(&public_keys, &messages, signature).is_ok()
    }
}

impl TestCase for AggregateVerifyTestCase {
    fn should_succeed(&self) -> bool {
        self.output
    }

    fn verify_success(&self) -> bool {
        self.run()
    }

    fn verify_failure(&self) -> bool {
        if self.input.signature.is_none() {
            return true;
        }
        !self.run()
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct FastAggregateVerifyInput {
    pubkeys: Vec<String>,
    message: Bytes32,
    #[serde_as(deserialize_as = "DefaultOnError")]
    signature: Option<Signature>,
}

#[derive(Debug, Deserialize)]
pub struct FastAggregateVerifyTestCase {
    input: FastAggregateVerifyInput,
    output: bool,
}

impl FastAggregateVerifyTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/data.yaml";
        load_yaml(&path)
    }

    pub fn run(&self) -> bool {
        let public_keys = &self
            .input
            .pubkeys
            .iter()
            .map(|repr| serde_yaml::from_str(repr).unwrap())
            .collect::<Vec<PublicKey>>();
        let public_keys = public_keys.iter().collect::<Vec<_>>();
        fast_aggregate_verify(
            &public_keys,
            self.input.message.as_ref(),
            self.input.signature.as_ref().unwrap(),
        )
        .is_ok()
    }
}

impl TestCase for FastAggregateVerifyTestCase {
    fn should_succeed(&self) -> bool {
        self.output
    }

    fn verify_success(&self) -> bool {
        self.run()
    }

    fn verify_failure(&self) -> bool {
        if self.input.signature.is_none() {
            return true;
        }
        if self.input.pubkeys.iter().any(|key| {
            let input: Result<PublicKey, _> = serde_yaml::from_str(key);
            input.is_err()
        }) {
            return true;
        }

        !self.run()
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct SignInput {
    #[serde_as(deserialize_as = "DefaultOnError")]
    privkey: Option<SecretKey>,
    #[serde(deserialize_with = "eth_serde::as_hex::deserialize")]
    message: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct SignTestCase {
    input: SignInput,
    output: Option<Signature>,
}

impl SignTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/data.yaml";
        load_yaml(&path)
    }
}

impl TestCase for SignTestCase {
    fn should_succeed(&self) -> bool {
        self.output.is_some()
    }

    fn verify_success(&self) -> bool {
        let signature = self
            .input
            .privkey
            .as_ref()
            .unwrap()
            .sign(self.input.message.as_ref());
        &signature == self.output.as_ref().unwrap()
    }

    fn verify_failure(&self) -> bool {
        self.input.privkey.is_none()
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct VerifyInput {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pubkey: Option<PublicKey>,
    message: Bytes32,
    #[serde_as(deserialize_as = "DefaultOnError")]
    signature: Option<Signature>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyTestCase {
    input: VerifyInput,
    output: bool,
}

impl VerifyTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/data.yaml";
        load_yaml(&path)
    }

    fn run(&self) -> bool {
        verify_signature(
            self.input.pubkey.as_ref().unwrap(),
            self.input.message.as_ref(),
            self.input.signature.as_ref().unwrap(),
        )
        .is_ok()
    }
}

impl TestCase for VerifyTestCase {
    fn should_succeed(&self) -> bool {
        self.output
    }

    fn verify_success(&self) -> bool {
        self.run()
    }

    fn verify_failure(&self) -> bool {
        if self.input.signature.is_none() {
            return true;
        }
        if self.input.pubkey.is_none() {
            return true;
        }
        !self.run()
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct EthAggregatePubkeysTestCase {
    #[serde_as(deserialize_as = "DefaultOnError")]
    input: Vec<PublicKey>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    output: Option<PublicKey>,
}

impl EthAggregatePubkeysTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/data.yaml";
        load_yaml(&path)
    }

    pub fn run(&self) -> bool {
        let aggregate_public_key = eth_aggregate_public_keys(&self.input).unwrap();
        &aggregate_public_key == self.output.as_ref().unwrap()
    }
}

impl TestCase for EthAggregatePubkeysTestCase {
    fn should_succeed(&self) -> bool {
        self.output.is_some()
    }

    fn verify_success(&self) -> bool {
        self.run()
    }

    fn verify_failure(&self) -> bool {
        self.output.is_none()
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct EthFastAggregateVerifyInput {
    #[serde(rename = "pubkeys")]
    public_keys: Vec<String>,
    message: Bytes32,
    #[serde_as(deserialize_as = "DefaultOnError")]
    signature: Option<Signature>,
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct EthFastAggregateVerifyTestCase {
    input: EthFastAggregateVerifyInput,
    output: bool,
}

impl EthFastAggregateVerifyTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let path = test_case_path.to_string() + "/data.yaml";
        load_yaml(&path)
    }

    pub fn run(&self) -> bool {
        let public_keys = &self
            .input
            .public_keys
            .iter()
            .map(|repr| serde_yaml::from_str(repr).unwrap())
            .collect::<Vec<PublicKey>>();
        let public_keys = public_keys.iter().collect::<Vec<_>>();
        let message = self.input.message.as_ref();
        let signature = self.input.signature.as_ref().unwrap();
        eth_fast_aggregate_verify(&public_keys, message, signature).is_ok()
    }
}

impl TestCase for EthFastAggregateVerifyTestCase {
    fn should_succeed(&self) -> bool {
        self.output
    }

    fn verify_success(&self) -> bool {
        self.run()
    }

    fn verify_failure(&self) -> bool {
        if self.input.signature.is_none() {
            return true;
        }

        if self.input.public_keys.iter().any(|key| {
            let input: Result<PublicKey, _> = serde_yaml::from_str(key);
            input.is_err()
        }) {
            return true;
        }

        !self.run()
    }
}
