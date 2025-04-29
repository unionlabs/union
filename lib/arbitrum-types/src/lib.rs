/// Constant slot values for the [`RollupCore`] contract on the L1.
///
/// Values can be verified by running the following command:
///
/// ```sh
/// cast storage $ROLLUP_PROXY_ADDRESS
/// ```
///
/// Known [`RollupCore`] proxy addresses:
///
/// - Corn, settling on Ethereum: <https://etherscan.io/address/0x828C71bc1D7A34F32FfA624240633b6B7272C3D6>
/// - Corn Testnet, settling on Sepolia: <https://sepolia.etherscan.io/address/0xD318638594A5B17b50a1389B0c0580576226C0AE>
///   - NOTE: Not verified?
///
/// [`RollupCore`]: https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/RollupCore.sol
pub mod slots {
    use solidity_slot::{MappingKey, Slot, U256};
    use unionlabs_primitives::ByteArrayExt;

    /// The slot containing [`_latestNodeCreated`].
    ///
    /// ```solidity
    /// _latestNodeCreated uint64;
    /// ```
    ///
    /// [`_latestNodeCreated`] https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/RollupCore.sol#L62
    pub const ROLLUP_CORE_LATEST_NODE_CREATED: Slot =
        Slot::Offset(U256::from_limbs([117, 0, 0, 0]));

    /// The offset into the storage layout of the slot containing [`_latestNodeCreated`].
    ///
    /// [`_latestNodeCreated`]: https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/RollupCore.sol#L62
    pub const ROLLUP_CORE_LATEST_NODE_CREATED_SLOT_OFFSET_BYTES: u8 = 16;

    /// The base slot of the [`_nodes`] mapping.
    ///
    /// ```solidity
    /// _nodes mapping(uint64 => struct Node);
    /// ```
    ///
    /// Use [`nodes()`] to calculate the slot for the mapping.
    ///
    /// [`_nodes`] https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/RollupCore.sol#64
    pub const ROLLUP_CORE_NODES_MAPPING_BASE: Slot = Slot::Offset(U256::from_limbs([118, 0, 0, 0]));

    /// The offset of the `confirmData` field in the storage layout of the [`Node`] struct.
    ///
    /// The node value is stored in the mapping at [`ROLLUP_CORE_NODES_MAPPING_BASE_SLOT`].
    ///
    /// [`Node`]: https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/Node.sol#L21-L46
    pub const NODE_CONFIRM_DATA_OFFSET: U256 = U256::from_limbs([2, 0, 0, 0]);

    /// Calculate the slot of the [`confirmData`] field of the `Node` struct stored in the `_nodes`
    /// mapping.
    ///
    /// ```solidity
    /// _nodes mapping(uint64 => struct Node);
    /// ```
    ///
    /// The base slot for this mapping is [`ROLLUP_CORE_NODES_MAPPING_BASE_SLOT`], and the
    /// `confirmData` offset is [`NODE_CONFIRM_DATA_OFFSET`].
    ///
    /// [`confirmData`]: https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/Node.sol#L27
    pub fn rollup_core_nodes_confirm_data_slot(node_num: u64) -> U256 {
        Slot::StructOffset(
            &Slot::Mapping(
                &ROLLUP_CORE_NODES_MAPPING_BASE,
                MappingKey::Uint64(node_num),
            ),
            NODE_CONFIRM_DATA_OFFSET,
        )
        .slot()
    }

    /// Read the value of the `_latestNodeCreated` in the provided storage slot.
    pub fn read_latest_node_created(value: U256) -> u64 {
        u64::from_be_bytes(
            value
                .to_be_bytes()
                // values are packed in the slot *right-aligned*
                //
                // (1_u16, 2_u16, 3_u16) is stored as:
                //
                // 0x0000000000000000000000000000000000000000000000000000**000300020001**
                //
                // NOT:
                //
                // 0x**000100020003**0000000000000000000000000000000000000000000000000000
                //
                // the offsets of the values in the slot are:
                //
                // 0x0000000000000000000000000000000000000000000000000000**000300020001**
                //                                                            4   2   0
                //
                // note that offsets are from the *end* of the slot
                //
                // therefore, we read sizeof(T) bytes at (32 - (offset + sizeof(T)))
                //
                // TODO: Move this functionality to solidity-slot somehow
                .array_slice::<{
                    32 - ((ROLLUP_CORE_LATEST_NODE_CREATED_SLOT_OFFSET_BYTES as usize)
                        + size_of::<u64>())
                }, { size_of::<u64>() }>(),
        )
    }
}

alloy_sol_types::sol! {
    /// <https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/state/GlobalState.sol>
    #[derive(Debug)]
    struct GlobalState {
        bytes32[2] bytes32Vals;
        uint64[2] u64Vals;
    }

    /// <https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/state/Machine.sol>
    #[derive(Debug)]
    enum MachineStatus {
        RUNNING,
        FINISHED,
        ERRORED,
        TOO_FAR
    }

    /// <https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/Node.sol#L10>
    #[derive(Debug)]
    struct ExecutionState {
        GlobalState globalState;
        MachineStatus machineStatus;
    }


    /// <https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/Node.sol#L15>
    #[derive(Debug)]
    struct Assertion {
        ExecutionState beforeState;
        ExecutionState afterState;
        uint64 numBlocks;
    }

    /// <https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/IRollupCore.sol#L26>
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

#[cfg(test)]
mod tests {
    use crate::slots::read_latest_node_created;

    #[test]
    fn read_latest_node_created_correct_value() {
        assert_eq!(
            read_latest_node_created(solidity_slot::U256::from_be_bytes(hex_literal::hex!(
                "000000000143dd37000000000000011c00000000000001120000000000000111"
            )),),
            0x000000000000011c,
        )
    }
}
