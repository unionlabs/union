pragma solidity ^0.8.18;

interface IZKVerifierV2 {
    function verifyProof(
        uint256[8] calldata proof,
        uint256[2] calldata proofCommitment,
        uint256[2] calldata proofCommitmentPOK,
        uint256[2] calldata input
    ) external returns (bool);
}
