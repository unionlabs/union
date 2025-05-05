use consensus_primitives::Duration;
use unionlabs::{ibc::core::client::height::Height, primitives::H256};

use crate::chain_id::ChainId;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientState {
    pub chain_id: ChainId,
    pub trusting_period: Duration,
    pub max_clock_drift: Duration,
    /// This field only ever has one of two values:
    ///
    /// - 0: client is not frozen
    /// - 1: client is frozen
    ///
    /// Both the field name and type match the ICS07 Tendermint implementation.
    ///
    /// Note that the above bounds are not enforced at the type level, which also matches the Tendermint specification.
    pub frozen_height: Height,
    pub latest_height: Height,
    /// For clients that connect to the cosmwasm implementation of ibc-union, the contract address of the IBC host is required in order to verify storage proofs. For clients connecting to IBC classic, this field is not required and can be ignored during client creation and migration.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "H256::is_zero")
    )]
    pub contract_address: H256,
}

#[cfg(feature = "proto")]
pub mod proto {
    use consensus_primitives::Duration;
    use unionlabs::{
        errors::{InvalidLength, MissingField},
        impl_proto_via_try_from_into,
        primitives::H256,
        required,
    };

    use crate::{client_state::ClientState, ChainId};

    impl_proto_via_try_from_into!(ClientState => protos::union::ibc::lightclients::cometbls::v1::ClientState);

    impl From<ClientState> for protos::union::ibc::lightclients::cometbls::v1::ClientState {
        fn from(value: ClientState) -> Self {
            Self {
                chain_id: value.chain_id.to_string(),
                trusting_period: value.trusting_period.as_nanos(),
                max_clock_drift: value.max_clock_drift.as_nanos(),
                frozen_height: Some(value.frozen_height.into()),
                latest_height: Some(value.latest_height.into()),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid chain_id")]
        ChainId(#[from] InvalidLength),
    }

    impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::ClientState> for ClientState {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::cometbls::v1::ClientState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                chain_id: ChainId::from_string(value.chain_id)?,
                trusting_period: Duration::from_secs(value.trusting_period),
                max_clock_drift: Duration::from_secs(value.max_clock_drift),
                frozen_height: required!(value.frozen_height)?.into(),
                latest_height: required!(value.latest_height)?.into(),
                // NOTE: the contract_address is not used for cometbls clients encoded using protobuf. if this field is required, use a different encoding (i.e. bincode or ethabi).
                contract_address: H256::default(),
            })
        }
    }
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use std::string::FromUtf8Error;

    use consensus_primitives::Duration;
    use unionlabs::{ibc::core::client::height::Height, impl_ethabi_via_try_from_into};

    use crate::{ChainId, ClientState};

    alloy::sol! {
        struct SolClientState {
            bytes31 chainId;
            uint64 trustingPeriod;
            uint64 maxClockDrift;
            uint64 frozenHeight;
            uint64 latestHeight;
            bytes32 contractAddress;
        }
    }

    impl_ethabi_via_try_from_into!(ClientState => SolClientState);

    impl From<ClientState> for SolClientState {
        fn from(value: ClientState) -> Self {
            SolClientState {
                chainId: value.chain_id.into_fixed_bytes(),
                trustingPeriod: value.trusting_period.as_nanos(),
                maxClockDrift: value.max_clock_drift.as_nanos(),
                // NOTE: The revision height is not used for cometbls clients encoded using ethabi. If this value is required, use a different encoding (i.e. proto or bincode).
                frozenHeight: value.frozen_height.height(),
                latestHeight: value.latest_height.height(),
                contractAddress: value.contract_address.into(),
            }
        }
    }

    impl TryFrom<SolClientState> for ClientState {
        type Error = Error;

        fn try_from(value: SolClientState) -> Result<Self, Self::Error> {
            Ok(Self {
                chain_id: ChainId::try_from_fixed_bytes(value.chainId)?,
                trusting_period: Duration::from_nanos(value.trustingPeriod),
                max_clock_drift: Duration::from_nanos(value.maxClockDrift),
                frozen_height: Height::new(value.frozenHeight),
                latest_height: Height::new(value.latestHeight),
                contract_address: value.contractAddress.into(),
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid chain_id")]
        ChainId(#[from] FromUtf8Error),
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use unionlabs::{
        encoding::{Bincode, EthAbi, Json, Proto},
        test_utils::{assert_codec_iso, assert_codec_iso_bytes},
    };

    use super::*;

    fn mk_client_state() -> ClientState {
        ClientState {
            chain_id: ChainId::from_string("oogabooga").unwrap(),
            trusting_period: Duration::from_nanos(12345),
            max_clock_drift: Duration::from_nanos(67890),
            frozen_height: Height::default(),
            latest_height: Height::new(1337),
            contract_address: <H256>::from([0xAA; 32]),
        }
    }

    #[test]
    fn bincode_iso() {
        assert_codec_iso::<_, Bincode>(&mk_client_state());
    }

    #[test]
    fn ethabi_iso() {
        assert_codec_iso::<_, EthAbi>(&mk_client_state());
    }

    #[test]
    fn json_iso() {
        assert_codec_iso::<_, Json>(&mk_client_state());
    }

    #[test]
    fn proto_iso() {
        let mut client_state = mk_client_state();

        // this field is lost when encoding to proto since it is not supported in the native ibc-go implementation
        client_state.contract_address = <H256>::from([0x00; 32]);

        assert_codec_iso::<_, Proto>(&client_state);
    }

    #[test]
    fn ethabi_bytes() {
        assert_codec_iso_bytes::<_, EthAbi>(
            &ClientState {
                chain_id: ChainId::from_string("union-1").unwrap(),
                frozen_height: Height::new(0),
                latest_height: Height::new(578192),
                max_clock_drift: Duration::from_nanos(1200000000000),
                trusting_period: Duration::from_nanos(1982880000000000),
                contract_address: hex!(
                    "bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4"
                )
                .into(),
            },
            &hex!("000000000000000000000000000000000000000000000000756e696f6e2d310000000000000000000000000000000000000000000000000000070b6b3a084000000000000000000000000000000000000000000000000000000001176592e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008d290bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4"),
        );
    }

    #[test]
    fn bincode_bytes() {
        assert_codec_iso_bytes::<_, Bincode>(
            &ClientState {
                chain_id: ChainId::from_string("union-1").unwrap(),
                frozen_height: Height::new(0),
                latest_height: Height::new_with_revision(1, 578227),
                max_clock_drift: Duration::from_nanos(1200000000000),
                trusting_period: Duration::from_nanos(1982880000000000),
                contract_address: hex!(
                    "bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4"
                )
                .into(),
            },
            &hex!("0700000000000000756e696f6e2d310040083a6b0b070000e0926517010000000000000000000000010100000000000000b3d2080000000000bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4"),
        );
    }
}
