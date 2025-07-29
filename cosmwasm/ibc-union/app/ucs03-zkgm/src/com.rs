use alloy_primitives::U256;
use cosmwasm_std::Uint256;
use serde::{Deserialize, Serialize};
use unionlabs::primitives::Bytes;

pub const INSTR_VERSION_0: u8 = 0x00;
pub const INSTR_VERSION_1: u8 = 0x01;
pub const INSTR_VERSION_2: u8 = 0x02;

pub const OP_FORWARD: u8 = 0x00;
pub const OP_CALL: u8 = 0x01;
pub const OP_BATCH: u8 = 0x02;
pub const OP_TOKEN_ORDER: u8 = 0x03;

pub const OP_STAKE: u8 = 0x04;
pub const OP_UNSTAKE: u8 = 0x05;
pub const OP_WITHDRAW_STAKE: u8 = 0x06;
pub const OP_WITHDRAW_REWARDS: u8 = 0x07;

pub const ACK_ERR_ONLY_MAKER: &[u8] = &[0xDE, 0xAD, 0xC0, 0xDE];

pub const TAG_ACK_FAILURE: U256 = U256::ZERO;
pub const TAG_ACK_SUCCESS: U256 = U256::from_be_slice(&[1]);

pub const FILL_TYPE_PROTOCOL: U256 = U256::from_be_slice(&[0xB0, 0xCA, 0xD0]);
pub const FILL_TYPE_MARKETMAKER: U256 = U256::from_be_slice(&[0xD1, 0xCE, 0xC4, 0x5E]);

pub const TOKEN_ORDER_KIND_INITIALIZE: u8 = 0x00;
pub const TOKEN_ORDER_KIND_ESCROW: u8 = 0x01;
pub const TOKEN_ORDER_KIND_UNESCROW: u8 = 0x02;

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
        // TODO: Forward v2 to remove this field
        uint64 timeout_height;
        uint64 timeout_timestamp;
        Instruction instruction;
    }

    struct Call {
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
    struct TokenOrderV2 {
        bytes sender;
        bytes receiver;
        bytes base_token;
        uint256 base_amount;
        bytes quote_token;
        uint256 quote_amount;
        uint8 kind;
        bytes metadata;
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
        bytes governance_token_wrapped;
        bytes sender;
        bytes beneficiary;
        bytes validator;
        uint256 amount;
    }

    struct Unstake {
        uint256 token_id;
        bytes governance_token;
        bytes governance_token_wrapped;
        bytes sender;
        bytes validator;
    }

    struct WithdrawStake {
        uint256 token_id;
        bytes governance_token;
        bytes governance_token_wrapped;
        bytes sender;
        bytes beneficiary;
    }

    struct WithdrawRewards {
        uint256 token_id;
        bytes governance_token;
        bytes governance_token_wrapped;
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
pub struct CwTokenOrderV2 {
    pub sender: Bytes,
    pub receiver: Bytes,
    pub base_token: Bytes,
    pub base_amount: Uint256,
    pub quote_token: Bytes,
    pub quote_amount: Uint256,
    pub kind: u8,
    pub metadata: Bytes,
}

impl From<TokenOrderV2> for CwTokenOrderV2 {
    fn from(value: TokenOrderV2) -> Self {
        Self {
            sender: Vec::from(value.sender.0).into(),
            receiver: Vec::from(value.receiver.0).into(),
            base_token: Vec::from(value.base_token.0).into(),
            base_amount: Uint256::from_be_bytes(value.base_amount.to_be_bytes()),
            quote_token: Vec::from(value.quote_token.0).into(),
            quote_amount: Uint256::from_be_bytes(value.quote_amount.to_be_bytes()),
            kind: value.kind,
            metadata: Vec::from(value.metadata.0).into(),
        }
    }
}
