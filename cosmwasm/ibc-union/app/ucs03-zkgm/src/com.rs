use alloy_primitives::U256;
use cosmwasm_std::Uint256;
use serde::{Deserialize, Serialize};
use unionlabs::primitives::{Bytes, H256};

pub const INSTR_VERSION_0: u8 = 0x00;
pub const INSTR_VERSION_1: u8 = 0x01;
pub const INSTR_VERSION_2: u8 = 0x02;

pub const OP_FORWARD: u8 = 0x00;
pub const OP_MULTIPLEX: u8 = 0x01;
pub const OP_BATCH: u8 = 0x02;
pub const OP_FUNGIBLE_ASSET_ORDER: u8 = 0x03;

pub const OP_STAKE: u8 = 0x04;
pub const OP_UNSTAKE: u8 = 0x05;
pub const OP_WITHDRAW_STAKE: u8 = 0x06;
pub const OP_WITHDRAW_REWARDS: u8 = 0x07;

pub const ACK_ERR_ONLY_MAKER: &[u8] = &[0xDE, 0xAD, 0xC0, 0xDE];

pub const TAG_ACK_FAILURE: U256 = U256::ZERO;
pub const TAG_ACK_SUCCESS: U256 = U256::from_be_slice(&[1]);

pub const FILL_TYPE_PROTOCOL: U256 = U256::from_be_slice(&[0xB0, 0xCA, 0xD0]);
pub const FILL_TYPE_MARKETMAKER: U256 = U256::from_be_slice(&[0xD1, 0xCE, 0xC4, 0x5E]);

pub const FUNGIBLE_ASSET_METADATA_TYPE_IMAGE: u8 = 0x00;
pub const FUNGIBLE_ASSET_METADATA_TYPE_PREIMAGE: u8 = 0x01;
pub const FUNGIBLE_ASSET_METADATA_TYPE_IMAGE_UNWRAP: u8 = 0x02;

pub const FUNGIBLE_ASSET_METADATA_IMAGE_PREDICT_V1: H256 = H256::new([
    0xC0, 0xDE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE,
    0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE,
]);

pub const FORWARD_SALT_MAGIC: U256 = U256::from_be_slice(&[
    0xC0, 0xDE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xBA, 0xBE,
]);

alloy_sol_types::sol! {
    #[derive(Debug)]
    struct ZkgmPacket {
        bytes32 salt;
        uint256 path;
        Instruction instruction;
    }

    #[derive(Debug)]
    struct Instruction {
        uint8 version;
        uint8 opcode;
        bytes operand;
    }

    struct Forward {
        uint256 path;
        uint64 timeout_height;
        uint64 timeout_timestamp;
        Instruction instruction;
    }

    struct Multiplex {
        bytes sender;
        bool eureka;
        bytes contract_address;
        bytes contract_calldata;
    }

    struct Batch {
        Instruction[] instructions;
    }

    #[derive(Debug, PartialEq)]
    struct FungibleAssetOrder {
        bytes sender;
        bytes receiver;
        bytes base_token;
        uint256 base_amount;
        string base_token_symbol;
        string base_token_name;
        uint8 base_token_decimals;
        uint256 base_token_path;
        bytes quote_token;
        uint256 quote_amount;
    }

    #[derive(Debug, PartialEq)]
    struct FungibleAssetOrderV2 {
        bytes sender;
        bytes receiver;
        bytes base_token;
        uint256 base_amount;
        uint8 metadata_type;
        bytes metadata;
        bytes quote_token;
        uint256 quote_amount;
    }

    #[derive(Debug, PartialEq)]
    struct FungibleAssetMetadata {
        bytes implementation;
        bytes initializer;
    }

    #[derive(Debug, PartialEq)]
    struct Stake {
        uint256 token_id;
        bytes governance_token;
        bytes32 governance_metadata_image;
        bytes sender;
        bytes beneficiary;
        bytes validator;
        uint256 amount;
    }

    struct Unstake {
        uint256 token_id;
        bytes governance_token;
        bytes32 governance_metadata_image;
        bytes sender;
        bytes validator;
    }

    struct WithdrawStake {
        uint256 token_id;
        bytes governance_token;
        bytes32 governance_metadata_image;
        bytes sender;
        bytes beneficiary;
    }

    struct WithdrawRewards {
        uint256 token_id;
        bytes governance_token;
        bytes32 governance_metadata_image;
        bytes validator;
        bytes sender;
        bytes beneficiary;
    }

    #[derive(Debug)]
    struct Ack {
        uint256 tag;
        bytes inner_ack;
    }

    struct BatchAck {
        bytes[] acknowledgements;
    }

    #[derive(Debug)]
    struct FungibleAssetOrderAck {
        uint256 fill_type;
        bytes market_maker;
    }

    struct UnstakeAck {
        uint256 completion_time;
    }

    struct WithdrawStakeAck {
        uint256 amount;
    }

    struct WithdrawRewardsAck {
        uint256 amount;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct CwFungibleAssetOrderV2 {
    pub sender: Bytes,
    pub receiver: Bytes,
    pub base_token: Bytes,
    pub base_amount: Uint256,
    pub metadata_type: u8,
    pub metadata: Bytes,
    pub quote_token: Bytes,
    pub quote_amount: Uint256,
}

impl From<FungibleAssetOrderV2> for CwFungibleAssetOrderV2 {
    fn from(value: FungibleAssetOrderV2) -> Self {
        Self {
            sender: Vec::from(value.sender.0).into(),
            receiver: Vec::from(value.receiver.0).into(),
            base_token: Vec::from(value.base_token.0).into(),
            base_amount: Uint256::from_be_bytes(value.base_amount.to_be_bytes()),
            metadata_type: value.metadata_type,
            metadata: Vec::from(value.metadata.0).into(),
            quote_token: Vec::from(value.quote_token.0).into(),
            quote_amount: Uint256::from_be_bytes(value.quote_amount.to_be_bytes()),
        }
    }
}
