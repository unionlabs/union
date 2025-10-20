/// Types and constant slot values for the [`RollupCore`] contract on the L1.
///
/// Slot values can be verified by running the following command:
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
pub mod v1 {
    use solidity_slot::{MappingKey, Slot, U256};
    use unionlabs_primitives::ByteArrayExt;

    /// The slot containing [`_latestNodeCreated`].
    ///
    /// ```solidity
    /// _latestNodeCreated uint64;
    /// ```
    ///
    /// [`_latestNodeCreated`]: https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/RollupCore.sol#L62
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
    /// [`_nodes`]: https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/RollupCore.sol#64
    pub const ROLLUP_CORE_NODES_MAPPING_BASE: Slot = Slot::Offset(U256::from_limbs([118, 0, 0, 0]));

    /// The offset of the `confirmData` field in the storage layout of the [`Node`] struct.
    ///
    /// The node value is stored in the mapping at [`ROLLUP_CORE_NODES_MAPPING_BASE`].
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
    /// The base slot for this mapping is [`ROLLUP_CORE_NODES_MAPPING_BASE`], and the
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
        use crate::v1::read_latest_node_created;

        #[test]
        fn read_latest_node_created_correct_value() {
            assert_eq!(
                read_latest_node_created(solidity_slot::U256::from_be_bytes(hex_literal::hex!(
                    "000000000143dd37000000000000011c00000000000001120000000000000111"
                ))),
                0x000000000000011c,
            );
        }
    }
}

/// Types and constant slot values for the [`RollupCore`] contract on the L1, after the [BoLD]
/// upgrade.
///
/// Slot values can be verified by running the following command:
///
/// ```sh
/// cast storage $ROLLUP_PROXY_ADDRESS
/// ```
///
/// Known [`RollupCore`] proxy addresses:
///
/// - Arbitrum, settling on Ethereum: <https://etherscan.io/address/0x4DCeB440657f21083db8aDd07665f8ddBe1DCfc0>
/// - Arbitrum Sepolia, settling on Sepolia: <https://sepolia.etherscan.io/address/0x042B2E6C5E99d4c521bd49beeD5E99651D9B0Cf4>
///
/// [`RollupCore`]: https://github.com/OffchainLabs/nitro-contracts/blob/0b8c04e8f5f66fe6678a4f53aa15f23da417260e/src/rollup/RollupCore.sol
/// [BoLD]: https://docs.arbitrum.io/how-arbitrum-works/bold/gentle-introduction
pub mod v2 {
    use alloy_sol_types::SolValue;
    use solidity_slot::{H256, MappingKey, Slot, U256, keccak256};

    /// The base slot of the [`_assertions`] mapping.
    ///
    /// ```solidity
    /// mapping(bytes32 => AssertionNode) private _assertions;
    /// ```
    ///
    /// [`_assertions`]: https://github.com/OffchainLabs/nitro-contracts/blob/0b8c04e8f5f66fe6678a4f53aa15f23da417260e/src/rollup/RollupCore.sol#L103
    pub const ROLLUP_CORE_ASSERTIONS_MAPPING_BASE: Slot =
        Slot::Offset(U256::from_limbs([117, 0, 0, 0]));

