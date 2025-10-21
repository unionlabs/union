use ethereum_light_client_types::{AccountProof, StorageProof};
use unionlabs::{ibc::core::client::height::Height, primitives::H256, tuple::AsTuple};

use crate::L2Header;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum Header {
    V1(HeaderV1),
    V2(HeaderV2),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct HeaderV1 {
    pub l1_height: Height,
    pub l1_account_proof: AccountProof,
    pub l2_ibc_account_proof: AccountProof,
    pub l1_next_node_num_slot_proof: StorageProof,
    pub l1_nodes_slot_proof: StorageProof,
    pub l2_header: L2Header,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct HeaderV2 {
    pub l1_height: Height,

    /// Proof of the `l1_contract_address` in the state root of the L1.
    pub l1_account_proof: AccountProof,

    pub l2_ibc_account_proof: AccountProof,

    pub parent_assertion_hash: H256,
    pub assertion_state: AssertionState,
    pub inbox_acc: H256,

    pub l1_assertions_proof: StorageProof,

    pub l2_header: L2Header,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct AssertionState {
    pub global_state: GlobalState,
    pub machine_status: MachineStatus,
    pub end_history_root: H256,
}

#[derive(Debug, Clone, PartialEq, AsTuple)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct GlobalState {
    /// NOTE: `[0]` is the L2 block hash.
    pub bytes32_vals: [H256; 2],
    pub u64_vals: [u64; 2],
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum MachineStatus {
    Running,
    Finished,
    Errored,
}

#[cfg(feature = "ethabi")]
impl AssertionState {
    pub fn hash(&self) -> H256 {
        use alloy::{primitives::keccak256, sol_types::SolValue};

        keccak256(
            arbitrum_types::v2::AssertionState {
                globalState: arbitrum_types::v2::GlobalState {
                    bytes32Vals: self.global_state.bytes32_vals.map(Into::into),
                    u64Vals: self.global_state.u64_vals,
                },
                machineStatus: match self.machine_status {
                    MachineStatus::Running => arbitrum_types::v2::MachineStatus::RUNNING,
                    MachineStatus::Finished => arbitrum_types::v2::MachineStatus::FINISHED,
                    MachineStatus::Errored => arbitrum_types::v2::MachineStatus::ERRORED,
                },
                endHistoryRoot: self.end_history_root.into(),
            }
            .abi_encode(),
        )
        .into()
    }
}

#[cfg(test)]
mod tests {
    use arbitrum_types::v2::assertion_hash;
    use hex_literal::hex;

    use super::*;

    // https://dashboard.tenderly.co/tx/0x6d5ee1b3f3da458f6d5c805460b0f8cb10906b5f7ff16c2f304ef75eff7df308/logs
    #[test]
    fn assertion_hash_works() {
        let assertion_state = AssertionState {
            global_state: GlobalState {
                bytes32_vals: [
                    hex!("7f0885fd65e8802bfd17ca397f2c4c642efe8472c5966c6a5e27b1665597493b").into(),
                    hex!("baf4c82c9da2bf7a219e04ab87b6539a15d2bd8f1500be7550a31c5f2d09ae6f").into(),
                ],
                u64_vals: [1039954, 0],
            },
            machine_status: MachineStatus::Finished,
            end_history_root: hex!(
                "971163f4a9507eca033fc492c106e96a4b0649c4e06de388879722ef141a0d0a"
            )
            .into(),
        };

        let ah = <H256>::new(hex!(
            "ba7b69f71edcf8daa1d2a7b383a57c080009d90cac65b283e2c7789aa888cedd"
        ));

        assert_eq!(
            ah,
            assertion_hash(
                hex!("7153d0e5fb235e7fd4ee5b87f20818ed88cc1a1c786baf33e1dfe2f8e215e952").into(),
                assertion_state.hash(),
                hex!("2a57d18325c2c245a3b4c3b0302e2915bb558a351b1cb7cb5f7630c34320910a").into()
            )
        );
    }
}
