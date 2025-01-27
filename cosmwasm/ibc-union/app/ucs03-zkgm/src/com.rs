use alloy::primitives::U256;

pub const ZKGM_VERSION_0: u8 = 0x00;

pub const OP_FORWARD: u8 = 0x00;
pub const OP_MULTIPLEX: u8 = 0x01;
pub const OP_BATCH: u8 = 0x02;
pub const OP_FUNGIBLE_ASSET_ORDER: u8 = 0x03;

pub const ACK_ERR_ONLY_MAKER: &[u8] = &[0xDE, 0xAD, 0xC0, 0xDE];

pub const TAG_ACK_FAILURE: U256 = U256::ZERO;
pub const TAG_ACK_SUCCESS: U256 = U256::from_be_slice(&[1]);

pub const FILL_TYPE_PROTOCOL: U256 = U256::from_be_slice(&[0xB0, 0xCA, 0xD0]);
pub const FILL_TYPE_MARKETMAKER: U256 = U256::from_be_slice(&[0xD1, 0xCE, 0xC4, 0x5E]);

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
      uint32 channel_id;
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
  struct FungibleAssetOrder {
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

  struct Ack {
      uint256 tag;
      bytes inner_ack;
  }

  struct BatchAck {
      bytes[] acknowledgements;
  }

  struct FungibleAssetOrderAck {
      uint256 fill_type;
      bytes market_maker;
  }
}
