pragma solidity ^0.8.18;

interface IZKVerifierV2 {
    function verifyProof(
        uint256[8] memory proof,
        uint256[2] memory proofCommitment,
        uint256[5] calldata input
    ) external view returns (bool r);
}
