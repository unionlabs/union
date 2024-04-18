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

    function isExpired(
        uint64 headerTime,
        uint64 trustingPeriod,
        uint64 currentTime
    ) internal pure returns (bool) {
        return currentTime > (headerTime + trustingPeriod);
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

    function marshalEthABIMemory(
        UnionIbcLightclientsCometblsV1Header.Data memory header
    ) internal pure returns (bytes memory) {
        return abi.encode(
            header.signed_header,
            header.trusted_height,
            header.zero_knowledge_proof
        );
    }

    function marshalEthABI(
        UnionIbcLightclientsCometblsV1Header.Data calldata header
    ) internal pure returns (bytes memory) {
        return abi.encode(
            header.signed_header,
            header.trusted_height,
            header.zero_knowledge_proof
        );
    }

    function unmarshalEthABI(bytes calldata bz)
        internal
        pure
        returns (UnionIbcLightclientsCometblsV1Header.Data calldata)
    {
        UnionIbcLightclientsCometblsV1Header.Data calldata header;
        assembly {
            header := bz.offset
        }
        return header;
    }

    function marshalEthABI(
        UnionIbcLightclientsCometblsV1ClientState.Data memory clientState
    ) internal pure returns (bytes memory) {
        return abi.encode(
            clientState.chain_id,
            clientState.trusting_period,
            clientState.unbonding_period,
            clientState.max_clock_drift,
            clientState.frozen_height,
            clientState.latest_height
        );
    }

    function unmarshalClientStateEthABI(bytes calldata bz)
        internal
        pure
        returns (UnionIbcLightclientsCometblsV1ClientState.Data calldata)
    {
        UnionIbcLightclientsCometblsV1ClientState.Data calldata clientState;
        assembly {
            clientState := bz.offset
        }
        return clientState;
    }

    function marshalEthABI(OptimizedConsensusState memory consensusState)
        internal
        pure
        returns (bytes memory)
    {
        return abi.encode(
            consensusState.timestamp,
            consensusState.appHash,
            consensusState.nextValidatorsHash
        );
    }

    function unmarshalConsensusStateEthABI(bytes calldata bz)
        internal
        pure
        returns (OptimizedConsensusState calldata)
    {
        OptimizedConsensusState calldata consensusState;
        assembly {
            consensusState := bz.offset
        }
        return consensusState;
    }

    function unmarshalConsensusStateEthABIMemory(bytes memory bz)
        internal
        pure
        returns (OptimizedConsensusState memory)
    {
        OptimizedConsensusState memory consensusState;
        (uint64 timestamp, bytes32 appHash, bytes32 nextValidatorsHash) =
            abi.decode(bz, (uint64, bytes32, bytes32));
        consensusState.timestamp = timestamp;
        consensusState.appHash = appHash;
        consensusState.nextValidatorsHash = nextValidatorsHash;
        return consensusState;
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
