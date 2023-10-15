#![feature(return_position_impl_trait_in_trait)]
// #![warn(clippy::pedantic)]
#![allow(
     // required due to return_position_impl_trait_in_trait false positives
    clippy::manual_async_fn,
)]

use std::{
    error::Error,
    fmt::{Debug, Display},
    sync::Arc,
};

use crossbeam_queue::ArrayQueue;
use futures::{Future, Stream};
use unionlabs::{
    ethereum::H256,
    events::IbcEvent,
    ibc::core::client::height::Height,
    traits::{Chain, ClientState, Id},
};

pub mod evm;
pub mod union;

pub mod private_key;

pub trait EventSource {
    type Event;
    type Error: Debug;
    /// The initial state of this event source, if any.
    type Seed;

    fn events(self, seed: Self::Seed) -> impl Stream<Item = Result<Self::Event, Self::Error>>;
}

// Serialize, Deserialize
#[derive(Debug, Clone, PartialEq)]
pub struct ChainEvent<C: Chain> {
    /// The chain this event originated from.
    pub chain_id: <C::SelfClientState as ClientState>::ChainId,
    pub block_hash: H256,
    pub height: Height,
    pub event: IbcEvent<C::ClientId, C::ClientType, String>,
}

macro_rules! chain_client_id {
    (
        #[ty = $Ty:ident]
        pub enum $Enum:ident {
            $(
                // will wrap in `Id<_>`
                #[id(ty = $ty:literal)]
                $Variant:ident(Id<_>),
            )+
        }
    ) => {
        #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(untagged)]
        pub enum $Enum {
            $(
                $Variant(Id<$Variant>),
            )+
        }

        const _: () = {
            const STRINGS: &[&str] = &[$($ty),*];

            const LEN: usize = STRINGS.len();

            let mut i = 0;
            let mut j = 0;

            while i < LEN {
                while j < LEN {
                    if i == j {
                        j += 1;
                        continue;
                    } else {
                        assert!(!const_str_equal(STRINGS[i], STRINGS[j]), "strings are not unique");
                    }
                    j += 1;
                }
                i += 1;
            }

            // https://internals.rust-lang.org/t/why-i-cannot-compare-two-static-str-s-in-a-const-context/17726/8
            const fn const_bytes_equal(lhs: &[u8], rhs: &[u8]) -> bool {
                if lhs.len() != rhs.len() {
                    return false;
                }
                let mut i = 0;
                while i < lhs.len() {
                    if lhs[i] != rhs[i] {
                        return false;
                    }
                    i += 1;
                }
                true
            }

            const fn const_str_equal(lhs: &str, rhs: &str) -> bool {
                const_bytes_equal(lhs.as_bytes(), rhs.as_bytes())
            }
        };

        $(
            impl From<Id<$Variant>> for $Enum {
                fn from(id: Id<$Variant>) -> Self {
                    Self::$Variant(id)
                }
            }

            impl TryFrom<$Enum> for Id<$Variant> {
                type Error = $Enum;

                fn try_from(value: $Enum) -> Result<Self, Self::Error> {
                    match value {
                        $Enum::$Variant(id) => Ok(id),
                        #[allow(unreachable_patterns)] // for when new types are added
                        _ => Err(value),
                    }
                }
            }

            impl From<$Variant> for $Ty {
                fn from(id: $Variant) -> Self {
                    Self::$Variant(id)
                }
            }

            impl TryFrom<$Ty> for $Variant {
                type Error = $Ty;

                fn try_from(value: $Ty) -> Result<Self, Self::Error> {
                    match value {
                        $Ty::$Variant(id) => Ok(id),
                        #[allow(unreachable_patterns)] // for when new types are added
                        _ => Err(value),
                    }
                }
            }
        )+

        #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(untagged)]
        pub enum $Ty {
            $(
                $Variant($Variant),
            )+
        }

        $(
            id_type! {
                #[ty = $ty]
                pub struct $Variant;
            }
        )+

        impl crate::Id for $Enum {
            type FromStrErr = crate::ChainClientIdParseError;
        }

        impl FromStr for $Enum {
            type Err = crate::ChainClientIdParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $(
                    if let Ok(ok) = s.parse::<Id<$Variant>>().map(Self::$Variant) {
                        return Ok(ok);
                    }
                )+

                Err(crate::ChainClientIdParseError {
                    expected: &[$($ty),+],
                    found: s.to_string(),
                })
            }
        }

        impl Display for $Enum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$Variant(id) => f.write_fmt(format_args!("{id}")),
                    )+
                }
            }
        }


        impl FromStr for $Ty {
            type Err = crate::ChainClientIdParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        $ty => Ok(Self::$Variant($Variant)),
                    )+
                    _ => Err(crate::ChainClientIdParseError {
                        expected: &[$($ty),+],
                        found: s.to_string(),
                    })
                }
            }
        }

        impl Display for $Ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$Variant(_) => f.write_str($ty),
                    )+
                }
            }
        }
    };
}

pub(crate) use chain_client_id;

// TODO: Make this a more generic error and put it in unionlabs::errors
#[derive(Debug, Clone, PartialEq)]
pub struct ChainClientIdParseError {
    expected: &'static [&'static str],
    found: String,
}

impl Display for ChainClientIdParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "expected one of `{}`, found `{}`",
            self.expected
                .iter()
                .map(|exp| format!("`{exp}`"))
                .collect::<Vec<_>>()
                .join(","),
            self.found,
        ))
    }
}

impl Error for ChainClientIdParseError {}

#[derive(Debug, Clone)]
pub struct Pool<T> {
    pool: Arc<ArrayQueue<T>>,
}

impl<T: Clone> Pool<T> {
    pub fn new(ts: impl ExactSizeIterator<Item = T>) -> Self {
        let data = ArrayQueue::new(ts.len());

        for t in ts {
            data.push(t)
                .map_err(|_| ())
                .expect("queue is initialized with the correct length; qed;");
        }

        Self {
            pool: Arc::new(data),
        }
    }

    pub async fn with<R, F: FnOnce(T) -> Fut, Fut: Future<Output = R>>(&self, f: F) -> R {
        let t = loop {
            match self.pool.pop() {
                Some(t) => break t,
                None => {
                    const RETRY_SECONDS: u64 = 3;

                    tracing::warn!(
                        "high traffic in queue of {}, ran out of items! trying again in {RETRY_SECONDS} seconds",
                        std::any::type_name::<T>()
                    );

                    tokio::time::sleep(std::time::Duration::from_secs(RETRY_SECONDS)).await;

                    continue;
                }
            }
        };

        // TODO: Figure out a way to pass this as ref
        let r = f(t.clone()).await;

        self.pool
            .push(t)
            .map_err(|_| ())
            .expect("no additional items are added; qed;");

        r
    }
}

pub trait MaybeRecoverableError: Error {
    fn is_recoverable(&self) -> bool;
}

fn _is_object_safe(_: &dyn MaybeRecoverableError) {}
