pragma solidity ^0.8.23;

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

    function getIavlProofSpec() internal pure returns (ProofSpec memory) {
        return
            ProofSpec({childSize: 33, minPrefixLength: 4, maxPrefixLength: 12});
    }

    function getTendermintProofSpec()
        internal
        pure
        returns (ProofSpec memory)
    {
        return
            ProofSpec({childSize: 32, minPrefixLength: 1, maxPrefixLength: 1});
    }

    function empty(NonExistenceProof memory proof)
        internal
        pure
        returns (bool)
    {
        if (proof.key.length != 0) {
            return false;
        }

        return empty(proof.left) && empty(proof.right);
    }

    function empty(ExistenceProof memory proof) internal pure returns (bool) {
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
