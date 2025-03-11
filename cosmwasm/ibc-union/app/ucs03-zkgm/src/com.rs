use alloy::{primitives::U256, sol_types::SolValue};

pub const INSTR_VERSION_0: u8 = 0x00;
pub const INSTR_VERSION_1: u8 = 0x01;

pub const OP_FORWARD: u8 = 0x00;
pub const OP_MULTIPLEX: u8 = 0x01;
pub const OP_BATCH: u8 = 0x02;
pub const OP_FUNGIBLE_ASSET_ORDER: u8 = 0x03;

pub const ACK_ERR_ONLY_MAKER: &[u8] = &[0xDE, 0xAD, 0xC0, 0xDE];

pub const TAG_ACK_FAILURE: U256 = U256::ZERO;
pub const TAG_ACK_SUCCESS: U256 = U256::from_be_slice(&[1]);

pub const FILL_TYPE_PROTOCOL: U256 = U256::from_be_slice(&[0xB0, 0xCA, 0xD0]);
pub const FILL_TYPE_MARKETMAKER: U256 = U256::from_be_slice(&[0xD1, 0xCE, 0xC4, 0x5E]);

pub const FORWARD_SALT_MAGIC: U256 = U256::from_be_slice(&[
    0xC0, 0xDE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xBA, 0xBE,
]);

alloy::sol! {
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

  #[derive(Debug)]
  struct FungibleAssetOrderV0 {
      bytes sender;
      bytes receiver;
      bytes base_token;
      uint256 base_amount;
      string base_token_symbol;
      string base_token_name;
      uint256 base_token_path;
      bytes quote_token;
      uint256 quote_amount;
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
}

impl From<FungibleAssetOrderV0> for FungibleAssetOrder {
    fn from(value: FungibleAssetOrderV0) -> Self {
        FungibleAssetOrder {
            sender: value.sender,
            receiver: value.receiver,
            base_token: value.base_token,
            base_amount: value.base_amount,
            base_token_symbol: value.base_token_symbol,
            base_token_name: value.base_token_name,
            base_token_decimals: 0,
            base_token_path: value.base_token_path,
            quote_token: value.quote_token,
            quote_amount: value.quote_amount,
        }
    }
}

pub fn decode_fungible_asset(
    instruction: &Instruction,
) -> Result<FungibleAssetOrder, alloy::sol_types::Error> {
    match instruction.version {
        INSTR_VERSION_0 => {
            FungibleAssetOrderV0::abi_decode_params(&instruction.operand, true).map(Into::into)
        }
        INSTR_VERSION_1 => FungibleAssetOrder::abi_decode_params(&instruction.operand, true),
        _ => panic!("the protocol must handle an incorrect version"),
    }
}

pub fn decode_fungible_asset_order_from_v0(
    data: &[u8],
) -> Result<FungibleAssetOrder, alloy::sol_types::Error> {
    FungibleAssetOrderV0::abi_decode_params(data, true).map(Into::into)
}
