#![feature(trait_alias)]

// pub mod arbitrum;
// pub mod berachain;
pub mod cosmos;
pub mod ethereum;
// pub mod scroll;
pub mod union;

pub mod cosmos_sdk;

pub mod private_key;

pub mod keyring;

pub type BoxDynError = Box<dyn core::error::Error + Send + Sync + 'static>;
