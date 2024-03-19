pragma solidity ^0.8.23;

import "../proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";
import "../proto/ibc/lightclients/wasm/v1/wasm.sol";
import "../proto/ibc/core/commitment/v1/commitment.sol";
import "../proto/ibc/lightclients/tendermint/v1/tendermint.sol";
import "../proto/tendermint/types/types.sol";
import "../proto/tendermint/types/validator.sol";
import "../proto/tendermint/types/canonical.sol";
import "../proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";
import "./Encoder.sol";
import "./MerkleTree.sol";
import "../core/IZKVerifierV2.sol";
import "solidity-bytes-utils/BytesLib.sol";
import "@openzeppelin/utils/math/SafeCast.sol";
import {GoogleProtobufAny as Any} from "../proto/GoogleProtobufAny.sol";

struct OptimizedConsensusState {
    uint64 timestamp;
    bytes32 appHash;
    bytes32 nextValidatorsHash;
}

struct ProcessedMoment {
    uint256 timestamp;
    uint256 height;
}

library CometblsHelp {
    using BytesLib for bytes;

    uint256 constant PRIME_R =
        21888242871839275222246405745257275088548364400416034343698204186575808495617;
    uint256 constant PRIME_R_MINUS_ONE = PRIME_R - 1;

    bytes constant HMAC_I =
        hex"75595B5342747A653636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636";
    bytes constant HMAC_O =
        hex"1F333139281E100F5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C";

    function hmac_keccak(bytes memory message)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(
            abi.encodePacked(HMAC_O, keccak256(HMAC_I.concat(message)))
        );
    }

    // Union whitepaper: (1) H_{hmac_r}
    function hashToField(bytes memory message)
        internal
        pure
        returns (uint256)
    {
        return (uint256(hmac_keccak(message)) % PRIME_R_MINUS_ONE) + 1;
    }

    function verifyZKP(
        IZKVerifierV2 verifier,
        bytes memory zkp,
        string memory chainId,
        bytes32 trustedValidatorsHash,
        UnionIbcLightclientsCometblsV1LightHeader.Data memory header
    ) internal returns (bool) {
        (
            uint256[8] memory proof,
            uint256[2] memory proofCommitment,
            uint256[2] memory proofCommitmentPOK
        ) = abi.decode(zkp, (uint256[8], uint256[2], uint256[2]));

        uint256 commitmentHash = hashToField(abi.encodePacked(proofCommitment));

        uint256 l = bytes(chainId).length;
        bytes memory paddedChainId = new bytes(32 - l).concat(bytes(chainId));

        // Drop the most significant byte to fit in F_r
        bytes32 inputsHash = sha256(
            abi.encodePacked(
                bytes32(paddedChainId),
                bytes32(uint256(int256(header.height))),
                bytes32(uint256(int256(header.time.secs))),
                bytes32(uint256(int256(header.time.nanos))),
                bytes32(header.validators_hash),
                bytes32(header.next_validators_hash),
                bytes32(header.app_hash),
                trustedValidatorsHash
            )
        ) & 0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;

        uint256[2] memory publicInputs = [
            uint256(inputsHash),
            // Gnark commitment API extend internal inputs with the following commitment hash and proof commitment
            // See https://github.com/ConsenSys/gnark/issues/652
            commitmentHash
        ];

        return verifier.verifyProof(
            proof, proofCommitment, proofCommitmentPOK, publicInputs
        );
    }

    function isExpired(
        GoogleProtobufTimestamp.Data memory headerTime,
        uint64 trustingPeriod,
        uint64 currentTime
    ) internal pure returns (bool) {
        GoogleProtobufTimestamp.Data memory expirationTime =
        GoogleProtobufTimestamp.Data({
            secs: headerTime.secs + int64(trustingPeriod),
            nanos: headerTime.nanos
        });
        return gt(
            GoogleProtobufTimestamp.Data({secs: int64(currentTime), nanos: 0}),
            expirationTime
        );
    }

    function gt(
        GoogleProtobufTimestamp.Data memory t1,
        GoogleProtobufTimestamp.Data memory t2
    ) internal pure returns (bool) {
        if (t1.secs > t2.secs) {
            return true;
        } else if (t1.secs == t2.secs && t1.nanos > t2.nanos) {
            return true;
        } else {
            return false;
        }
    }

    function merkleRoot(TendermintTypesHeader.Data memory h)
        internal
        pure
        returns (bytes32)
    {
        return MerkleTree.optimizedBlockRoot(
            [
                MerkleTree.leafHash(TendermintVersionConsensus.encode(h.version)),
                MerkleTree.leafHash(Encoder.cdcEncode(h.chain_id)),
                MerkleTree.leafHash(Encoder.cdcEncode(h.height)),
                MerkleTree.leafHash(GoogleProtobufTimestamp.encode(h.time)),
                MerkleTree.leafHash(TendermintTypesBlockID.encode(h.last_block_id)),
                MerkleTree.leafHash(Encoder.cdcEncode(h.last_commit_hash)),
                MerkleTree.leafHash(Encoder.cdcEncode(h.data_hash)),
                MerkleTree.leafHash(Encoder.cdcEncode(h.validators_hash)),
                MerkleTree.leafHash(Encoder.cdcEncode(h.next_validators_hash)),
                MerkleTree.leafHash(Encoder.cdcEncode(h.consensus_hash)),
                MerkleTree.leafHash(Encoder.cdcEncode(h.app_hash)),
                MerkleTree.leafHash(Encoder.cdcEncode(h.last_results_hash)),
                MerkleTree.leafHash(Encoder.cdcEncode(h.evidence_hash)),
                MerkleTree.leafHash(Encoder.cdcEncode(h.proposer_address))
            ]
        );
    }

    function optimize(
        UnionIbcLightclientsCometblsV1ConsensusState.Data memory consensusState
    ) internal pure returns (OptimizedConsensusState memory) {
        return OptimizedConsensusState({
            timestamp: consensusState.timestamp,
            appHash: consensusState.root.hash.toBytes32(0),
            nextValidatorsHash: consensusState.next_validators_hash.toBytes32(0)
        });
    }

    function canonicalize(
        TendermintTypesCommit.Data memory commit,
        string memory chainId,
        bytes32 expectedBlockHash
    ) internal pure returns (TendermintTypesCanonicalVote.Data memory) {
        return TendermintTypesCanonicalVote.Data({
            type_: TendermintTypesTypesGlobalEnums
                .SignedMsgType
                .SIGNED_MSG_TYPE_PRECOMMIT,
            height: commit.height,
            round: commit.round,
            block_id: TendermintTypesCanonicalBlockID.Data({
                hash: abi.encodePacked(expectedBlockHash),
                part_set_header: TendermintTypesCanonicalPartSetHeader.Data({
                    total: commit.block_id.part_set_header.total,
                    hash: commit.block_id.part_set_header.hash
                })
            }),
            chain_id: chainId
        });
    }

    function marshalHeaderEthABI(
        UnionIbcLightclientsCometblsV1Header.Data memory header
    ) internal pure returns (bytes memory) {
        return abi.encode(header);
    }

    function marshalEthABI(
        UnionIbcLightclientsCometblsV1Header.Data memory header
    ) internal pure returns (bytes memory) {
        return abi.encode(header);
    }

    function unmarshalHeaderEthABI(bytes memory bz)
        internal
        pure
        returns (UnionIbcLightclientsCometblsV1Header.Data memory header)
    {
        return abi.decode(bz, (UnionIbcLightclientsCometblsV1Header.Data));
    }

    function marshalEthABI(
        UnionIbcLightclientsCometblsV1ClientState.Data memory clientState
    ) internal pure returns (bytes memory) {
        return abi.encode(clientState);
    }

    function unmarshalClientStateEthABI(bytes memory bz)
        internal
        pure
        returns (UnionIbcLightclientsCometblsV1ClientState.Data memory)
    {
        return abi.decode(bz, (UnionIbcLightclientsCometblsV1ClientState.Data));
    }

    function marshalEthABI(OptimizedConsensusState memory consensusState)
        internal
        pure
        returns (bytes memory)
    {
        return abi.encode(consensusState);
    }

    function unmarshalConsensusStateEthABI(bytes memory bz)
        internal
        pure
        returns (OptimizedConsensusState memory consensusState)
    {
        return abi.decode(bz, (OptimizedConsensusState));
    }

    function marshalToCommitmentEthABI(
        OptimizedConsensusState memory consensusState
    ) internal pure returns (bytes32) {
        return keccak256(marshalEthABI(consensusState));
    }

    function marshalToCommitmentEthABI(
        UnionIbcLightclientsCometblsV1ClientState.Data memory clientState
    ) internal pure returns (bytes32) {
        return keccak256(marshalEthABI(clientState));
    }
}
