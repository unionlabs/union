use alloy::primitives::U256;

pub const L1_NEXT_NODE_NUM_SLOT: U256 = U256::from_limbs([117, 0, 0, 0]);
pub const L1_NEXT_NODE_NUM_SLOT_OFFSET_BYTES: u64 = 24;
pub const L1_NODES_SLOT: U256 = U256::from_limbs([118, 0, 0, 0]);
pub const L1_NODES_CONFIRM_DATA_OFFSET: u64 = 2;

alloy::sol! {
    // https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/state/GlobalState.sol
    #[derive(Debug)]
    struct GlobalState {
        bytes32[2] bytes32Vals;
        uint64[2] u64Vals;
    }

    // https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/state/Machine.sol
    #[derive(Debug)]
    enum MachineStatus {
        RUNNING,
        FINISHED,
        ERRORED,
        TOO_FAR
    }

    // https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/Node.sol#L10
    #[derive(Debug)]
    struct ExecutionState {
        GlobalState globalState;
        MachineStatus machineStatus;
    }


    // https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/Node.sol#L15
    #[derive(Debug)]
    struct Assertion {
        ExecutionState beforeState;
        ExecutionState afterState;
        uint64 numBlocks;
    }

    // https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/IRollupCore.sol#L26
    #[derive(Debug)]
    event NodeCreated(
        uint64 indexed nodeNum,
        bytes32 indexed parentNodeHash,
        bytes32 indexed nodeHash,
        bytes32 executionHash,
        Assertion assertion,
        bytes32 afterInboxBatchAcc,
        bytes32 wasmModuleRoot,
        uint256 inboxMaxCount
    );
}
