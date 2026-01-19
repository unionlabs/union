use unionlabs_primitives::U256;

pub const INSTR_VERSION_0: u8 = 0x00;
pub const INSTR_VERSION_1: u8 = 0x01;
pub const INSTR_VERSION_2: u8 = 0x02;

pub const OP_FORWARD: u8 = 0x00;
pub const OP_CALL: u8 = 0x01;
pub const OP_BATCH: u8 = 0x02;
pub const OP_TOKEN_ORDER: u8 = 0x03;

pub const TAG_ACK_FAILURE: U256 = U256::ZERO;
pub const TAG_ACK_SUCCESS: U256 = U256::ONE;

pub const FILL_TYPE_PROTOCOL: U256 = U256::from_limbs([0, 0, 0, 0xB0CAD0]);
pub const FILL_TYPE_MARKETMAKER: U256 = U256::from_limbs([0, 0, 0, 0xD1CEC45E]);

pub const TOKEN_ORDER_KIND_INITIALIZE: u8 = 0x00;
pub const TOKEN_ORDER_KIND_ESCROW: u8 = 0x01;
pub const TOKEN_ORDER_KIND_UNESCROW: u8 = 0x02;
pub const TOKEN_ORDER_KIND_SOLVE: u8 = 0x03;

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
    struct TokenOrderV1 {
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
    struct TokenMetadata {
        bytes implementation;
        bytes initializer;
    }

    #[derive(Debug, PartialEq)]
    struct SolverMetadata {
        bytes solverAddress;
        bytes metadata;
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
    struct TokenOrderAck {
        uint256 fill_type;
        bytes market_maker;
    }
}
