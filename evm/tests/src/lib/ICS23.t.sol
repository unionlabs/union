// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../../../contracts/lib/ICS23.sol";
import "@openzeppelin/contracts/utils/math/Math.sol";

contract ICS23Tests is Test {
    using Ops for bytes;
    using Proof for UnionIcs23.ExistenceProof;

    bytes32 mockRoot;
    bytes mockPrefix;
    bytes mockKey;
    bytes mockValue;
    UnionIcs23.ProofSpec mockProofSpec;

    function setUp() public {
        // Initialize mock data
        mockRoot = keccak256("mockRoot");
        mockPrefix = "mockPrefix";
        mockKey = "mockKey";
        mockValue = "mockValue";

        // Initialize mock proof spec (example values)
        mockProofSpec = UnionIcs23.ProofSpec({
            minPrefixLength: 0,
            maxPrefixLength: 20,
            childSize: 32
        });
    }
}
