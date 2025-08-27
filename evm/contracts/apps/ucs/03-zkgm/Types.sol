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

struct Stake {
    uint256 tokenId;
    address stakedToken;
    bytes validator;
    bytes sender;
    bytes beneficiary;
    uint256 amount;
}

struct Unstake {
    uint256 tokenId;
    bytes validator;
    bytes sender;
}

struct WithdrawStake {
    uint256 tokenId;
    bytes sender;
    bytes beneficiary;
}

struct WithdrawRewards {
    uint256 tokenId;
    bytes validator;
    bytes sender;
    bytes beneficiary;
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

struct UnstakeAck {
    uint256 completionTime;
}

struct WithdrawStakeAck {
    uint256 amount;
}

struct WithdrawRewardsAck {
    uint256 amount;
}

enum ZkgmStakeState {
    // The position doesn't exist yet.
    UNDEFINED,
    // The tokens are in-flight to be staked.
    STAKING,
    // The tokens are bonded and the position is being rewarded.
    STAKED,
    // The rewards are being withdrawn.
    WITHDRAWING_REWARDS,
    // The tokens are being unbonded, the position no longer earns rewards.
    UNSTAKING,
    // The tokens has been unstaked and withdrawn.
    UNSTAKED
}

struct ZkgmStake {
    // Staking position state.
    ZkgmStakeState state;
    // The channel we staked on.
    uint32 channelId;
    // The staked token.
    address stakedToken;
    // Validator we staked for.
    bytes validator;
    // Staked amount.
    uint256 amount;
    // Time at which unstaking will complete (allows the NFT to be burnt in exchange for withdrawal).
    uint256 unstakingCompletion;
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
