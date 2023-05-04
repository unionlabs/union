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
import "0xsequence-sstore2/contracts/SSTORE2.sol";
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

    mapping(string => address) internal clientStates;
    mapping(string => mapping(uint128 => address)) internal consensusStates;
    mapping(string => mapping(uint128 => ProcessedMoment)) internal processedMoments;
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
        clientStates[clientId] = SSTORE2.write(abi.encode(clientState));
        consensusStates[clientId][clientState.latest_height.toUint128()] = SSTORE2.write(abi.encode(consensusState.toOptimizedConsensusState()));
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
        OptimizedConsensusState memory consensusState =
            abi.decode(SSTORE2.read(consensusStates[clientId][height.toUint128()]), (OptimizedConsensusState));
        return (consensusState.timestamp, consensusState.timestamp != 0);
    }

    function getLatestHeight(string calldata clientId) external view override returns (IbcCoreClientV1Height.Data memory, bool) {
        UnionIbcLightclientsCometblsV1ClientState.Data memory clientState =
            abi.decode(SSTORE2.read(clientStates[clientId]), (UnionIbcLightclientsCometblsV1ClientState.Data));
        return (clientState.latest_height, clientState.latest_height.revision_height != 0);
    }

    function updateClient(string calldata clientId, bytes calldata clientMessageBytes)
        external
        override
        onlyIBC
        returns (bytes32, ConsensusStateUpdate[] memory, bool)
    {
        uint256 gas = gasleft();
        UnionIbcLightclientsCometblsV1Header.Data memory header =
            abi.decode(clientMessageBytes, (UnionIbcLightclientsCometblsV1Header.Data));
        console.log("Cometbls: Header.unmarshal(): ", gas - gasleft());

        gas = gasleft();
        UnionIbcLightclientsCometblsV1ClientState.Data memory clientState =
            abi.decode(SSTORE2.read(clientStates[clientId]), (UnionIbcLightclientsCometblsV1ClientState.Data));
        OptimizedConsensusState memory consensusState =
            abi.decode(SSTORE2.read(consensusStates[clientId][header.trusted_height.toUint128()]), (OptimizedConsensusState));
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

        GoogleProtobufDuration.Data memory currentTime =
            GoogleProtobufDuration.Data({
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
        bool ok = CometblsHelp.verifyZKP(verifier, trustedValidatorsHash, untrustedValidatorsHash, signedVote, header.zero_knowledge_proof);
        require(ok, "LC: invalid ZKP");
        console.log("Cometbls: ZKP.verify()", gas - gasleft());

        IbcCoreClientV1Height.Data memory newHeight =
            IbcCoreClientV1Height.Data({
                revision_number: header.trusted_height.revision_number,
                revision_height: untrustedHeight
            });

        uint128 newHeightIdx = newHeight.toUint128();

        // Update states
        if (untrustedHeight > clientState.latest_height.revision_height) {
            // TODO!!
            /* clientState.latest_height.revision_height = untrustedHeight; */
        }

        gas = gasleft();
        consensusState.timestamp = uint64(header.signed_header.header.time.secs);
        consensusState.root = header.signed_header.header.app_hash.toBytes32(0);
        consensusState.nextValidatorsHash = untrustedValidatorsHash;
        consensusStates[clientId][newHeightIdx] = SSTORE2.write(abi.encode(consensusState));
        console.log("Cometbls: ConsensusState.update()", gas - gasleft());

        ConsensusStateUpdate[] memory updates = new ConsensusStateUpdate[](1);
        updates[0] =
            ConsensusStateUpdate({
                consensusStateCommitment: keccak256(consensusState.toUnoptimizedConsensusState().marshalConsensusState()),
                height: newHeight
            });

        gas = gasleft();
        processedMoments[clientId][newHeightIdx] =
            ProcessedMoment({
                timestamp: block.timestamp,
                height: block.number
            });
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
        // TODO
        revert("not implemented");
    }

    function getClientState(string calldata clientId) external view returns (bytes memory clientStateBytes, bool) {
        UnionIbcLightclientsCometblsV1ClientState.Data memory clientState =
            abi.decode(SSTORE2.read(clientStates[clientId]), (UnionIbcLightclientsCometblsV1ClientState.Data));
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
        OptimizedConsensusState memory consensusState =
            abi.decode(SSTORE2.read(consensusStates[clientId][height.toUint128()]), (OptimizedConsensusState));
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
