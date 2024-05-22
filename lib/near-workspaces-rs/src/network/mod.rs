//! All builtin network types and traits.
//!
//! Currently the builtin network types are [`Mainnet`], [`Testnet`], and [`Sandbox`].

mod config;
mod info;
mod sandbox;
mod server;

pub(crate) mod builder;
pub(crate) mod variants;

pub mod betanet;
pub mod custom;
pub mod mainnet;
pub mod testnet;

pub(crate) use variants::DEV_ACCOUNT_SEED;

pub use self::betanet::Betanet;
pub use self::custom::Custom;
pub use self::info::Info;
pub use self::mainnet::Mainnet;
pub use self::sandbox::Sandbox;
pub use self::server::{pick_unused_port, ValidatorKey};
pub use self::testnet::Testnet;
pub use self::variants::{
    AllowDevAccountCreation, NetworkClient, NetworkInfo, TopLevelAccountCreator,
};
