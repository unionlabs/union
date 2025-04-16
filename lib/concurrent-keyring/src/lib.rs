#![feature(trait_alias)]

pub mod private_key;

use std::{
    collections::HashMap, fmt::Display, hash::Hash, panic::UnwindSafe, path::PathBuf, sync::Arc,
};

use crossbeam_queue::ArrayQueue;
use futures::{Future, FutureExt};
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use tracing::{debug, info_span, warn, Instrument};
use unionlabs::primitives::H256;

pub trait ChainKeyring {
    type Address: Hash + Eq + Clone + Display + Send + Sync;
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

    /// Ring buffer containing the addresses, used to index into `keys`. Items are popped out of this and then pushed to the back once they're finished being used.
    addresses_buffer: Arc<ArrayQueue<A>>,

    signers: Arc<HashMap<A, S>>,
}

pub struct KeyringEntry<A, S> {
    pub address: A,
    pub signer: S,
}

impl<A: Hash + Eq + Clone + Display, S: 'static> ConcurrentKeyring<A, S> {
    // TODO: Maybe add a from_config constructor that takes KeyringConfig and a fn from KeyringConfigEntry -> KeyringEntry?
    pub fn new(
        name: impl Into<String>,
        entries: impl ExactSizeIterator<Item = KeyringEntry<A, S>>,
    ) -> Self {
        // let mut address_to_key = HashMap::new();
        // let mut key_to_address = HashMap::new();
        let mut signers = HashMap::new();
        let addresses_buffer = ArrayQueue::new(entries.len());

        let mut rng = &mut rand::thread_rng();

        let mut entries = entries.collect::<Vec<_>>();
        entries.shuffle(&mut rng);

        for key in entries {
            signers.insert(key.address.clone(), key.signer);
            addresses_buffer
                .push(key.address)
                .ok()
                .expect("buffer is created with the expected length; qed;");
        }

        Self {
            name: Arc::new(name.into()),
            addresses_buffer: Arc::new(addresses_buffer),
            signers: Arc::new(signers),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &A> {
        self.signers.keys()
    }

    pub async fn with<'a, F, Fut>(&'a self, f: F) -> Option<Fut::Output>
    where
        F: FnOnce(&'a S) -> Fut + 'a,
        Fut: Future<Output: 'a> + Sized + UnwindSafe + 'a,
    {
        let Some(address) = self.addresses_buffer.pop() else {
            debug!(keyring = %self.name, "high traffic in keyring");
            return None;
        };

        let secret = self.signers.get(&address).expect("key is present; qed;");

        let r = f(secret)
            .catch_unwind()
            .instrument(info_span!(
                "using signer",
                keyring = %self.name,
                %address
            ))
            .await;

        self.addresses_buffer
            .push(address)
            .ok()
            .expect("no additional items are added; qed;");

        match r {
            Ok(res) => Some(res),
            Err(err) => {
                // lol https://github.com/rust-lang/rust/blob/1.67.1/library/std/src/panicking.rs#L247-L253
                let err = err
                    .downcast::<String>()
                    .map(|s| *s)
                    .or_else(|err| err.downcast::<&str>().map(|s| (*s).to_owned()))
                    .unwrap_or_else(|_| "Box<dyn Any>".to_owned());

                warn!("ConcurrentKeyring::with future panicked: {err}");

                None
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct KeyringConfig {
    pub name: String,
    pub keys: Vec<KeyringConfigEntry>,
}

impl KeyringConfigEntry {
    pub fn value(&self) -> Vec<u8> {
        match &self {
            KeyringConfigEntry::File { path } => std::fs::read_to_string(path)
                .expect("key does not exist")
                .trim()
                .parse::<H256>()
                .expect("key is in an invalid format")
                .into(),
            KeyringConfigEntry::Raw { name: _, key } => key.clone(),
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
