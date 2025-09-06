pragma solidity ^0.8.27;

struct ZkgmPacket {
    bytes32 salt;
    uint256 path;
    Instruction instruction;
}

struct Instruction {
    uint8 version;
    uint8 opcode;
    bytes operand;
}

struct Forward {
    uint256 path;
    uint64 timeoutHeight;
    uint64 timeoutTimestamp;
    Instruction instruction;
}

struct Call {
    bytes sender;
    bool eureka;
    bytes contractAddress;
    bytes contractCalldata;
}

struct Batch {
    Instruction[] instructions;
}

struct TokenOrderV1 {
    bytes sender;
    bytes receiver;
    bytes baseToken;
    uint256 baseAmount;
    string baseTokenSymbol;
    string baseTokenName;
    uint8 baseTokenDecimals;
    uint256 baseTokenPath;
    bytes quoteToken;
    uint256 quoteAmount;
}

struct TokenOrderV2 {
    bytes sender;
    bytes receiver;
    bytes baseToken;
    uint256 baseAmount;
    bytes quoteToken;
    uint256 quoteAmount;
    uint8 kind;
    bytes metadata;
}

struct TokenMetadata {
    bytes implementation;
    bytes initializer;
}

struct SolverMetadata {
    bytes solverAddress;
    bytes metadata;
}

struct Ack {
    uint256 tag;
    bytes innerAck;
}

struct BatchAck {
    bytes[] acknowledgements;
}

struct TokenOrderAck {
    uint256 fillType;
    bytes marketMaker;
}

struct V1ToV2Migration {
    uint256 path;
    uint32 channelId;
    address baseToken;
    bytes quoteToken;
}

struct V1ToV2WrappedTokenMigration {
    uint256 path;
    uint32 channelId;
    bytes baseToken;
    address quoteToken;
}
