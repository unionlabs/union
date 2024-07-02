use std::{collections::HashMap, fmt::Display, hash::Hash, path::PathBuf, sync::Arc};

use crossbeam_queue::ArrayQueue;
use futures::Future;
use serde::{Deserialize, Serialize};
use tracing::{info_span, warn, Instrument};

pub trait ChainKeyring {
    type Address: Hash + Eq + Clone + Display;
    type Signer;

    fn keyring(&self) -> &ConcurrentKeyring<Self::Address, Self::Signer>;

    fn balances(&self) -> impl Future<Output = Vec<SignerBalance<Self::Address>>> + Send;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerBalance<A> {
    pub key_name: String,
    pub address: A,
    #[serde(with = "::serde_utils::string")]
    pub balance: u128,
    pub denom: String,
}

#[derive(Debug, Clone)]
pub struct ConcurrentKeyring<A: Hash + Eq, S> {
    pub name: Arc<String>,

    /// Bidirectional mapping of address <-> key name, allowing for accessing keys by either name or address.
    address_to_key: Arc<HashMap<A, String>>,
    key_to_address: Arc<HashMap<String, A>>,

    /// Ring buffer containing the addresses, used to index into `keys`. Items are popped out of this and then pushed to the back once they're finished being used.
    addresses_buffer: Arc<ArrayQueue<A>>,

    signers: Arc<HashMap<A, S>>,
}

pub struct KeyringEntry<A, S> {
    pub name: String,
    pub address: A,
    pub signer: S,
}

impl<A: Hash + Eq + Clone + Display, S: 'static> ConcurrentKeyring<A, S> {
    // TODO: Maybe add a from_config constructor that takes KeyringConfig and a fn from KeyringConfigEntry -> KeyringEntry?
    pub fn new(
        name: impl Into<String>,
        entries: impl ExactSizeIterator<Item = KeyringEntry<A, S>>,
    ) -> Self {
        let mut address_to_key = HashMap::new();
        let mut key_to_address = HashMap::new();
        let mut signers = HashMap::new();
        let addresses_buffer = ArrayQueue::new(entries.len());

        for key in entries {
            key_to_address.insert(key.name.clone(), key.address.clone());
            address_to_key.insert(key.address.clone(), key.name);
            signers.insert(key.address.clone(), key.signer);
            addresses_buffer
                .push(key.address)
                .ok()
                .expect("buffer is created with the expected length; qed;");
        }

        Self {
            name: Arc::new(name.into()),
            address_to_key: Arc::new(address_to_key),
            key_to_address: Arc::new(key_to_address),
            addresses_buffer: Arc::new(addresses_buffer),
            signers: Arc::new(signers),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = (&str, &A)> {
        self.key_to_address.iter().map(|(a, b)| (a.as_str(), b))
    }

    pub async fn with<'a, F: FnOnce(&'a S) -> Fut + 'a, Fut: Future<Output: 'a> + 'a>(
        &'a self,
        f: F,
    ) -> Option<Fut::Output> {
        let Some(address) = self.addresses_buffer.pop() else {
            warn!(keyring = %self.name, "high traffic in keyring");
            return None;
        };

        let key_name = self
            .address_to_key
            .get(&address)
            .expect("key is present; qed;");
        let secret = self.signers.get(&address).expect("key is present; qed;");

        let r = f(secret)
            .instrument(info_span!(
                "using signer",
                keyring = %self.name,
                %key_name,
                %address
            ))
            .await;

        self.addresses_buffer
            .push(address)
            .ok()
            .expect("no additional items are added; qed;");

        Some(r)
    }
}

#[derive(Default)] // NOTE: Default impl is temporary until the EthereumSignersConfig stuff gets removed/ refactored
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct KeyringConfig {
    pub name: String,
    pub keys: Vec<KeyringConfigEntry>,
}

impl KeyringConfigEntry {
    pub fn value(&self) -> Vec<u8> {
        match &self {
            KeyringConfigEntry::File { path: _ } => {
                panic!("file keyring is currently unimplemented")
            }
            KeyringConfigEntry::Raw { name: _, key } => key.clone(),
        }
    }

    pub fn name(&self) -> String {
        match &self {
            KeyringConfigEntry::File { path: _ } => {
                panic!("file keyring is currently unimplemented")
            }
            KeyringConfigEntry::Raw { name, key: _ } => name.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum KeyringConfigEntry {
    File {
        path: PathBuf,
    },
    Raw {
        name: String,
        #[serde(with = "::serde_utils::hex_string")]
        key: Vec<u8>,
    },
}
