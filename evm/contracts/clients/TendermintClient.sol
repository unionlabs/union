// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.18;

import "../core/02-client/ILightClient.sol";
import "../core/02-client/IBCHeight.sol";
import "../proto/ibc/core/client/v1/client.sol";
import "../proto/ibc/lightclients/tendermint/v1/tendermint.sol";
import "../proto/ibc/lightclients/wasm/v1/wasm.sol";
import {GoogleProtobufAny as Any} from "../proto/GoogleProtobufAny.sol";
import "solidity-bytes-utils/contracts/BytesLib.sol";
import "../lib/TrieProofs.sol";

struct NormalizedClientState {
    string chain_id;
    IbcLightclientsTendermintV1Fraction.Data trust_level;
    GoogleProtobufDuration.Data trusting_period;
    GoogleProtobufDuration.Data unbonding_period;
    GoogleProtobufDuration.Data max_clock_drift;
    IbcCoreClientV1Height.Data frozen_height;
    IbcCoreClientV1Height.Data latest_height;
    bool allow_update_after_expiry;
    bool allow_update_after_misbehaviour;
}

contract TendermintClient is ILightClient {
    using TrieProofs for bytes;
    using BytesLib for bytes;
    using IBCHeight for IbcCoreClientV1Height.Data;

    string private constant HEADER_TYPE_URL = "/ibc.lightclients.wasm.v1.Header";
    string private constant CLIENT_STATE_TYPE_URL = "/ibc.lightclients.wasm.v1.ClientState";
    string private constant CONSENSUS_STATE_TYPE_URL = "/ibc.lightclients.wasm.v1.ConsensusState";

    bytes32 private constant HEADER_TYPE_URL_HASH = keccak256(abi.encodePacked(HEADER_TYPE_URL));
    bytes32 private constant CLIENT_STATE_TYPE_URL_HASH = keccak256(abi.encodePacked(CLIENT_STATE_TYPE_URL));
    bytes32 private constant CONSENSUS_STATE_TYPE_URL_HASH = keccak256(abi.encodePacked(CONSENSUS_STATE_TYPE_URL));

    mapping(string => NormalizedClientState) internal clientStates;
    mapping(string => mapping(uint128 => IbcLightclientsTendermintV1ConsensusState.Data)) internal consensusStates;
    mapping(string => mapping(uint128 => uint256)) internal processedTimes;
    mapping(string => mapping(uint128 => uint256)) internal processedHeights;
    address internal ibcHandler;

    CosmosIcs23V1ProofSpec.Data private _defaultProofSpec = CosmosIcs23V1ProofSpec.Data({
        leaf_spec: CosmosIcs23V1LeafOp.Data({
            hash: CosmosIcs23V1GlobalEnums.HashOp.SHA256,
            prehash_key: CosmosIcs23V1GlobalEnums.HashOp.NO_HASH,
            prehash_value: CosmosIcs23V1GlobalEnums.HashOp.SHA256,
            length: CosmosIcs23V1GlobalEnums.LengthOp.VAR_PROTO,
            prefix: hex"00"
            }),
        inner_spec: CosmosIcs23V1InnerSpec.Data({
            child_order: getChildOrder(),
            child_size: 32,
            min_prefix_length: 1,
            max_prefix_length: 1,
            empty_child: abi.encodePacked(),
            hash: CosmosIcs23V1GlobalEnums.HashOp.SHA256
            }),
        min_depth: 0,
        max_depth: 0
        });

    function getChildOrder() internal pure returns (int32[] memory) {
        int32[] memory childOrder = new int32[](2);
        childOrder[0] = 0;
        childOrder[1] = 1;
        return childOrder;
    }

    constructor(address ibcHandler_) {
        ibcHandler = ibcHandler_;
    }

    function createClient(string calldata clientId, bytes calldata clientStateBytes, bytes calldata consensusStateBytes)
        external
        override
        onlyIBC
        returns (bytes32 clientStateCommitment, ConsensusStateUpdate memory update, bool ok)
    {
        IbcLightclientsTendermintV1ClientState.Data memory clientState;
        IbcLightclientsTendermintV1ConsensusState.Data memory consensusState;

        (clientState, ok) = unmarshalClientState(clientStateBytes);
        if (!ok) {
            return (clientStateCommitment, update, false);
        }
        (consensusState, ok) = unmarshalConsensusState(consensusStateBytes);
        if (!ok) {
            return (clientStateCommitment, update, false);
        }
        clientStates[clientId] = normalizeClientState(clientState);
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
        IbcLightclientsTendermintV1ConsensusState.Data storage consensusState =
            consensusStates[clientId][height.toUint128()];
        return (consensusState.timestamp, consensusState.timestamp.secs != 0);
    }

    function getLatestHeight(string calldata clientId) external view override returns (IbcCoreClientV1Height.Data memory, bool) {
        NormalizedClientState storage clientState = clientStates[clientId];
        return (clientState.latest_height, clientState.latest_height.revision_height != 0);
    }

    function updateClient(string calldata clientId, bytes calldata clientMessageBytes)
        external
        override
        onlyIBC
        returns (bytes32 clientStateCommitment, ConsensusStateUpdate[] memory updates, bool ok)
    {
        IbcLightclientsTendermintV1Header.Data memory header;
        (header, ok) = unmarshalHeader(clientMessageBytes);
        require(ok, "LC: invalid block header");

        NormalizedClientState storage clientState = clientStates[clientId];
        IbcLightclientsTendermintV1ConsensusState.Data storage consensusState =
            consensusStates[clientId][header.trusted_height.toUint128()];

        // ---> TODO: Verify

        uint64 trustedHeight = header.trusted_height.revision_height;
        GoogleProtobufTimestamp.Data memory trustedTimestamp = consensusState.timestamp;
        bytes memory trustedNextValidatorHash = consensusState.next_validators_hash;

        require(
                uint64(header.signed_header.header.height) > trustedHeight,
                "LC: header height <= consensus state height"
        );

        /* Make updates message */
        updates = new ConsensusStateUpdate[](1);
        /* updates[0] = ConsensusStateUpdate({ */
        /*     consensusStateCommitment: keccak256(marshalConsensusState(consensusState)), */
        /*     height: newHeight */
        /* }); */

        /* processedTimes[clientId][newHeight.toUint128()] = block.timestamp; */
        /* processedHeights[clientId][newHeight.toUint128()] = block.number; */

        return (keccak256(marshalClientState(canonicalizeClientState(clientState))), updates, true);
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
        IbcLightclientsTendermintV1ConsensusState.Data storage consensusState =
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
        IbcLightclientsTendermintV1ConsensusState.Data storage consensusState =
            consensusStates[clientId][height.toUint128()];

        // TODO
        revert("not implemented");
    }

    // Skip proof_specs from proto definition
    // The proof_specs is consequently fixed
    function normalizeClientState(IbcLightclientsTendermintV1ClientState.Data memory clientState) internal pure returns (NormalizedClientState memory) {
        return NormalizedClientState({
                chain_id: clientState.chain_id,
                trust_level: clientState.trust_level,
                trusting_period: clientState.trusting_period,
                unbonding_period: clientState.unbonding_period,
                max_clock_drift: clientState.max_clock_drift,
                frozen_height: clientState.frozen_height,
                latest_height: clientState.latest_height,
                allow_update_after_expiry: clientState.allow_update_after_expiry,
                allow_update_after_misbehaviour: clientState.allow_update_after_misbehaviour
            });
    }

    function canonicalizeClientState(NormalizedClientState memory clientState) internal view returns (IbcLightclientsTendermintV1ClientState.Data memory) {
        CosmosIcs23V1ProofSpec.Data[] memory proofSpecs = new CosmosIcs23V1ProofSpec.Data[](1);
        proofSpecs[0] = _defaultProofSpec;
        return IbcLightclientsTendermintV1ClientState.Data({
            chain_id: clientState.chain_id,
            trust_level: clientState.trust_level,
            trusting_period: clientState.trusting_period,
            unbonding_period: clientState.unbonding_period,
            max_clock_drift: clientState.max_clock_drift,
            frozen_height: clientState.frozen_height,
            latest_height: clientState.latest_height,
            proof_specs: proofSpecs,
            upgrade_path: new string[](0),
            allow_update_after_expiry: clientState.allow_update_after_expiry,
            allow_update_after_misbehaviour: clientState.allow_update_after_misbehaviour
            });
    }

    function marshalClientState(IbcLightclientsTendermintV1ClientState.Data memory clientState) internal pure returns (bytes memory) {
        Any.Data memory anyClientState;
        anyClientState.type_url = CLIENT_STATE_TYPE_URL;
        anyClientState.value = IbcLightclientsTendermintV1ClientState.encode(clientState);
        return Any.encode(anyClientState);
    }

    function marshalConsensusState(IbcLightclientsTendermintV1ConsensusState.Data storage consensusState) internal pure returns (bytes memory) {
        Any.Data memory anyConsensusState;
        anyConsensusState.type_url = CONSENSUS_STATE_TYPE_URL;
        anyConsensusState.value = IbcLightclientsTendermintV1ConsensusState.encode(consensusState);
        return Any.encode(anyConsensusState);
    }

    function unmarshalHeader(bytes memory bz) internal pure returns (IbcLightclientsTendermintV1Header.Data memory header, bool ok) {
        Any.Data memory anyHeader = Any.decode(bz);
        if (keccak256(abi.encodePacked(anyHeader.type_url)) != HEADER_TYPE_URL_HASH) {
            return (header, false);
        }
        return (IbcLightclientsTendermintV1Header.decode(anyHeader.value), true);
    }

    function unmarshalClientState(bytes memory bz)
        internal
        pure
        returns (IbcLightclientsTendermintV1ClientState.Data memory clientState, bool ok)
    {
        Any.Data memory anyClientState = Any.decode(bz);
        if (keccak256(abi.encodePacked(anyClientState.type_url)) != CLIENT_STATE_TYPE_URL_HASH) {
            return (clientState, false);
        }
        return (IbcLightclientsTendermintV1ClientState.decode(anyClientState.value), true);
    }

    function unmarshalConsensusState(bytes memory bz)
        internal
        pure
        returns (IbcLightclientsTendermintV1ConsensusState.Data memory consensusState, bool ok)
    {
        Any.Data memory anyConsensusState = Any.decode(bz);
        if (keccak256(abi.encodePacked(anyConsensusState.type_url)) != CONSENSUS_STATE_TYPE_URL_HASH) {
            return (consensusState, false);
        }
        return (IbcLightclientsTendermintV1ConsensusState.decode(anyConsensusState.value), true);
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
                          IbcLightclientsTendermintV1ClientState.Data storage cs,
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
        NormalizedClientState storage clientState = clientStates[clientId];
        if (clientState.latest_height.revision_height == 0) {
            return (clientStateBytes, false);
        }
        return (Any.encode(Any.Data({type_url: CLIENT_STATE_TYPE_URL, value: IbcLightclientsTendermintV1ClientState.encode(canonicalizeClientState(clientState))})), true);
    }

    function getConsensusState(string calldata clientId, IbcCoreClientV1Height.Data calldata height)
        external
        view
        returns (bytes memory consensusStateBytes, bool)
    {
        IbcLightclientsTendermintV1ConsensusState.Data storage consensusState = consensusStates[clientId][height.toUint128()];
        if (consensusState.timestamp.secs == 0) {
            return (consensusStateBytes, false);
        }
        return (
                Any.encode(Any.Data({type_url: CONSENSUS_STATE_TYPE_URL, value: IbcLightclientsTendermintV1ConsensusState.encode(consensusState)})),
            true
        );
    }

    modifier onlyIBC() {
        require(msg.sender == ibcHandler);
        _;
    }
}