    /// The byte offset of the `status` field in the first slot of the storage layout of the
    /// [`AssertionNode`] struct.
    ///
    /// The assertion node value is stored in the mapping at
    /// [`ROLLUP_CORE_ASSERTIONS_MAPPING_BASE`].
    ///
    /// # Explanation
    ///
    /// Given the following assertion for arbitrum mainnet:
    ///
    /// ```sh
    /// $ cast call 0x4DCeB440657f21083db8aDd07665f8ddBe1DCfc0 'getAssertion(bytes32)((uint64,uint64,uint64,bool,uint8,bytes32))' 0x8ddd22a1705b4a85846edb930e9bea085a61cb60b528a4c74708b61f02ebf3fe -r https://eth.drpc.org --block 23596629
    /// (23550467 [2.355e7], 0, 23550208 [2.355e7], true, 2, 0xb4c8df7fdf16b57e388cb0474ebb52ef162c6a6127592cecd7d684a99cb0b17e)
    /// ```
    ///
    /// Which decodes to this struct:
    ///
    /// ```rust
    /// # use arbitrum_types::v2::{AssertionNode, AssertionStatus};
    /// # use hex_literal::hex;
    /// AssertionNode {
    ///     firstChildBlock: 23550467,
    ///     secondChildBlock: 0,
    ///     createdAtBlock: 23550208,
    ///     isFirstChild: true,
    ///     status: AssertionStatus::Pending,
    ///     configHash: hex!(
    ///         "b4c8df7fdf16b57e388cb0474ebb52ef162c6a6127592cecd7d684a99cb0b17e"
    ///     )
    ///     .into(),
    /// };
    /// ```
    ///
    /// The storage slot can be calculated using [`rollup_core_assertions_slot`]:
    ///
    /// ```rust
    /// # use arbitrum_types::v2::rollup_core_assertions_slot;
    /// # use hex_literal::hex;
    /// let slot = rollup_core_assertions_slot(hex!(
    ///     "8ddd22a1705b4a85846edb930e9bea085a61cb60b528a4c74708b61f02ebf3fe"
    /// ).into());
    ///
    /// assert_eq!(
    ///     slot.to_string(),
    ///     "3542958973085992967800882309256516822563740080775677574028415039593964399721",
    /// );
    /// ```
    ///
    /// The value for this can the be read with this call:
    ///
    /// ```sh
    /// $ cast storage -r https://eth.drpc.org 0x4dceb440657f21083db8add07665f8ddbe1dcfc0 3542958973085992967800882309256516822563740080775677574028415039593964399721
    /// 0x0000000000000201000000000167590000000000000000000000000001675a03
    /// ```
    ///
    /// Deconstructing this value, we can see how the values are stored (note that `configHash` is
    /// stored in the next slot):
    ///
    /// ```txt
    /// 0000000000000201000000000167590000000000000000000000000001675a03
    /// ├───────────┼─┼─┼───────────────┼───────────────┼──────────────┤
    /// (unused)    │ │ createdAtBlock  │               firstChildBlock
    ///             │ isFirstChild      secondChildBlock
    ///             status
    /// ```
    ///
    /// And finally, the status can be extracted by indexing into the stored value with this offset:
    ///
    /// ```rust
    /// # use arbitrum_types::v2::ASSERTION_NODE_STATUS_BYTE_OFFSET;
    /// # use hex_literal::hex;
    /// let value = hex!("0000000000000201000000000167590000000000000000000000000001675a03");
    ///
    /// assert_eq!(value[ASSERTION_NODE_STATUS_BYTE_OFFSET], 2);
    /// ```
    ///
    /// [`AssertionNode`]: super::AssertionNode
    pub const ASSERTION_NODE_STATUS_BYTE_OFFSET: usize = 6;

    /// Calculate the slot of the [`confirmData`] field of the `Node` struct stored in the `_nodes`
    /// mapping.
    ///
    /// ```solidity
    /// mapping(bytes32 => AssertionNode) private _assertions;
    /// ```
    ///
    /// The base slot for this mapping is [`ROLLUP_CORE_ASSERTIONS_MAPPING_BASE`].
    ///
    /// [`confirmData`]: https://github.com/OffchainLabs/nitro-contracts/blob/90037b996509312ef1addb3f9352457b8a99d6a6/src/rollup/Node.sol#L27
    pub fn rollup_core_assertions_slot(assertion_hash: H256) -> U256 {
        Slot::Mapping(
            &ROLLUP_CORE_ASSERTIONS_MAPPING_BASE,
            MappingKey::Bytes32(assertion_hash),
        )
        .slot()
    }

    /// ```solidity
    /// function assertionHash(
    ///     bytes32 parentAssertionHash,
    ///     bytes32 afterStateHash,
    ///     bytes32 inboxAcc
    /// ) internal pure returns (bytes32)
    /// ```
    ///
    /// <https://github.com/OffchainLabs/nitro-contracts/blob/0b8c04e8f5f66fe6678a4f53aa15f23da417260e/src/rollup/RollupLib.sol#L32>
    pub fn assertion_hash(
        parent_assertion_hash: H256,
        after_state_hash: H256,
        inbox_acc: H256,
    ) -> H256 {
        keccak256((parent_assertion_hash, after_state_hash, inbox_acc).abi_encode_packed())
    }

