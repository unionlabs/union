// SPDX-License-Identifier: TBD

pragma solidity ^0.8.18;

import "../proto/ibc/core/commitment/v1/commitment.sol";
import "../proto/ibc/lightclients/tendermint/v1/tendermint.sol";
import "../proto/tendermint/types/types.sol";
import "../proto/tendermint/types/validator.sol";
import "./Encoder.sol";
import "./MerkleTree.sol";
import "../core/IZKVerifier.sol";
import "solidity-bytes-utils/contracts/BytesLib.sol";
import "@openzeppelin/contracts/utils/math/SafeCast.sol";

library CometblsHelp {
    using BytesLib for bytes;

    uint256 constant PRIME_Q = 21888242871839275222246405745257275088696311157297823662689037894645226208583;
    uint256 constant PRIME_Q_MINUS_ONE = PRIME_Q - 1;

    bytes constant HMAC_I = hex"75595B5342747A653636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636";
    bytes constant HMAC_O = hex"1F333139281E100F5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C";

    bytes1 constant ZERO = bytes1(uint8(0));
    bytes1 constant ONE = bytes1(uint8(1));

    // Specialized https://en.wikipedia.org/wiki/HMAC for keccak256 with `CometBLS` as key.
    function hmac(bytes memory message) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(HMAC_O, keccak256(abi.encodePacked(HMAC_I, message))));
    }

    function hashToField(bytes memory message) internal pure returns (uint256) {
        return (uint256(hmac(message)) % PRIME_Q_MINUS_ONE) + 1;
    }

    function hashToField2(bytes memory message) internal pure returns (uint256, uint256) {
        return (hashToField(abi.encodePacked(ZERO, message)),
                hashToField(abi.encodePacked(ONE, message)));
    }

    function verifyZKP(IZKVerifier verifier, bytes memory trustedValidatorsHash, bytes memory untrustedValidatorsHash, bytes memory message, bytes memory zkp) internal view returns (bool) {
        (uint256 messageX, uint256 messageY) =
            hashToField2(message);

        (uint256[2] memory a, uint256[2][2] memory b, uint256[2] memory c, uint256 commitmentHash, uint256[2] memory proofCommitment) =
            abi.decode(zkp, (uint256[2], uint256[2][2], uint256[2], uint256, uint256[2]));

        uint256[9] memory inputs =
            [
             uint256(trustedValidatorsHash.toUint128(0)),
             uint256(trustedValidatorsHash.toUint128(16)),
             uint256(untrustedValidatorsHash.toUint128(0)),
             uint256(untrustedValidatorsHash.toUint128(16)),
             messageX,
             messageY,
             // Gnark commitment API extend public inputs with the following commitment hash and proof commitment
             // See https://github.com/ConsenSys/gnark/issues/652
             commitmentHash,
             proofCommitment[0],
             proofCommitment[1]
            ];

        return verifier.verifyProof(a, b, c, inputs);
    }

    function toConsensusState(IbcLightclientsTendermintV1Header.Data memory header) internal pure returns (IbcLightclientsTendermintV1ConsensusState.Data memory) {
        return
            IbcLightclientsTendermintV1ConsensusState.Data({
                timestamp: header.signed_header.header.time,
                root: IbcCoreCommitmentV1MerkleRoot.Data({hash: header.signed_header.header.app_hash}),
                next_validators_hash: header.signed_header.header.next_validators_hash
            });
    }

    function isExpired(
        GoogleProtobufTimestamp.Data memory headerTime,
        GoogleProtobufDuration.Data memory trustingPeriod,
        GoogleProtobufDuration.Data memory currentTime
    ) internal pure returns (bool) {
        GoogleProtobufTimestamp.Data memory expirationTime = GoogleProtobufTimestamp.Data({
            secs: headerTime.secs + int64(trustingPeriod.Seconds),
            nanos: headerTime.nanos
        });
        return gt(GoogleProtobufTimestamp.Data({secs: int64(currentTime.Seconds), nanos: 0}), expirationTime);
    }

    function gt(GoogleProtobufTimestamp.Data memory t1, GoogleProtobufTimestamp.Data memory t2) internal pure returns (bool) {
        if (t1.secs > t2.secs) {
            return true;
        } else if (t1.secs == t2.secs && t1.nanos > t2.nanos) {
            return true;
        } else {
            return false;
        }
    }
}
