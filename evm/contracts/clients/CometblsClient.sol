pragma solidity ^0.8.21;

import "../core/02-client/ILightClient.sol";
import "../core/02-client/IBCHeight.sol";
import "../proto/ibc/core/client/v1/client.sol";
import "../proto/ibc/lightclients/tendermint/v1/tendermint.sol";
import "../proto/cosmos/ics23/v1/proofs.sol";
import "../proto/tendermint/types/types.sol";
import "../proto/tendermint/types/canonical.sol";
import "../proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";
import "../proto/ibc/lightclients/wasm/v1/wasm.sol";
import {GoogleProtobufAny as Any} from "../proto/GoogleProtobufAny.sol";
import "solidity-bytes-utils/BytesLib.sol";
import "../lib/CometblsHelp.sol";
import "../lib/ICS23.sol";
import "../core/IZKVerifier.sol";
import "../core/IMembershipVerifier.sol";

contract CometblsClient is ILightClient {
    using BytesLib for bytes;
    using IBCHeight for IbcCoreClientV1Height.Data;
    using CometblsHelp for TendermintTypesHeader.Data;
    using CometblsHelp for TendermintTypesCommit.Data;
    using CometblsHelp for UnionIbcLightclientsCometblsV1ConsensusState.Data;
    using CometblsHelp for UnionIbcLightclientsCometblsV1ClientState.Data;
    using CometblsHelp for OptimizedConsensusState;
    using CometblsHelp for bytes;
    using CometblsHelp for IZKVerifier;

    // OptimizedConsensusState
    mapping(string => IbcCoreClientV1Height.Data) internal latestHeights;
    mapping(string => bytes) internal codeIds;
    mapping(string => UnionIbcLightclientsCometblsV1ClientState.Data)
        internal clientStates;
    mapping(bytes32 => OptimizedConsensusState) internal consensusStates;
    mapping(bytes32 => ProcessedMoment) internal processedMoments;

    address internal ibcHandler;
    IZKVerifier internal zkVerifier;
    IMembershipVerifier internal membershipVerifier;

    constructor(
        address ibcHandler_,
        IZKVerifier zkVerifier_,
        IMembershipVerifier membershipVerifier_
    ) {
        ibcHandler = ibcHandler_;
        zkVerifier = zkVerifier_;
        membershipVerifier = membershipVerifier_;
    }

    function stateIndex(
        string calldata clientId,
        uint128 height
    ) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(clientId, height));
    }

    function createClient(
        string calldata clientId,
        bytes calldata clientStateBytes,
        bytes calldata consensusStateBytes
    )
        external
        override
        onlyIBC
        returns (
            bytes32 clientStateCommitment,
            ConsensusStateUpdate memory update,
            bool ok
        )
    {
        (
            UnionIbcLightclientsCometblsV1ClientState.Data memory clientState,
            IbcCoreClientV1Height.Data memory latestHeight,
            bytes memory codeId
        ) = clientStateBytes.unmarshalClientStateFromProto();
        (
            UnionIbcLightclientsCometblsV1ConsensusState.Data
                memory consensusState,
            uint64 timestamp
        ) = consensusStateBytes.unmarshalConsensusStateFromProto();

        if (latestHeight.revision_height == 0 || timestamp == 0) {
            return (clientStateCommitment, update, false);
        }

        clientStates[clientId] = clientState;
        latestHeights[clientId] = latestHeight;
        codeIds[clientId] = codeId;
        OptimizedConsensusState memory optimizedConsensusState = consensusState
            .optimize();
        consensusStates[
            stateIndex(clientId, latestHeight.toUint128())
        ] = optimizedConsensusState;
        return (
            clientState.marshalToCommitment(latestHeight, codeId),
            ConsensusStateUpdate({
                consensusStateCommitment: optimizedConsensusState
                    .marshalToCommitment(),
                height: latestHeight
            }),
            true
        );
    }

    function getTimestampAtHeight(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height
    ) external view override returns (uint64, bool) {
        OptimizedConsensusState memory consensusState = consensusStates[
            stateIndex(clientId, height.toUint128())
        ];
        return (consensusState.timestamp, true);
    }

    function getLatestHeight(
        string calldata clientId
    ) external view override returns (IbcCoreClientV1Height.Data memory, bool) {
        return (latestHeights[clientId], true);
    }

    function updateClient(
        string calldata clientId,
        bytes calldata clientMessageBytes
    )
        external
        override
        onlyIBC
        returns (bytes32, ConsensusStateUpdate[] memory, bool)
    {
        UnionIbcLightclientsCometblsV1Header.Data
            memory header = clientMessageBytes.unmarshalHeaderEthABI();
        UnionIbcLightclientsCometblsV1ClientState.Data
            storage clientState = clientStates[clientId];
        OptimizedConsensusState storage consensusState = consensusStates[
            stateIndex(clientId, header.trusted_height.toUint128())
        ];

        require(
            consensusState.timestamp != 0,
            "LC: trusted height does not exists"
        );

        uint64 untrustedHeightNumber = uint64(
            header.signed_header.commit.height
        );
        uint64 trustedHeightNumber = header.trusted_height.revision_height;
        require(
            untrustedHeightNumber > trustedHeightNumber,
            "LC: header height <= consensus state height"
        );

        uint64 trustedTimestamp = consensusState.timestamp;
        uint64 untrustedTimestamp = uint64(
            header.signed_header.header.time.secs
        );
        require(
            untrustedTimestamp > trustedTimestamp,
            "LC: header time <= consensus state time"
        );

        require(
            !CometblsHelp.isExpired(
                header.signed_header.header.time,
                clientState.trusting_period,
                uint64(block.timestamp)
            ),
            "LC: header expired"
        );

        uint64 maxClockDrift = uint64(block.timestamp) +
            clientState.max_clock_drift;
        require(
            untrustedTimestamp < maxClockDrift,
            "LC: header back to the future"
        );

        /*
         We want to verify that 1/3 of trusted valset & 2/3 of untrusted valset signed.
         In adjacent verification, trusted vals = untrusted vals.
         In non adjacent verification, untrusted vals are coming from the untrusted header.
         */
        bytes32 trustedValidatorsHash = consensusState.nextValidatorsHash;
        bytes32 untrustedValidatorsHash;
        bool adjacent = untrustedHeightNumber == trustedHeightNumber + 1;
        if (adjacent) {
            untrustedValidatorsHash = trustedValidatorsHash;
        } else {
            untrustedValidatorsHash = header
                .signed_header
                .header
                .validators_hash
                .toBytes32(0);
        }

        bytes32 expectedBlockHash = header.signed_header.header.merkleRoot();

        require(
            header.signed_header.commit.block_id.hash.toBytes32(0) ==
                expectedBlockHash,
            "LC: commit.block_id.hash != header.root()"
        );

        TendermintTypesCanonicalVote.Data memory vote = header
            .signed_header
            .commit
            .canonicalize(clientState.chain_id, expectedBlockHash);
        bytes memory signedVote = Encoder.encodeDelim(
            TendermintTypesCanonicalVote.encode(vote)
        );

        bool ok = zkVerifier.verifyZKP(
            trustedValidatorsHash,
            untrustedValidatorsHash,
            signedVote,
            header.zero_knowledge_proof
        );
        require(ok, "LC: invalid ZKP");

        IbcCoreClientV1Height.Data
            memory untrustedHeight = IbcCoreClientV1Height.Data({
                revision_number: header.trusted_height.revision_number,
                revision_height: untrustedHeightNumber
            });

        // Update states
        IbcCoreClientV1Height.Data storage latestHeight = latestHeights[
            clientId
        ];
        if (untrustedHeightNumber > latestHeight.revision_height) {
            latestHeight.revision_height = untrustedHeightNumber;
        }

        uint128 newHeightIdx = untrustedHeight.toUint128();

        consensusState = consensusStates[stateIndex(clientId, newHeightIdx)];
        consensusState.timestamp = uint64(
            header.signed_header.header.time.secs
        );
        consensusState.root = header.signed_header.header.app_hash.toBytes32(0);
        consensusState.nextValidatorsHash = untrustedValidatorsHash;

        ConsensusStateUpdate[] memory updates = new ConsensusStateUpdate[](1);
        updates[0] = ConsensusStateUpdate({
            consensusStateCommitment: consensusState.marshalToCommitment(),
            height: untrustedHeight
        });

        processedMoments[stateIndex(clientId, newHeightIdx)] = ProcessedMoment({
            timestamp: uint128(block.timestamp),
            height: uint128(block.number)
        });

        return (
            clientState.marshalToCommitment(latestHeight, codeIds[clientId]),
            updates,
            true
        );
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
        OptimizedConsensusState memory consensusState = consensusStates[
            stateIndex(clientId, height.toUint128())
        ];
        require(
            consensusState.timestamp != 0,
            "LC: verifyMembership: consensusState does not exist"
        );
        if (
            !validateDelayPeriod(
                clientId,
                height,
                delayTimePeriod,
                delayBlockPeriod
            )
        ) {
            revert("LC: delayPeriod expired");
        }
        return
            membershipVerifier.verifyMembership(
                abi.encodePacked(consensusState.root),
                proof,
                prefix,
                path,
                value
            );
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
        OptimizedConsensusState memory consensusState = consensusStates[
            stateIndex(clientId, height.toUint128())
        ];
        require(
            consensusState.timestamp != 0,
            "LC: verifyNonMembership: consensusState does not exist"
        );
        if (
            !validateDelayPeriod(
                clientId,
                height,
                delayTimePeriod,
                delayBlockPeriod
            )
        ) {
            revert("LC: delayPeriod expired");
        }
        return
            membershipVerifier.verifyNonMembership(
                abi.encodePacked(consensusState.root),
                proof,
                prefix,
                path
            );
    }

    function validateDelayPeriod(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height,
        uint64 delayPeriodTime,
        uint64 delayPeriodBlocks
    ) public view returns (bool) {
        uint128 heightU128 = height.toUint128();
        uint64 currentTime = uint64(block.timestamp);
        ProcessedMoment memory moment = processedMoments[
            stateIndex(clientId, heightU128)
        ];
        uint64 validTime = uint64(moment.timestamp) + delayPeriodTime;
        if (delayPeriodTime != 0 && currentTime < validTime) {
            return false;
        }
        uint64 currentHeight = uint64(block.number);
        uint64 validHeight = uint64(moment.height) + delayPeriodBlocks;
        if (delayPeriodBlocks != 0 && currentHeight < validHeight) {
            return false;
        }
        return true;
    }

    function getClientState(
        string calldata clientId
    ) external view returns (bytes memory, bool) {
        bytes memory codeId = codeIds[clientId];
        if (codeId.length == 0) {
            return (bytes(""), false);
        }
        return (
            clientStates[clientId].marshalToProto(
                latestHeights[clientId],
                codeId
            ),
            true
        );
    }

    function getConsensusState(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height
    ) external view returns (bytes memory, bool) {
        OptimizedConsensusState memory consensusState = consensusStates[
            stateIndex(clientId, height.toUint128())
        ];
        if (consensusState.timestamp == 0) {
            return (bytes(""), false);
        }
        return (consensusState.marshalToProto(), true);
    }

    modifier onlyIBC() {
        require(msg.sender == ibcHandler);
        _;
    }
}
