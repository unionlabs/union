use beacon_api_types::{ForkParameters, PresetBaseKind};
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{H160, H256, U256},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientState {
    pub chain_id: U256,
    pub chain_spec: PresetBaseKind,
    pub genesis_validators_root: H256,
    pub genesis_time: u64,
    pub fork_parameters: ForkParameters,
    pub latest_height: u64,
    // even though it would be better to have option, ethabicodec don't handle it as zero struct...
    pub frozen_height: Height,
    /// the ibc contract on the counterparty chain that contains the ICS23 commitments
    pub ibc_contract_address: H160,
}

#[cfg(test)]
mod tests {
    use beacon_api_types::{Fork, Slot, Version};
    use unionlabs::{
        encoding::{Bincode, Json},
        primitives::{FixedBytes, H256},
        test_utils::assert_codec_iso,
    };

    use super::*;

    fn mk_client_state() -> ClientState {
        ClientState {
            chain_id: U256::from(1u64),
            chain_spec: PresetBaseKind::Minimal,
            genesis_validators_root: H256::new([0xAA; 32]),
            genesis_time: 123,
            fork_parameters: ForkParameters {
                genesis_fork_version: Version(FixedBytes::new([1, 2, 3, 4])),
                genesis_slot: Slot::new(1),
                altair: Fork {
                    version: Version(FixedBytes::new([2, 3, 4, 5])),
                    epoch: 2,
                },
                bellatrix: Fork {
                    version: Version(FixedBytes::new([3, 4, 5, 6])),
                    epoch: 3,
                },
                capella: Fork {
                    version: Version(FixedBytes::new([4, 5, 6, 7])),
                    epoch: 4,
                },
                deneb: Fork {
                    version: Version(FixedBytes::new([5, 6, 7, 8])),
                    epoch: 5,
                },
                // electra: Fork {
                //     version: Version(FixedBytes::new([6, 7, 8, 9])),
                //     epoch: 6,
                // },
            },
            latest_height: 987,
            frozen_height: Height::new(1),
            ibc_contract_address: H160::new([0xAA; 20]),
        }
    }

    #[test]
    fn bincode_iso() {
        assert_codec_iso::<_, Bincode>(&mk_client_state());
    }

    #[test]
    fn json_iso() {
        assert_codec_iso::<_, Json>(&mk_client_state());
    }
}
