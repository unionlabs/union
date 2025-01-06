use unionlabs::{ibc::core::client::height::Height, primitives::H256};

use crate::chain_id::ChainId;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientState {
    pub chain_id: ChainId,
    pub trusting_period: u64,
    pub max_clock_drift: u64,
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
    use std::array::TryFromSliceError;

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
                trusting_period: value.trusting_period,
                max_clock_drift: value.max_clock_drift,
                frozen_height: Some(value.frozen_height.into()),
                latest_height: Some(value.latest_height.into()),
            }
        }
    }

    #[derive(Debug, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid chain_id")]
        ChainId(#[from] InvalidLength),
        #[error("invalid contract address")]
        ContractAddress(#[source] TryFromSliceError),
    }

    impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::ClientState> for ClientState {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::cometbls::v1::ClientState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                chain_id: ChainId::from_string(value.chain_id)?,
                trusting_period: value.trusting_period,
                max_clock_drift: value.max_clock_drift,
                frozen_height: required!(value.frozen_height)?.into(),
                latest_height: required!(value.latest_height)?.into(),
                contract_address: H256::default(),
            })
        }
    }
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use std::string::FromUtf8Error;

    use alloy::sol_types::SolValue;
    use unionlabs::{
        encoding::{Decode, Encode, EthAbi},
        ibc::core::client::height::Height,
        TryFromEthAbiBytesErrorAlloy,
    };

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

    impl Encode<EthAbi> for ClientState {
        fn encode(self) -> Vec<u8> {
            SolClientState {
                chainId: self.chain_id.into_fixed_bytes(),
                trustingPeriod: self.trusting_period,
                maxClockDrift: self.max_clock_drift,
                frozenHeight: self.frozen_height.height(),
                latestHeight: self.latest_height.height(),
                contractAddress: self.contract_address.into(),
            }
            .abi_encode_params()
        }
    }

    impl Decode<EthAbi> for ClientState {
        type Error = TryFromEthAbiBytesErrorAlloy<Error>;

        fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
            let client_state = SolClientState::abi_decode(bytes, true)?;

            Ok(Self {
                chain_id: ChainId::try_from_fixed_bytes(client_state.chainId)
                    .map_err(|err| TryFromEthAbiBytesErrorAlloy::Convert(Error::ChainId(err)))?,
                trusting_period: client_state.trustingPeriod,
                max_clock_drift: client_state.maxClockDrift,
                frozen_height: Height::new(client_state.frozenHeight),
                latest_height: Height::new(client_state.latestHeight),
                contract_address: client_state.contractAddress.into(),
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
    use unionlabs::{
        encoding::{Bincode, EthAbi, Json, Proto},
        test_utils::assert_codec_iso,
    };

    use super::*;

    fn mk_client_state() -> ClientState {
        ClientState {
            chain_id: ChainId::from_string("oogabooga").unwrap(),
            trusting_period: 12345,
            max_clock_drift: 67890,
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

        // this field is currently lost when encoding to proto since it is not supported in the native ibc-go implementation
        client_state.contract_address = <H256>::from([0x00; 32]);

        assert_codec_iso::<_, Proto>(&client_state);
    }
}
