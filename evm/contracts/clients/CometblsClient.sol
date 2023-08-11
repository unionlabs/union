pragma solidity ^0.8.18;

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

    /// helper struct to avoid "stack too deep" errors in `createClient`
    struct CreateClientLocals {
        // client state fields
        UnionIbcLightclientsCometblsV1ClientState.Data clientState;
        IbcCoreClientV1Height.Data latestHeight;
        bytes codeId;
        // consensus state fields
        UnionIbcLightclientsCometblsV1ConsensusState.Data consensusState;
        uint64 timestamp;
        // optimized consensus state fields
        OptimizedConsensusState optimizedConsensusState;
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
        CreateClientLocals memory locals;

        (
            locals.clientState,
            locals.latestHeight,
            locals.codeId
        ) = clientStateBytes.unmarshalClientStateFromProto();
        (locals.consensusState, locals.timestamp) = consensusStateBytes
            .unmarshalConsensusStateFromProto();
        clientStates[clientId] = locals.clientState;
        latestHeights[clientId] = locals.latestHeight;
        codeIds[clientId] = locals.codeId;

        locals.optimizedConsensusState = locals
            .consensusState
            .toOptimizedConsensusState(locals.timestamp);
        consensusStates[
            stateIndex(clientId, locals.latestHeight.toUint128())
        ] = locals.optimizedConsensusState;

        return (
            locals.clientState.marshalToCommitment(
                locals.latestHeight,
                locals.codeId
            ),
            ConsensusStateUpdate({
                consensusStateCommitment: locals
                    .optimizedConsensusState
                    .marshalToCommitment(),
                height: locals.latestHeight
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

    struct UpdateClientStruct {
        UnionIbcLightclientsCometblsV1Header.Data header;
        uint64 untrustedHeightNumber;
        uint64 trustedHeightNumber;
        uint64 trustedTimestamp;
        uint64 untrustedTimestamp;
        GoogleProtobufDuration.Data currentTime;
        uint64 maxClockDrift;
        bytes32 trustedValidatorsHash;
        bytes32 untrustedValidatorsHash;
        bool adjacent;
        bytes32 expectedBlockHash;
        bytes signedVote;
        IbcCoreClientV1Height.Data untrustedHeight;
        IbcCoreClientV1Height.Data latestHeight;
        uint128 newHeightIdx;
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
        UpdateClientStruct memory locals;
        bool ok;

        (locals.header, ok) = clientMessageBytes.unmarshalHeaderEthABI();
        require(ok, "LC: invalid block header");

        UnionIbcLightclientsCometblsV1ClientState.Data
            storage clientState = clientStates[clientId];
        OptimizedConsensusState storage consensusState = consensusStates[
            stateIndex(clientId, locals.header.trusted_height.toUint128())
        ];

        require(
            consensusState.timestamp != 0,
            "LC: trusted height does not exists"
        );

        locals.untrustedHeightNumber = uint64(
            locals.header.signed_header.commit.height
        );
        locals.trustedHeightNumber = locals
            .header
            .trusted_height
            .revision_height;
        require(
            locals.untrustedHeightNumber > locals.trustedHeightNumber,
            "LC: header height <= consensus state height"
        );

        locals.trustedTimestamp = consensusState.timestamp;
        locals.untrustedTimestamp = uint64(
            locals.header.signed_header.header.time.secs
        );
        require(
            locals.untrustedTimestamp > locals.trustedTimestamp,
            "LC: header time <= consensus state time"
        );

        locals.currentTime = GoogleProtobufDuration.Data({
            Seconds: int64(uint64(block.timestamp)),
            nanos: 0
        });
        require(
            !CometblsHelp.isExpired(
                locals.header.signed_header.header.time,
                clientState.trusting_period,
                locals.currentTime
            ),
            "LC: header expired"
        );

        locals.maxClockDrift = uint64(
            locals.currentTime.Seconds + clientState.max_clock_drift.Seconds
        );
        require(
            locals.untrustedTimestamp < locals.maxClockDrift,
            "LC: header back to the future"
        );

        /*
         We want to verify that 1/3 of trusted valset & 2/3 of untrusted valset signed.
         In adjacent verification, trusted vals = untrusted vals.
         In non adjacent verification, untrusted vals are coming from the untrusted header.
         */
        locals.trustedValidatorsHash = consensusState.nextValidatorsHash;
        locals.untrustedValidatorsHash;
        locals.adjacent =
            locals.untrustedHeightNumber == locals.trustedHeightNumber + 1;
        if (locals.adjacent) {
            locals.untrustedValidatorsHash = locals.trustedValidatorsHash;
        } else {
            locals.untrustedValidatorsHash = locals
                .header
                .untrusted_validator_set_root
                .toBytes32(0);
        }

        locals.expectedBlockHash = locals
            .header
            .signed_header
            .header
            .merkleRoot();

        require(
            locals.header.signed_header.commit.block_id.hash.toBytes32(0) ==
                locals.expectedBlockHash,
            "LC: commit.block_id.hash != header.root()"
        );

        // create a scope to narrow down the `vote` local
        {
            TendermintTypesCanonicalVote.Data memory vote = locals
                .header
                .signed_header
                .commit
                .toCanonicalVote(
                    clientState.chain_id,
                    locals.expectedBlockHash
                );
            locals.signedVote = Encoder.encodeDelim(
                TendermintTypesCanonicalVote.encode(vote)
            );
        }

        ok = zkVerifier.verifyZKP(
            locals.trustedValidatorsHash,
            locals.untrustedValidatorsHash,
            locals.signedVote,
            locals.header.zero_knowledge_proof
        );
        require(ok, "LC: invalid ZKP");

        locals.untrustedHeight = IbcCoreClientV1Height.Data({
            revision_number: locals.header.trusted_height.revision_number,
            revision_height: locals.untrustedHeightNumber
        });

        // Update states
        locals.latestHeight = latestHeights[clientId];
        if (
            locals.untrustedHeightNumber > locals.latestHeight.revision_height
        ) {
            locals.latestHeight.revision_height = locals.untrustedHeightNumber;
        }

        locals.newHeightIdx = locals.untrustedHeight.toUint128();

        consensusState = consensusStates[
            stateIndex(clientId, locals.newHeightIdx)
        ];
        consensusState.timestamp = uint64(
            locals.header.signed_header.header.time.secs
        );
        consensusState.root = locals
            .header
            .signed_header
            .header
            .app_hash
            .toBytes32(0);
        consensusState.nextValidatorsHash = locals.untrustedValidatorsHash;

        ConsensusStateUpdate[] memory updates = new ConsensusStateUpdate[](1);
        updates[0] = ConsensusStateUpdate({
            consensusStateCommitment: consensusState.marshalToCommitment(),
            height: locals.untrustedHeight
        });

        processedMoments[
            stateIndex(clientId, locals.newHeightIdx)
        ] = ProcessedMoment({
            timestamp: uint128(block.timestamp),
            height: uint128(block.number)
        });

        return (
            clientState.marshalToCommitment(
                locals.latestHeight,
                codeIds[clientId]
            ),
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
        bytes32 idx = stateIndex(clientId, height.toUint128());
        OptimizedConsensusState memory consensusState = consensusStates[idx];
        require(
            consensusState.timestamp != 0,
            "LC: verifyMembership: consensusState does not exist"
        );
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
        return true;
    }

    function getClientState(
        string calldata clientId
    ) external view returns (bytes memory, bool) {
        return (
            clientStates[clientId].marshalToProto(
                latestHeights[clientId],
                codeIds[clientId]
            ),
            true
        );
    }

    function getConsensusState(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height
    ) external view returns (bytes memory, bool) {
        return (
            consensusStates[stateIndex(clientId, height.toUint128())]
                .marshalToProto(),
            true
        );
    }

    modifier onlyIBC() {
        require(msg.sender == ibcHandler);
        _;
    }
}