    alloy_sol_types::sol! {
        /// <https://github.com/OffchainLabs/nitro-contracts/blob/0b8c04e8f5f66fe6678a4f53aa15f23da417260e/src/rollup/Assertion.sol#L9>
        enum AssertionStatus {
            /// No assertion at this index
            NoAssertion,
            /// Assertion is being computed
            Pending,
            /// Assertion is confirmed
            Confirmed
        }

        /// <https://github.com/OffchainLabs/nitro-contracts/blob/0b8c04e8f5f66fe6678a4f53aa15f23da417260e/src/rollup/Assertion.sol#L18>
        struct AssertionNode {
            /// This value starts at zero and is set to a value when the first child is created. After that it is constant until the assertion is destroyed or the owner destroys pending assertions
            uint64 firstChildBlock;
            /// This value starts at zero and is set to a value when the second child is created. After that it is constant until the assertion is destroyed or the owner destroys pending assertions
            uint64 secondChildBlock;
            /// The block number when this assertion was created
            uint64 createdAtBlock;
            /// True if this assertion is the first child of its prev
            bool isFirstChild;
            /// Status of the Assertion
            AssertionStatus status;
            /// A hash of the context available at the time of this assertions creation. It should contain information that is not specific
            /// to this assertion, but instead to the environment at the time of creation. This is necessary to store on the assertion
            /// as this environment can change and we need to know what it was like at the time this assertion was created. An example
            /// of this is the wasm module root which determines the state transition function on the L2. If the wasm module root
            /// changes we need to know that previous assertions were made under a different root, so that we can understand that they
            /// were valid at the time. So when resolving a challenge by one step, the edge challenge manager finds the wasm module root
            /// that was recorded on the prev of the assertions being disputed and uses it to resolve the one step proof.
            bytes32 configHash;
        }

        /// <https://github.com/OffchainLabs/nitro-contracts/blob/0b8c04e8f5f66fe6678a4f53aa15f23da417260e/src/rollup/Assertion.sol#L39>
        #[derive(Debug)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct BeforeStateData {
            // The assertion hash of the prev of the beforeState(prev)
            bytes32 prevPrevAssertionHash;
            // The sequencer inbox accumulator asserted by the beforeState(prev)
            bytes32 sequencerBatchAcc;
            // below are the components of config hash
            ConfigData configData;
        }

        /// <https://github.com/OffchainLabs/nitro-contracts/blob/0b8c04e8f5f66fe6678a4f53aa15f23da417260e/src/rollup/Assertion.sol#L55>
        #[derive(Debug)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct ConfigData {
            bytes32 wasmModuleRoot;
            uint256 requiredStake;
            address challengeManager;
            uint64 confirmPeriodBlocks;
            uint64 nextInboxPosition;
        }

        /// <https://github.com/OffchainLabs/nitro-contracts/blob/0b8c04e8f5f66fe6678a4f53aa15f23da417260e/src/rollup/Assertion.sol#L48>
        #[derive(Debug)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct AssertionInputs {
            /// Additional data used to validate the before state
            BeforeStateData beforeStateData;
            AssertionState beforeState;
            AssertionState afterState;
        }

        /// <https://github.com/OffchainLabs/nitro-contracts/blob/0b8c04e8f5f66fe6678a4f53aa15f23da417260e/src/rollup/AssertionState.sol#L11>
        #[derive(Debug)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct AssertionState {
            GlobalState globalState;
            MachineStatus machineStatus;
            bytes32 endHistoryRoot;
        }

        /// <https://github.com/OffchainLabs/nitro-contracts/blob/0b8c04e8f5f66fe6678a4f53aa15f23da417260e/src/state/GlobalState.sol#L7>
        #[derive(Debug)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct GlobalState {
            /// NOTE: [0] is the L2 block hash.
            bytes32[2] bytes32Vals;
            uint64[2] u64Vals;
        }

        /// <https://github.com/OffchainLabs/nitro-contracts/blob/0b8c04e8f5f66fe6678a4f53aa15f23da417260e/src/state/Machine.sol#L12>
        #[derive(Debug)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        enum MachineStatus {
            RUNNING,
            FINISHED,
            ERRORED
        }

        /// <https://github.com/OffchainLabs/nitro-contracts/blob/0b8c04e8f5f66fe6678a4f53aa15f23da417260e/src/rollup/IRollupCore.sol#L26>
        #[derive(Debug)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        event AssertionCreated(
            bytes32 indexed assertionHash,
            bytes32 indexed parentAssertionHash,
            AssertionInputs assertion,
            bytes32 afterInboxBatchAcc,
            uint256 inboxMaxCount,
            bytes32 wasmModuleRoot,
            uint256 requiredStake,
            address challengeManager,
            uint64 confirmPeriodBlocks
        );
    }
}
