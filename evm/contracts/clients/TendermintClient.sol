// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.18;

import "../core/02-client/ILightClient.sol";
import "../core/02-client/IBCHeight.sol";
import "../proto/ibc/core/client/v1/client.sol";
import "../proto/ibc/lightclients/tendermint/v1/tendermint.sol";
import "../proto/tendermint/types/types.sol";
import "../proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";
import "../proto/ibc/lightclients/wasm/v1/wasm.sol";
import {GoogleProtobufAny as Any} from "../proto/GoogleProtobufAny.sol";
import "solidity-bytes-utils/contracts/BytesLib.sol";
import "../lib/TrieProofs.sol";
import "../lib/TendermintHelp.sol";
import "../core/IZKVerifier.sol";


contract TendermintClient is ILightClient {
    using TrieProofs for bytes;
    using BytesLib for bytes;
    using IBCHeight for IbcCoreClientV1Height.Data;

    uint256 constant PRIME_Q = 21888242871839275222246405745257275088696311157297823662689037894645226208583;
    string private constant HEADER_TYPE_URL = "/ibc.lightclients.wasm.v1.Header";
    string private constant CLIENT_STATE_TYPE_URL = "/ibc.lightclients.wasm.v1.ClientState";
    string private constant CONSENSUS_STATE_TYPE_URL = "/ibc.lightclients.wasm.v1.ConsensusState";

    bytes32 private constant HEADER_TYPE_URL_HASH = keccak256(abi.encodePacked(HEADER_TYPE_URL));
    bytes32 private constant CLIENT_STATE_TYPE_URL_HASH = keccak256(abi.encodePacked(CLIENT_STATE_TYPE_URL));
    bytes32 private constant CONSENSUS_STATE_TYPE_URL_HASH = keccak256(abi.encodePacked(CONSENSUS_STATE_TYPE_URL));

    mapping(string => UnionIbcLightclientsCometblsV1ClientState.Data) internal clientStates;
    mapping(string => mapping(uint128 => UnionIbcLightclientsCometblsV1ConsensusState.Data)) internal consensusStates;
    mapping(string => mapping(uint128 => uint256)) internal processedTimes;
    mapping(string => mapping(uint128 => uint256)) internal processedHeights;
    address internal ibcHandler;
    IZKVerifier internal verifier;

    constructor(address ibcHandler_, IZKVerifier verifier_) {
        ibcHandler = ibcHandler_;
        verifier = verifier_;
    }

    function createClient(string calldata clientId, bytes calldata clientStateBytes, bytes calldata consensusStateBytes)
        external
        override
        onlyIBC
        returns (bytes32 clientStateCommitment, ConsensusStateUpdate memory update, bool ok)
    {
        UnionIbcLightclientsCometblsV1ClientState.Data memory clientState;
        UnionIbcLightclientsCometblsV1ConsensusState.Data memory consensusState;

        (clientState, ok) = unmarshalClientState(clientStateBytes);
        if (!ok) {
            return (clientStateCommitment, update, false);
        }
        (consensusState, ok) = unmarshalConsensusState(consensusStateBytes);
        if (!ok) {
            return (clientStateCommitment, update, false);
        }
        clientStates[clientId] = clientState;
        consensusStates[clientId][clientState.latest_height.toUint128()] = consensusState;
        return (
            keccak256(clientStateBytes),
            ConsensusStateUpdate({
                consensusStateCommitment: keccak256(consensusStateBytes),
                height: clientState.latest_height
            }),
            true
        );
    }

    function getTimestampAtHeight(string calldata clientId, IbcCoreClientV1Height.Data calldata height)
        external
        view
        override
        returns (GoogleProtobufTimestamp.Data memory, bool)
    {
        UnionIbcLightclientsCometblsV1ConsensusState.Data storage consensusState =
            consensusStates[clientId][height.toUint128()];
        return (consensusState.timestamp, consensusState.timestamp.secs != 0);
    }

    function getLatestHeight(string calldata clientId) external view override returns (IbcCoreClientV1Height.Data memory, bool) {
        UnionIbcLightclientsCometblsV1ClientState.Data storage clientState = clientStates[clientId];
        return (clientState.latest_height, clientState.latest_height.revision_height != 0);
    }

    function hmac(bytes memory key, bytes memory message) internal pure returns (bytes32) {
        bytes32 keyl;
        bytes32 keyr;
        uint i;
        if (key.length > 64) {
            keyl = keccak256(key);
        } else {
            for (i = 0; i < key.length && i < 32; i++)
                keyl |= bytes32(uint256(uint8(key[i])) * 2 ** (8 * (31 - i)));
            for (i = 32; i < key.length && i < 64; i++)
                keyr |= bytes32(uint256(uint8(key[i])) * 2 ** (8 * (63 - i)));
        }
        bytes32 threesix = 0x3636363636363636363636363636363636363636363636363636363636363636;
        bytes32 fivec = 0x5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c5c;
        return keccak256(abi.encodePacked(fivec ^ keyl, fivec ^ keyr, keccak256(abi.encodePacked(threesix ^ keyl, threesix ^ keyr, message))));
    }

    function hashToField(bytes memory message) internal pure returns (uint256) {
        return (uint256(hmac("CometBLS", message)) % (PRIME_Q - 1)) + 1;
    }

    function hashToField2(bytes memory message) internal pure returns (uint256, uint256) {
        return (hashToField(bytes(hex"00").concat(message)),
                hashToField(bytes(hex"01").concat(message)));
    }

    function updateClient(string calldata clientId, bytes calldata clientMessageBytes)
        external
        override
        onlyIBC
        returns (bytes32, ConsensusStateUpdate[] memory, bool)
    {
        (UnionIbcLightclientsCometblsV1Header.Data memory header, bool ok) =
            unmarshalHeader(clientMessageBytes);
        require(ok, "LC: invalid block header");

        UnionIbcLightclientsCometblsV1ClientState.Data storage clientState =
            clientStates[clientId];
        UnionIbcLightclientsCometblsV1ConsensusState.Data storage consensusState =
            consensusStates[clientId][header.trusted_height.toUint128()];

        uint64 untrustedHeight = uint64(header.signed_header.header.height);
        uint64 trustedHeight = header.trusted_height.revision_height;
        require(
                untrustedHeight > trustedHeight,
                "LC: header height <= consensus state height"
        );

        GoogleProtobufTimestamp.Data memory trustedTimestamp = consensusState.timestamp;
        GoogleProtobufTimestamp.Data memory untrustedTimestamp = header.signed_header.header.time;
        require(
                (untrustedTimestamp.secs > trustedTimestamp.secs) ||
                (untrustedTimestamp.secs == trustedTimestamp.secs && untrustedTimestamp.nanos > trustedTimestamp.nanos),
                "LC: header time <= consensus state time"
        );

        GoogleProtobufDuration.Data memory currentTime = GoogleProtobufDuration.Data({
            Seconds: int64(uint64(block.timestamp)),
            nanos: 0
            });
        require(
                !TendermintHelper.isExpired(header.signed_header.header.time, clientState.trusting_period, currentTime),
                "LC: header expired"
        );

        GoogleProtobufDuration.Data memory maxClockDrift = GoogleProtobufDuration.Data({
            Seconds: currentTime.Seconds + clientState.max_clock_drift.Seconds,
            nanos: currentTime.nanos + clientState.max_clock_drift.nanos
            });
        require(
            (untrustedTimestamp.secs < maxClockDrift.Seconds) ||
            (untrustedTimestamp.secs == maxClockDrift.Seconds && untrustedTimestamp.nanos < maxClockDrift.nanos),
            "LC: header back to the future"
        );

        /*
         We want to verify that 1/3 of trusted valset & 2/3 of untrusted valset signed.
         In adjacent verification, trusted vals = untrusted vals.
         In non adjacent verification, untrusted vals are coming from the untrusted header.
         */
        bytes memory trustedValidatorsHash = consensusState.next_validators_hash;
        bytes memory untrustedValidatorsHash;
        bool adjacent = untrustedHeight == trustedHeight + 1;
        if (adjacent) {
            untrustedValidatorsHash = trustedValidatorsHash;
        } else {
            untrustedValidatorsHash = header.untrusted_validator_set_root;
        }

        (uint256 messageX, uint256 messageY) =
            hashToField2(TendermintTypesSignedHeader.encode(header.signed_header));

        (uint256[2] memory a, uint256[2][2] memory b, uint256[2] memory c, uint256 commitmentHash, uint256[2] memory proofCommitment) =
            abi.decode(header.zero_knowledge_proof, (uint256[2], uint256[2][2], uint256[2], uint256, uint256[2]));

        uint256[9] memory inputs =
            [
             trustedValidatorsHash.slice(0, 16).toUint256(0),
             trustedValidatorsHash.slice(16, 32).toUint256(0),
             untrustedValidatorsHash.slice(0, 16).toUint256(0),
             untrustedValidatorsHash.slice(16, 32).toUint256(0),
             messageX,
             messageY,
             // Gnark commitment API extend public inputs with the following commitment hash and proof commitment
             // See https://github.com/ConsenSys/gnark/issues/652
             commitmentHash,
             proofCommitment[0],
             proofCommitment[1]
            ];

        ok = verifier.verifyProof(a, b, c, inputs);
        require(ok, "LC: invalid ZKP");

        IbcCoreClientV1Height.Data memory newHeight = IbcCoreClientV1Height.Data({
            revision_number: header.trusted_height.revision_number,
            revision_height: untrustedHeight
            });

        uint128 newHeightIdx = newHeight.toUint128();

        // Update states
        if (untrustedHeight > clientState.latest_height.revision_height) {
            clientState.latest_height.revision_height = untrustedHeight;
        }
        consensusState = consensusStates[clientId][newHeightIdx];
        consensusState.timestamp = header.signed_header.header.time;
        consensusState.root = IbcCoreCommitmentV1MerkleRoot.Data({
            hash: header.signed_header.header.app_hash
            });
        consensusState.next_validators_hash = untrustedValidatorsHash;

        ConsensusStateUpdate[] memory updates = new ConsensusStateUpdate[](1);
        updates[0] =
            ConsensusStateUpdate({
                consensusStateCommitment: keccak256(marshalConsensusState(consensusState)),
                height: newHeight
                });

        processedTimes[clientId][newHeightIdx] = block.timestamp;
        processedHeights[clientId][newHeightIdx] = block.number;

        return (keccak256(marshalClientState(clientState)), updates, true);
    }

    function verifyMembership(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height,
        uint64 delayTimePeriod,
        uint64 delayBlockPeriod,
        bytes calldata proof,
        bytes memory prefix,
        bytes calldata path,
        bytes calldata value
    ) external view override returns (bool) {
        if (!validateArgsAndDelayPeriod(clientId, height, delayTimePeriod, delayBlockPeriod, prefix, proof)) {
            return false;
        }
        UnionIbcLightclientsCometblsV1ConsensusState.Data storage consensusState =
            consensusStates[clientId][height.toUint128()];
        assert(consensusState.timestamp.secs != 0);

        // TODO
        revert("not implemented");
    }

    function verifyNonMembership(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height,
        uint64 delayTimePeriod,
        uint64 delayBlockPeriod,
        bytes calldata proof,
        bytes calldata prefix,
        bytes calldata path
    ) external returns (bool) {
        if (!validateArgsAndDelayPeriod(clientId, height, delayTimePeriod, delayBlockPeriod, prefix, proof)) {
            return false;
        }
        UnionIbcLightclientsCometblsV1ConsensusState.Data storage consensusState =
            consensusStates[clientId][height.toUint128()];

        // TODO
        revert("not implemented");
    }

    function marshalClientState(UnionIbcLightclientsCometblsV1ClientState.Data memory clientState) internal pure returns (bytes memory) {
        IbcLightclientsWasmV1ClientState.Data memory wasmClientState =
            IbcLightclientsWasmV1ClientState.Data({
                data: UnionIbcLightclientsCometblsV1ClientState.encode(clientState),
                // Not used
                code_id: bytes(""),
                // Not used
                latest_height: clientState.latest_height
                });
        return Any.encode(Any.Data({type_url: CLIENT_STATE_TYPE_URL, value: IbcLightclientsWasmV1ClientState.encode(wasmClientState)}));
    }

    function marshalConsensusState(UnionIbcLightclientsCometblsV1ConsensusState.Data storage consensusState) internal pure returns (bytes memory) {
        IbcLightclientsWasmV1ConsensusState.Data memory wasmConsensusState =
            IbcLightclientsWasmV1ConsensusState.Data({
                data: UnionIbcLightclientsCometblsV1ConsensusState.encode(consensusState),
                // Not used
                timestamp: 0
                });
        return Any.encode(Any.Data({type_url: CONSENSUS_STATE_TYPE_URL, value: IbcLightclientsWasmV1ConsensusState.encode(wasmConsensusState)}));
    }

    function unmarshalHeader(bytes memory bz) internal pure returns (UnionIbcLightclientsCometblsV1Header.Data memory header, bool) {
        (IbcLightclientsWasmV1Header.Data memory wasmHeader, bool ok) = unmarshalWasmHeader(bz);
        if (ok) {
            return (UnionIbcLightclientsCometblsV1Header.decode(wasmHeader.data), true);
        } else {
            return (header, false);
        }
    }

    function unmarshalClientState(bytes memory bz)
        internal
        pure
        returns (UnionIbcLightclientsCometblsV1ClientState.Data memory clientState, bool)
    {
        (IbcLightclientsWasmV1ClientState.Data memory wasmClientState, bool ok) = unmarshalWasmClientState(bz);
        if (ok) {
            return (UnionIbcLightclientsCometblsV1ClientState.decode(wasmClientState.data), true);
        }
        else {
            return (clientState, false);
        }
    }

    function unmarshalConsensusState(bytes memory bz)
        internal
        pure
        returns (UnionIbcLightclientsCometblsV1ConsensusState.Data memory consensusState, bool)
    {
        (IbcLightclientsWasmV1ConsensusState.Data memory wasmConsensusState, bool ok) = unmarshalWasmConsensusState(bz);
        if (ok) {
            return (UnionIbcLightclientsCometblsV1ConsensusState.decode(wasmConsensusState.data), true);
        } else {
            return (consensusState, false);
        }
    }

    function unmarshalWasmHeader(bytes memory bz) internal pure returns (IbcLightclientsWasmV1Header.Data memory header, bool ok) {
        Any.Data memory anyHeader = Any.decode(bz);
        if (keccak256(abi.encodePacked(anyHeader.type_url)) != HEADER_TYPE_URL_HASH) {
            return (header, false);
        }
        return (IbcLightclientsWasmV1Header.decode(anyHeader.value), true);
    }


    function unmarshalWasmClientState(bytes memory bz)
        internal
        pure
        returns (IbcLightclientsWasmV1ClientState.Data memory clientState, bool ok)
    {
        Any.Data memory anyClientState = Any.decode(bz);
        if (keccak256(abi.encodePacked(anyClientState.type_url)) != CLIENT_STATE_TYPE_URL_HASH) {
            return (clientState, false);
        }
        return (IbcLightclientsWasmV1ClientState.decode(anyClientState.value), true);
    }

    function unmarshalWasmConsensusState(bytes memory bz)
        internal
        pure
        returns (IbcLightclientsWasmV1ConsensusState.Data memory consensusState, bool ok)
    {
        Any.Data memory anyConsensusState = Any.decode(bz);
        if (keccak256(abi.encodePacked(anyConsensusState.type_url)) != CONSENSUS_STATE_TYPE_URL_HASH) {
            return (consensusState, false);
        }
        return (IbcLightclientsWasmV1ConsensusState.decode(anyConsensusState.value), true);
    }

    function validateArgs(
                          UnionIbcLightclientsCometblsV1ClientState.Data storage cs,
        IbcCoreClientV1Height.Data memory height,
        bytes memory prefix,
        bytes memory proof
    ) internal view returns (bool) {
        if (cs.latest_height.lt(height)) {
            return false;
        } else if (prefix.length == 0) {
            return false;
        } else if (proof.length == 0) {
            return false;
        }
        return true;
    }

    function validateDelayPeriod(
        string memory clientId,
        IbcCoreClientV1Height.Data memory height,
        uint64 delayPeriodTime,
        uint64 delayPeriodBlocks
    ) private view returns (bool) {
        uint128 heightU128 = height.toUint128();
        uint64 currentTime = uint64(block.timestamp * 1000 * 1000 * 1000);
        uint64 validTime = uint64(processedTimes[clientId][heightU128]) * 1000 * 1000 * 1000 + delayPeriodTime;
        if (currentTime < validTime) {
            return false;
        }
        uint64 currentHeight = uint64(block.number);
        uint64 validHeight = uint64(processedHeights[clientId][heightU128]) + delayPeriodBlocks;
        if (currentHeight < validHeight) {
            return false;
        }
        return true;
    }

    function validateArgsAndDelayPeriod(
        string memory clientId,
        IbcCoreClientV1Height.Data memory height,
        uint64 delayTimePeriod,
        uint64 delayBlockPeriod,
        bytes memory prefix,
        bytes memory proof
    ) internal view returns (bool) {
        revert("not implemented");
    }

    function getClientState(string calldata clientId) external view returns (bytes memory clientStateBytes, bool) {
        UnionIbcLightclientsCometblsV1ClientState.Data storage clientState = clientStates[clientId];
        if (clientState.latest_height.revision_height == 0) {
            return (clientStateBytes, false);
        }
        return (Any.encode(Any.Data({type_url: CLIENT_STATE_TYPE_URL, value: UnionIbcLightclientsCometblsV1ClientState.encode(clientState)})), true);
    }

    function getConsensusState(string calldata clientId, IbcCoreClientV1Height.Data calldata height)
        external
        view
        returns (bytes memory consensusStateBytes, bool)
    {
        UnionIbcLightclientsCometblsV1ConsensusState.Data storage consensusState = consensusStates[clientId][height.toUint128()];
        if (consensusState.timestamp.secs == 0) {
            return (consensusStateBytes, false);
        }
        return (
                Any.encode(Any.Data({type_url: CONSENSUS_STATE_TYPE_URL, value: UnionIbcLightclientsCometblsV1ConsensusState.encode(consensusState)})),
            true
        );
    }

    modifier onlyIBC() {
        require(msg.sender == ibcHandler);
        _;
    }
}
