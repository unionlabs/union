pragma solidity ^0.8.27;

library UnionIcs23 {
    struct ExistenceProof {
        bytes key;
        bytes value;
        bytes leafPrefix;
        InnerOp[] path;
    }

    struct NonExistenceProof {
        bytes key;
        ExistenceProof left;
        ExistenceProof right;
    }

    struct InnerOp {
        bytes prefix;
        bytes suffix;
    }

    struct ProofSpec {
        uint256 childSize;
        uint256 minPrefixLength;
        uint256 maxPrefixLength;
    }

    // Original spec is as follows, we trimmed the common parts:
    //     int32[] memory childOrder = new int32[](2);
    //     childOrder[0] = 0;
    //     childOrder[1] = 1;
    //     iavlProofSpec = CosmosIcs23V1ProofSpec.Data({
    //         leaf_spec: CosmosIcs23V1LeafOp.Data({
    //             prefix: hex"00",
    //             prehash_key: CosmosIcs23V1GlobalEnums.HashOp.NO_HASH,
    //             hash: CosmosIcs23V1GlobalEnums.HashOp.SHA256,
    //             prehash_value: CosmosIcs23V1GlobalEnums.HashOp.SHA256,
    //             length: CosmosIcs23V1GlobalEnums.LengthOp.VAR_PROTO
    //         }),
    //         inner_spec: CosmosIcs23V1InnerSpec.Data({
    //             child_order: childOrder,
    //             child_size: 33,
    //             min_prefix_length: 4,
    //             max_prefix_length: 12,
    //             empty_child: abi.encodePacked(),
    //             hash: CosmosIcs23V1GlobalEnums.HashOp.SHA256
    //         }),
    //         min_depth: 0,
    //         max_depth: 0
    //     });
    function getIavlProofSpec() internal pure returns (ProofSpec memory) {
        return
            ProofSpec({childSize: 33, minPrefixLength: 4, maxPrefixLength: 12});
    }

    // Original spec is as follows, we trimmed the common parts:
    //     int32[] memory childOrder = new int32[](2);
    //     childOrder[0] = 0;
    //     childOrder[1] = 1;
    //     tendermintProofSpec = CosmosIcs23V1ProofSpec.Data({
    //         leaf_spec: CosmosIcs23V1LeafOp.Data({
    //             prefix: hex"00",
    //             prehash_key: CosmosIcs23V1GlobalEnums.HashOp.NO_HASH,
    //             hash: CosmosIcs23V1GlobalEnums.HashOp.SHA256,
    //             prehash_value: CosmosIcs23V1GlobalEnums.HashOp.SHA256,
    //             length: CosmosIcs23V1GlobalEnums.LengthOp.VAR_PROTO
    //         }),
    //         inner_spec: CosmosIcs23V1InnerSpec.Data({
    //             child_order: childOrder,
    //             child_size: 32,
    //             min_prefix_length: 1,
    //             max_prefix_length: 1,
    //             empty_child: abi.encodePacked(),
    //             hash: CosmosIcs23V1GlobalEnums.HashOp.SHA256
    //         }),
    //         min_depth: 0,
    //         max_depth: 0
    //     });
    function getTendermintProofSpec()
        internal
        pure
        returns (ProofSpec memory)
    {
        return
            ProofSpec({childSize: 32, minPrefixLength: 1, maxPrefixLength: 1});
    }

    function empty(
        NonExistenceProof calldata proof
    ) internal pure returns (bool) {
        if (proof.key.length != 0) {
            return false;
        }

        return empty(proof.left) && empty(proof.right);
    }

    function empty(
        ExistenceProof calldata proof
    ) internal pure returns (bool) {
        if (proof.key.length != 0) {
            return false;
        }

        if (proof.value.length != 0) {
            return false;
        }

        if (proof.leafPrefix.length != 0) {
            return false;
        }

        if (proof.path.length != 0) {
            return false;
        }

        return true;
    }
}
