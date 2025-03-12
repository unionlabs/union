use serde::{Deserialize, Serialize};
use unionlabs_primitives::Bytes;

use crate::bech32::Bech32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "permission")]
pub enum AccessConfig {
    #[serde(rename = "Nobody")]
    Nobody,
    #[serde(rename = "Everybody")]
    Everybody,
    #[serde(rename = "AnyOfAddresses")]
    AnyOfAddresses { addresses: Vec<Bech32<Bytes>> },
}

#[cfg(feature = "proto")]
pub mod proto {
    use core::str::FromStr;

    use unionlabs_primitives::Bytes;

    use super::AccessConfig;
    use crate::{bech32::Bech32, impl_proto_via_try_from_into};

    impl_proto_via_try_from_into!(AccessConfig => protos::cosmwasm::wasm::v1::AccessConfig);

    impl From<AccessConfig> for protos::cosmwasm::wasm::v1::AccessConfig {
        fn from(value: AccessConfig) -> Self {
            match value {
                AccessConfig::Nobody => Self {
                    permission: protos::cosmwasm::wasm::v1::AccessType::Nobody.into(),
                    ..Default::default()
                },
                AccessConfig::Everybody => Self {
                    permission: protos::cosmwasm::wasm::v1::AccessType::Everybody.into(),
                    ..Default::default()
                },
                AccessConfig::AnyOfAddresses { addresses } => Self {
                    permission: protos::cosmwasm::wasm::v1::AccessType::AnyOfAddresses.into(),
                    addresses: addresses.into_iter().map(|a| a.to_string()).collect(),
                },
            }
        }
    }

    impl TryFrom<protos::cosmwasm::wasm::v1::AccessConfig> for AccessConfig {
        type Error = Error;

        fn try_from(value: protos::cosmwasm::wasm::v1::AccessConfig) -> Result<Self, Self::Error> {
            // TODO: Use const patterns once stable
            const NOBODY: i32 = protos::cosmwasm::wasm::v1::AccessType::Nobody as i32;
            const EVERYBODY: i32 = protos::cosmwasm::wasm::v1::AccessType::Everybody as i32;
            const ANY_OF_ADDRESSES: i32 =
                protos::cosmwasm::wasm::v1::AccessType::AnyOfAddresses as i32;

            match value.permission {
                NOBODY => {
                    if value.addresses.is_empty() {
                        Ok(Self::Nobody)
                    } else {
                        Err(Error::UnexpectedAddresses(value.addresses))
                    }
                }
                EVERYBODY => {
                    if value.addresses.is_empty() {
                        Ok(Self::Everybody)
                    } else {
                        Err(Error::UnexpectedAddresses(value.addresses))
                    }
                }
                ANY_OF_ADDRESSES => Ok(Self::AnyOfAddresses {
                    addresses: value
                        .addresses
                        .into_iter()
                        .map(|s| s.parse())
                        .collect::<Result<_, _>>()?,
                }),
                other => Err(Error::AccessType(other)),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid address")]
        Addresses(#[from] <Bech32<Bytes> as FromStr>::Err),
        #[error("addresses are only valid for AnyOfAddresses, found {0:?}")]
        UnexpectedAddresses(Vec<String>),
        #[error("invalid access type `{0}`")]
        AccessType(i32),
    }
}
