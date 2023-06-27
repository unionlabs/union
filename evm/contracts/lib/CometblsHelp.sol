pragma solidity ^0.8.18;

import "../proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";
import "../proto/ibc/lightclients/wasm/v1/wasm.sol";
import "../proto/ibc/core/commitment/v1/commitment.sol";
import "../proto/ibc/lightclients/tendermint/v1/tendermint.sol";
import "../proto/tendermint/types/types.sol";
import "../proto/tendermint/types/validator.sol";
import "../proto/tendermint/types/canonical.sol";
import "./Encoder.sol";
import "./MerkleTree.sol";
import "../core/IZKVerifier.sol";
import "solidity-bytes-utils/BytesLib.sol";
import "@openzeppelin/contracts/utils/math/SafeCast.sol";
import {GoogleProtobufAny as Any} from "../proto/GoogleProtobufAny.sol";

struct OptimizedConsensusState {
    bytes32 root;
    bytes32 nextValidatorsHash;
    uint64 timestamp;
}

struct ProcessedMoment {
    uint128 timestamp;
    uint128 height;
}

library CometblsHelp {
    using BytesLib for bytes;

    string constant WASM_CLIENT_STATE_URL =
        "/ibc.lightclients.wasm.v1.ClientState";
    string constant WASM_CONSENSUS_STATE_URL =
        "/ibc.lightclients.wasm.v1.ConsensusState";

    uint256 constant PRIME_R =
        21888242871839275222246405745257275088548364400416034343698204186575808495617;
    uint256 constant PRIME_R_MINUS_ONE = PRIME_R - 1;

    bytes constant HMAC_I =
        hex"75595B5342747A653636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636";
    bytes constant HMAC_O =
        hex"1F333139281E100F5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C";

    bytes1 constant ZERO = bytes1(uint8(0));
    bytes1 constant ONE = bytes1(uint8(1));

    // Specialized https://en.wikipedia.org/wiki/HMAC for keccak256 with `CometBLS` as key.
    // TODO: link whitepaper
    function hmac_keccak(bytes memory message) internal pure returns (bytes32) {
        return
            keccak256(
                abi.encodePacked(
                    HMAC_O,
                    keccak256(abi.encodePacked(HMAC_I, message))
                )
            );
    }

    // TODO: link whitepaper
    function hashToField(bytes memory message) internal pure returns (uint256) {
        return (uint256(hmac_keccak(message)) % PRIME_R_MINUS_ONE) + 1;
    }

    // TODO: link whitepaper
    function hashToField2(
        bytes memory message
    ) internal pure returns (uint256, uint256) {
        return (
            hashToField(abi.encodePacked(ZERO, message)),
            hashToField(abi.encodePacked(ONE, message))
        );
    }

    function verifyZKP(
        IZKVerifier verifier,
        bytes32 trustedValidatorsHash,
        bytes32 untrustedValidatorsHash,
        bytes memory message,
        bytes memory zkp
    ) internal view returns (bool) {
        (uint256 messageX, uint256 messageY) = hashToField2(message);

        (
            uint256[2] memory a,
            uint256[2][2] memory b,
            uint256[2] memory c,
            uint256 commitmentHash,
            uint256[2] memory proofCommitment
        ) = abi.decode(
                zkp,
                (uint256[2], uint256[2][2], uint256[2], uint256, uint256[2])
            );

        bytes memory packedTrustedValidatorsHash = abi.encodePacked(
            trustedValidatorsHash
        );
        bytes memory packedUntrustedValidatorsHash = abi.encodePacked(
            untrustedValidatorsHash
        );

        uint256[9] memory inputs = [
            uint256(packedTrustedValidatorsHash.toUint128(0)),
            uint256(packedTrustedValidatorsHash.toUint128(16)),
            uint256(packedUntrustedValidatorsHash.toUint128(0)),
            uint256(packedUntrustedValidatorsHash.toUint128(16)),
            messageX,
            messageY,
            // Gnark commitment API extend internal inputs with the following commitment hash and proof commitment
            // See https://github.com/ConsenSys/gnark/issues/652
            commitmentHash,
            proofCommitment[0],
            proofCommitment[1]
        ];

        return verifier.verifyProof(a, b, c, inputs);
    }

    function isExpired(
        GoogleProtobufTimestamp.Data memory headerTime,
        GoogleProtobufDuration.Data memory trustingPeriod,
        GoogleProtobufDuration.Data memory currentTime
    ) internal pure returns (bool) {
        GoogleProtobufTimestamp.Data
            memory expirationTime = GoogleProtobufTimestamp.Data({
                secs: headerTime.secs + int64(trustingPeriod.Seconds),
                nanos: headerTime.nanos
            });
        return
            gt(
                GoogleProtobufTimestamp.Data({
                    secs: int64(currentTime.Seconds),
                    nanos: 0
                }),
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

    function merkleRoot(
        TendermintTypesHeader.Data memory h
    ) internal view returns (bytes32) {
        require(
            h.validators_hash.length > 0,
            "Tendermint: hash can't be empty"
        );
        bytes memory hbz = TendermintVersionConsensus.encode(h.version);
        bytes memory pbt = GoogleProtobufTimestamp.encode(h.time);
        bytes memory bzbi = TendermintTypesBlockID.encode(h.last_block_id);
        bytes32[14] memory normalizedHeader = [
            MerkleTree.leafHash(hbz),
            MerkleTree.leafHash(Encoder.cdcEncode(h.chain_id)),
            MerkleTree.leafHash(Encoder.cdcEncode(h.height)),
            MerkleTree.leafHash(pbt),
            MerkleTree.leafHash(bzbi),
            MerkleTree.leafHash(Encoder.cdcEncode(h.last_commit_hash)),
            MerkleTree.leafHash(Encoder.cdcEncode(h.data_hash)),
            MerkleTree.leafHash(Encoder.cdcEncode(h.validators_hash)),
            MerkleTree.leafHash(Encoder.cdcEncode(h.next_validators_hash)),
            MerkleTree.leafHash(Encoder.cdcEncode(h.consensus_hash)),
            MerkleTree.leafHash(Encoder.cdcEncode(h.app_hash)),
            MerkleTree.leafHash(Encoder.cdcEncode(h.last_results_hash)),
            MerkleTree.leafHash(Encoder.cdcEncode(h.evidence_hash)),
            MerkleTree.leafHash(Encoder.cdcEncode(h.proposer_address))
        ];
        return MerkleTree.optimizedBlockRoot(normalizedHeader);
    }

    function toOptimizedConsensusState(
        UnionIbcLightclientsCometblsV1ConsensusState.Data memory consensusState,
        uint64 timestamp
    ) internal pure returns (OptimizedConsensusState memory) {
        return
            OptimizedConsensusState({
                timestamp: timestamp,
                root: consensusState.root.hash.toBytes32(0),
                nextValidatorsHash: consensusState
                    .next_validators_hash
                    .toBytes32(0)
            });
    }

    function toCanonicalVote(
        TendermintTypesCommit.Data memory commit,
        string memory chainId,
        bytes32 expectedBlockHash
    ) internal pure returns (TendermintTypesCanonicalVote.Data memory) {
        return
            TendermintTypesCanonicalVote.Data({
                type_: TendermintTypesTypesGlobalEnums
                    .SignedMsgType
                    .SIGNED_MSG_TYPE_PRECOMMIT,
                height: commit.height,
                round: commit.round,
                block_id: TendermintTypesCanonicalBlockID.Data({
                    hash: abi.encodePacked(expectedBlockHash),
                    part_set_header: TendermintTypesCanonicalPartSetHeader
                        .Data({
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

    function unmarshalHeaderEthABI(
        bytes memory bz
    )
        internal
        pure
        returns (UnionIbcLightclientsCometblsV1Header.Data memory header, bool)
    {
        return (
            abi.decode(bz, (UnionIbcLightclientsCometblsV1Header.Data)),
            true
        );
    }

    function unmarshalClientStateFromProto(
        bytes memory bz
    )
        internal
        pure
        returns (
            UnionIbcLightclientsCometblsV1ClientState.Data memory,
            IbcCoreClientV1Height.Data memory,
            bytes memory
        )
    {
        Any.Data memory any = Any.decode(bz);
        require(
            keccak256(bytes(any.type_url)) ==
                keccak256(bytes(WASM_CLIENT_STATE_URL)),
            "invalid client state url"
        );
        IbcLightclientsWasmV1ClientState.Data
            memory wasmClientState = IbcLightclientsWasmV1ClientState.decode(
                any.value
            );
        return (
            UnionIbcLightclientsCometblsV1ClientState.decode(
                wasmClientState.data
            ),
            wasmClientState.latest_height,
            wasmClientState.code_id
        );
    }

    function unmarshalConsensusStateFromProto(
        bytes memory bz
    )
        internal
        pure
        returns (
            UnionIbcLightclientsCometblsV1ConsensusState.Data memory,
            uint64
        )
    {
        Any.Data memory any = Any.decode(bz);
        require(
            keccak256(bytes(any.type_url)) ==
                keccak256(bytes(WASM_CONSENSUS_STATE_URL)),
            "invalid consensus state url"
        );
        IbcLightclientsWasmV1ConsensusState.Data
            memory wasmConsensusState = IbcLightclientsWasmV1ConsensusState
                .decode(any.value);
        return (
            UnionIbcLightclientsCometblsV1ConsensusState.decode(
                wasmConsensusState.data
            ),
            wasmConsensusState.timestamp
        );
    }

    function marshalToProto(
        OptimizedConsensusState memory consensusState
    ) internal pure returns (bytes memory) {
        IbcLightclientsWasmV1ConsensusState.Data
            memory wasmConsensusState = IbcLightclientsWasmV1ConsensusState
                .Data({
                    timestamp: consensusState.timestamp,
                    data: UnionIbcLightclientsCometblsV1ConsensusState.encode(
                        UnionIbcLightclientsCometblsV1ConsensusState.Data({
                            root: IbcCoreCommitmentV1MerkleRoot.Data({
                                hash: abi.encodePacked(consensusState.root)
                            }),
                            next_validators_hash: abi.encodePacked(
                                consensusState.nextValidatorsHash
                            )
                        })
                    )
                });
        return
            Any.encode(
                Any.Data({
                    type_url: WASM_CONSENSUS_STATE_URL,
                    value: IbcLightclientsWasmV1ConsensusState.encode(
                        wasmConsensusState
                    )
                })
            );
    }

    function marshalToProto(
        UnionIbcLightclientsCometblsV1ClientState.Data memory clientState,
        IbcCoreClientV1Height.Data memory latestHeight,
        bytes memory codeId
    ) internal pure returns (bytes memory) {
        IbcLightclientsWasmV1ClientState.Data
            memory wasmClientState = IbcLightclientsWasmV1ClientState.Data({
                data: UnionIbcLightclientsCometblsV1ClientState.encode(
                    clientState
                ),
                code_id: codeId,
                latest_height: latestHeight
            });
        return
            Any.encode(
                Any.Data({
                    type_url: WASM_CLIENT_STATE_URL,
                    value: IbcLightclientsWasmV1ClientState.encode(
                        wasmClientState
                    )
                })
            );
    }

    function marshalToCommitment(
        OptimizedConsensusState memory consensusState
    ) internal pure returns (bytes32) {
        return keccak256(marshalToProto(consensusState));
    }

    function marshalToCommitment(
        UnionIbcLightclientsCometblsV1ClientState.Data memory clientState,
        IbcCoreClientV1Height.Data memory latestHeight,
        bytes memory codeId
    ) internal pure returns (bytes32) {
        return keccak256(marshalToProto(clientState, latestHeight, codeId));
    }
}
