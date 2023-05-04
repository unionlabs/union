// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.18;

import "../core/02-client/ILightClient.sol";
import "../core/02-client/IBCHeight.sol";
import "../proto/ibc/core/client/v1/client.sol";
import "../proto/ibc/lightclients/tendermint/v1/tendermint.sol";
import "../proto/tendermint/types/types.sol";
import "../proto/tendermint/types/canonical.sol";
import "../proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";
import "../proto/ibc/lightclients/wasm/v1/wasm.sol";
import {GoogleProtobufAny as Any} from "../proto/GoogleProtobufAny.sol";
import "solidity-bytes-utils/contracts/BytesLib.sol";
import "../lib/TrieProofs.sol";
import "../lib/CometblsHelp.sol";
import "../core/IZKVerifier.sol";
import "forge-std/Test.sol";

contract CometblsClient is ILightClient {
    using TrieProofs for bytes;
    using BytesLib for bytes;
    using IBCHeight for IbcCoreClientV1Height.Data;
    using CometblsHelp for TendermintTypesHeader.Data;
    using CometblsHelp for TendermintTypesCommit.Data;
    using CometblsHelp for UnionIbcLightclientsCometblsV1ConsensusState.Data;
    using CometblsHelp for UnionIbcLightclientsCometblsV1ClientState.Data;
    using CometblsHelp for OptimizedConsensusState;
    using CometblsHelp for bytes;

    mapping(string => UnionIbcLightclientsCometblsV1ClientState.Data) internal clientStates;
    mapping(string => mapping(uint128 => OptimizedConsensusState)) internal consensusStates;
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

        (clientState, ok) = clientStateBytes.unmarshalClientState();
        if (!ok) {
            return (clientStateCommitment, update, false);
        }
        (consensusState, ok) = consensusStateBytes.unmarshalConsensusState();
        if (!ok) {
            return (clientStateCommitment, update, false);
        }
        clientStates[clientId] = clientState;
        consensusStates[clientId][clientState.latest_height.toUint128()] = consensusState.toOptimizedConsensusState();
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
        returns (uint64, bool)
    {
        OptimizedConsensusState storage consensusState =
            consensusStates[clientId][height.toUint128()];
        return (consensusState.timestamp, consensusState.timestamp != 0);
    }

    function getLatestHeight(string calldata clientId) external view override returns (IbcCoreClientV1Height.Data memory, bool) {
        UnionIbcLightclientsCometblsV1ClientState.Data storage clientState = clientStates[clientId];
        return (clientState.latest_height, clientState.latest_height.revision_height != 0);
    }

    function updateClient(string calldata clientId, bytes calldata clientMessageBytes)
        external
        override
        onlyIBC
        returns (bytes32, ConsensusStateUpdate[] memory, bool)
    {
        uint256 gas = gasleft();
        (UnionIbcLightclientsCometblsV1Header.Data memory header, bool ok) =
            clientMessageBytes.unmarshalHeader();
        require(ok, "LC: invalid block header");
        console.log("Cometbls: Header.unmarshal(): ", gas - gasleft());

        gas = gasleft();
        UnionIbcLightclientsCometblsV1ClientState.Data storage clientState =
            clientStates[clientId];
        OptimizedConsensusState storage consensusState =
            consensusStates[clientId][header.trusted_height.toUint128()];
        console.log("Cometbls: loadState(): ", gas - gasleft());

        gas = gasleft();
        require(
                consensusState.timestamp != 0,
                "LC: unkonwn trusted height"
        );

        uint64 untrustedHeight = uint64(header.signed_header.commit.height);
        uint64 trustedHeight = header.trusted_height.revision_height;
        require(
                untrustedHeight > trustedHeight,
                "LC: header height <= consensus state height"
        );

        uint64 trustedTimestamp = consensusState.timestamp;
        uint64 untrustedTimestamp = uint64(header.signed_header.header.time.secs);
        require(
                untrustedTimestamp > trustedTimestamp,
                "LC: header time <= consensus state time"
        );

        GoogleProtobufDuration.Data memory currentTime = GoogleProtobufDuration.Data({
            Seconds: int64(uint64(block.timestamp)),
            nanos: 0
            });
        require(
                !CometblsHelp.isExpired(header.signed_header.header.time, clientState.trusting_period, currentTime),
                "LC: header expired"
        );

        uint64 maxClockDrift = uint64(currentTime.Seconds + clientState.max_clock_drift.Seconds);
        require(
            untrustedTimestamp < maxClockDrift,
            "LC: header back to the future"
        );

        console.log("Cometbls: validate()", gas - gasleft());

        /*
         We want to verify that 1/3 of trusted valset & 2/3 of untrusted valset signed.
         In adjacent verification, trusted vals = untrusted vals.
         In non adjacent verification, untrusted vals are coming from the untrusted header.
         */
        bytes32 trustedValidatorsHash = consensusState.nextValidatorsHash;
        bytes32 untrustedValidatorsHash;
        bool adjacent = untrustedHeight == trustedHeight + 1;
        if (adjacent) {
            untrustedValidatorsHash = trustedValidatorsHash;
        } else {
            untrustedValidatorsHash = header.untrusted_validator_set_root.toBytes32(0);
        }

        gas = gasleft();
        bytes32 blockHash = header.signed_header.header.merkleRoot();
        console.log("Cometbls: Header.merkleRoot(): ", gas - gasleft());

        gas = gasleft();
        TendermintTypesCanonicalVote.Data memory vote =
            header.signed_header.commit.toCanonicalVote(clientState.chain_id, blockHash);
        bytes memory signedVote = Encoder.encodeDelim(TendermintTypesCanonicalVote.encode(vote));
        console.log("Cometbls: Commit.toSignedVote()", gas - gasleft());

        gas = gasleft();
        ok = CometblsHelp.verifyZKP(verifier, trustedValidatorsHash, untrustedValidatorsHash, signedVote, header.zero_knowledge_proof);
        require(ok, "LC: invalid ZKP");
        console.log("Cometbls: ZKP.verify()", gas - gasleft());

        IbcCoreClientV1Height.Data memory newHeight = IbcCoreClientV1Height.Data({
            revision_number: header.trusted_height.revision_number,
            revision_height: untrustedHeight
            });

        uint128 newHeightIdx = newHeight.toUint128();

        // Update states
        if (untrustedHeight > clientState.latest_height.revision_height) {
            clientState.latest_height.revision_height = untrustedHeight;
        }

        gas = gasleft();
        consensusState = consensusStates[clientId][newHeightIdx];
        consensusState.timestamp = uint64(header.signed_header.header.time.secs);
        consensusState.root = header.signed_header.header.app_hash.toBytes32(0);
        consensusState.nextValidatorsHash = untrustedValidatorsHash;
        console.log("Cometbls: ConsensusState.update()", gas - gasleft());

        ConsensusStateUpdate[] memory updates = new ConsensusStateUpdate[](1);
        updates[0] =
            ConsensusStateUpdate({
                consensusStateCommitment: keccak256(consensusState.toUnoptimizedConsensusState().marshalConsensusState()),
                height: newHeight
                });

        gas = gasleft();
        processedTimes[clientId][newHeightIdx] = block.timestamp;
        processedHeights[clientId][newHeightIdx] = block.number;
        console.log("Cometbls: updateProcessed()", gas - gasleft());

        gas = gasleft();
        bytes32 newClientState = keccak256(clientState.marshalClientState());
        console.log("Cometbls: ClientState.marshal()", gas - gasleft());

        return (newClientState, updates, true);
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
        OptimizedConsensusState storage consensusState =
            consensusStates[clientId][height.toUint128()];
        assert(consensusState.timestamp != 0);

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
        OptimizedConsensusState storage consensusState =
            consensusStates[clientId][height.toUint128()];
        assert(consensusState.timestamp != 0);

        // TODO
        revert("not implemented");
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
        return (clientState.marshalClientState(), true);
    }

    function getConsensusState(string calldata clientId, IbcCoreClientV1Height.Data calldata height)
        external
        view
        returns (bytes memory consensusStateBytes, bool)
    {
        OptimizedConsensusState storage consensusState = consensusStates[clientId][height.toUint128()];
        if (consensusState.timestamp == 0) {
            return (consensusStateBytes, false);
        }
        return (consensusState.toUnoptimizedConsensusState().marshalConsensusState(), true);
    }

    modifier onlyIBC() {
        require(msg.sender == ibcHandler);
        _;
    }
}
