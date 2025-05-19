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

struct Multiplex {
    bytes sender;
    bool eureka;
    bytes contractAddress;
    bytes contractCalldata;
}

struct Batch {
    Instruction[] instructions;
}

struct FungibleAssetOrder {
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

struct Stake {
    uint256 tokenId;
    bytes governanceToken;
    bytes sender;
    bytes beneficiary;
    bytes validator;
    uint256 amount;
}

struct Unstake {
    uint256 tokenId;
    bytes governanceToken;
    bytes sender;
    bytes validator;
    uint256 amount;
}

struct WithdrawStake {
    uint256 tokenId;
    bytes governanceToken;
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

struct FungibleAssetOrderAck {
    uint256 fillType;
    bytes marketMaker;
}

struct UnstakeAck {
    uint256 completionTime;
}

struct WithdrawStakeAck {
    uint256 amount;
}

enum ZkgmStakeState {
    // The position doesn't exist yet.
    UNDEFINED,
    // The tokens are in-flight to be staked.
    STAKING,
    // The tokens are bonded and the position is being rewarded.
    STAKED,
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
    // Validator we staked for.
    bytes validator;
    // Bonded amount.
    uint256 amount;
    // Time at which unbonding will complete (allows the NFT to be burnt in exchange for withdrawal).
    uint256 unstakingCompletion;
}
