pragma solidity ^0.8.18;

interface IZKVerifierV2 {
    function verifyProof(
        uint256[2] memory a,
        uint256[2][2] memory b,
        uint256[2] memory c,
        uint256[5] calldata input,
        uint256[2] memory proofCommitment
    ) external view returns (bool r);
}
