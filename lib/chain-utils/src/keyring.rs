use std::{collections::HashMap, fmt::Display, hash::Hash, path::PathBuf, sync::Arc};

use bimap::BiMap;
use crossbeam_queue::ArrayQueue;
use futures::Future;
use serde::{Deserialize, Serialize};
use tracing::{info_span, warn, Instrument};

pub trait ChainKeyring {
    type Address: Hash + Eq + Clone + Display;
    type Signer;

    fn keyring(&self) -> &ConcurrentKeyring<Self::Address, Self::Signer>;

    fn balances(&self) -> impl Future<Output = Vec<SignerBalance<Self::Address>>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerBalance<A> {
    pub key_name: String,
    pub address: A,
    pub balance: u128,
    pub denom: String,
}

#[derive(Debug, Clone)]
pub struct ConcurrentKeyring<A: Hash + Eq, S> {
    pub name: Arc<String>,

    /// Bidirectional mapping of address <-> key name, allowing for accessing keys by either name or address.
    keys: Arc<BiMap<A, String>>,
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
        let mut keys = BiMap::new();
        let mut signers = HashMap::new();
        let addresses_buffer = ArrayQueue::new(entries.len());

        for key in entries {
            keys.insert(key.address.clone(), key.name);
            signers.insert(key.address.clone(), key.signer);
            addresses_buffer
                .push(key.address)
                .ok()
                .expect("buffer is created with the expected length; qed;");
        }

        Self {
            name: Arc::new(name.into()),
            keys: Arc::new(keys),
            addresses_buffer: Arc::new(addresses_buffer),
            signers: Arc::new(signers),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = (&str, &A)> {
        self.keys.iter().map(|(a, b)| (b.as_str(), a))
    }

    pub async fn with<'a, F: FnOnce(&'a S) -> Fut + 'a, Fut: Future<Output: 'static> + 'a>(
        &'a self,
        f: F,
    ) -> Fut::Output {
        let address = loop {
            match self.addresses_buffer.pop() {
                Some(t) => break t,
                None => {
                    const RETRY_SECONDS: u64 = 3;

                    warn!(keyring = %self.name, "high traffic in keyring");

                    tokio::time::sleep(std::time::Duration::from_secs(RETRY_SECONDS)).await;

                    continue;
                }
            }
        };

        let key_name = self
            .keys
            .get_by_left(&address)
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

        r
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
