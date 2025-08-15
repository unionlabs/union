use std::error::Error;

use alloy_sol_types::SolType;
use unionlabs_primitives::{H256, U256};

pub use crate::{batch::Batch, call::Call, forward::Forward, root::Root, token_order::TokenOrder};

pub mod batch;
pub mod call;
pub mod forward;
pub mod root;
pub mod stake;
pub mod token_order;
pub mod unstake;
pub mod withdraw_rewards;
pub mod withdraw_stake;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkgmPacket {
    salt: H256,
    path: U256,
    instruction: Root,
}

impl ZkgmPacket {
    pub fn decode(bz: impl AsRef<[u8]>) -> Result<Self> {
        let zkgm_packet = ucs03_zkgm::com::ZkgmPacket::abi_decode_params_validate(bz.as_ref())?;

        Ok(Self {
            salt: zkgm_packet.salt.into(),
            path: zkgm_packet.path.into(),
            instruction: Root::from_raw(zkgm_packet.instruction)?,
        })
    }
}

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

// pub mod abi {
//     alloy_sol_types::sol! {
//         "../../evm/contracts/apps/ucs/03-zkgm/Types.sol"
//     }
// }

// pub mod zkgm_lib_abi {
//     alloy_sol_types::sol! {
//         bytes public constant ACK_EMPTY = "";

//         uint256 public constant ACK_FAILURE = 0x00;
//         uint256 public constant ACK_SUCCESS = 0x01;

//         bytes public constant ACK_ERR_ONLYMAKER = "DEADC0DE";

//         bytes32 public constant ACK_ERR_ONLYMAKER_HASH =
//             keccak256(ACK_ERR_ONLYMAKER);

//         uint256 public constant FILL_TYPE_PROTOCOL = 0xB0CAD0;
//         uint256 public constant FILL_TYPE_MARKETMAKER = 0xD1CEC45E;

//         uint8 public constant TOKEN_ORDER_KIND_INITIALIZE = 0x00;
//         uint8 public constant TOKEN_ORDER_KIND_ESCROW = 0x01;
//         uint8 public constant TOKEN_ORDER_KIND_UNESCROW = 0x02;

//         // Public instructions
//         uint8 public constant OP_FORWARD = 0x00;
//         uint8 public constant OP_CALL = 0x01;
//         uint8 public constant OP_BATCH = 0x02;
//         uint8 public constant OP_TOKEN_ORDER = 0x03;

//         uint8 public constant OP_STAKE = 0x04;
//         uint8 public constant OP_UNSTAKE = 0x05;
//         uint8 public constant OP_WITHDRAW_STAKE = 0x06;
//         uint8 public constant OP_WITHDRAW_REWARDS = 0x07;

//         uint8 public constant WRAPPED_TOKEN_KIND_PROTOCOL = 0x00;
//         uint8 public constant WRAPPED_TOKEN_KIND_THIRD_PARTY = 0x01;

//         uint8 public constant INSTR_VERSION_0 = 0x00;
//         uint8 public constant INSTR_VERSION_1 = 0x01;
//         uint8 public constant INSTR_VERSION_2 = 0x02;

//         bytes32 public constant FORWARD_SALT_MAGIC =
//             0xC0DE00000000000000000000000000000000000000000000000000000000BABE;

//         address public constant NATIVE_TOKEN_ERC_7528_ADDRESS =
//             0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE;

//         string public constant IBC_VERSION_STR = "ucs03-zkgm-0";
//         bytes32 public constant IBC_VERSION = keccak256(bytes(IBC_VERSION_STR));
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode() {
        let bz = hex_literal::hex!("00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002e00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000204b2c85a26bb9fb2400000000000000000000000000000000000000000000000000000000000000000001e00000000000000000000000000000000000000000204b2c85a26bb9fb2400000000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000002c756e696f6e316a6768663236753576777278666e6435376a78657a736e73687a637a777767796878776a706500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014d31e23d60e07fa632334c2a671733a068592c638000000000000000000000000000000000000000000000000000000000000000000000000000000000000000261750000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014ba5ed44733953d79717f6269357c77718c8ba5ed00000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000014ba5ed44733953d79717f6269357c77718c8ba5ed0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");

        let packet = Root::decode(&bz).unwrap();

        dbg!(&packet);
    }
}
