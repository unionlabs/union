pragma solidity ^0.8.23;

import "../core/02-client/ILightClient.sol";
import "../core/02-client/IBCHeight.sol";
import "../proto/ibc/core/client/v1/client.sol";
import "../proto/ibc/lightclients/tendermint/v1/tendermint.sol";
import "../proto/cosmos/ics23/v1/proofs.sol";
import "../proto/tendermint/types/types.sol";
import "../proto/tendermint/types/canonical.sol";
import "../proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";
import "../proto/ibc/lightclients/wasm/v1/wasm.sol";
import "../lib/CometblsHelp.sol";
import "../lib/ICS23.sol";
import "../core/IZKVerifierV2.sol";
import "../core/IMembershipVerifier.sol";

import "solidity-bytes-utils/BytesLib.sol";

library CometblsClientLib {
    error ErrUnauthorized();
    error ErrTrustedConsensusStateNotFound();
    error ErrUntrustedHeightLTETrustedHeight();
    error ErrUntrustedTimestampLTETrustedTimestamp();
    error ErrHeaderExpired();
    error ErrMaxClockDriftExceeded();
    error ErrPrecomputedRootAndBlockRootMismatch();
    error ErrInvalidZKP();
    error ErrDelayPeriodNotExpired();
}

contract CometblsClient is ILightClient {
    using BytesLib for bytes;
    using IBCHeight for IbcCoreClientV1Height.Data;
    using CometblsHelp for TendermintTypesHeader.Data;
    using CometblsHelp for TendermintTypesCommit.Data;
    using CometblsHelp for UnionIbcLightclientsCometblsV1ClientState.Data;
    using CometblsHelp for OptimizedConsensusState;
    using CometblsHelp for bytes;
    using CometblsHelp for IZKVerifierV2;

    mapping(string => UnionIbcLightclientsCometblsV1ClientState.Data)
        private clientStates;
    mapping(string => mapping(uint128 => OptimizedConsensusState))
        private consensusStates;
    mapping(string => mapping(uint128 => ProcessedMoment))
        private processedMoments;

    address private ibcHandler;
    IZKVerifierV2 private zkVerifier;
    IMembershipVerifier private membershipVerifier;

    constructor(
        address ibcHandler_,
        IZKVerifierV2 zkVerifier_,
        IMembershipVerifier membershipVerifier_
    ) {
        ibcHandler = ibcHandler_;
        zkVerifier = zkVerifier_;
        membershipVerifier = membershipVerifier_;
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
        UnionIbcLightclientsCometblsV1ClientState.Data
            memory clientState = clientStateBytes.unmarshalClientStateEthABI();
        OptimizedConsensusState memory consensusState = consensusStateBytes
            .unmarshalConsensusStateEthABI();
        if (
            clientState.latest_height.revision_height == 0 ||
            consensusState.timestamp == 0
        ) {
            return (clientStateCommitment, update, false);
        }
        clientStates[clientId] = clientState;
        uint128 latestHeight = clientState.latest_height.toUint128();
        consensusStates[clientId][latestHeight] = consensusState;
        processedMoments[clientId][latestHeight] = ProcessedMoment({
            timestamp: block.timestamp,
            height: block.number
        });
        return (
            clientState.marshalToCommitmentEthABI(),
            ConsensusStateUpdate({
                consensusStateCommitment: consensusState
                    .marshalToCommitmentEthABI(),
                height: clientState.latest_height
            }),
            true
        );
    }

    function verifyHeader(
        UnionIbcLightclientsCometblsV1Header.Data memory header,
        OptimizedConsensusState storage consensusState,
        UnionIbcLightclientsCometblsV1ClientState.Data storage clientState
    ) internal returns (uint64, uint64, bytes32) {
        if (consensusState.timestamp == 0) {
            revert CometblsClientLib.ErrTrustedConsensusStateNotFound();
        }

        uint64 untrustedHeightNumber = uint64(
            header.signed_header.commit.height
        );
        uint64 trustedHeightNumber = header.trusted_height.revision_height;
        if (untrustedHeightNumber <= trustedHeightNumber) {
            revert CometblsClientLib.ErrUntrustedHeightLTETrustedHeight();
        }

        uint64 trustedTimestamp = consensusState.timestamp;
        uint64 untrustedTimestamp = uint64(
            header.signed_header.header.time.secs
        );
        if (untrustedTimestamp <= trustedTimestamp) {
            revert CometblsClientLib.ErrUntrustedTimestampLTETrustedTimestamp();
        }

        if (
            CometblsHelp.isExpired(
                header.signed_header.header.time,
                clientState.trusting_period,
                uint64(block.timestamp)
            )
        ) {
            revert CometblsClientLib.ErrHeaderExpired();
        }

        uint64 maxClockDrift = uint64(block.timestamp) +
            clientState.max_clock_drift;
        if (untrustedTimestamp >= maxClockDrift) {
            revert CometblsClientLib.ErrMaxClockDriftExceeded();
        }

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

        if (
            header.signed_header.commit.block_id.hash.toBytes32(0) !=
            expectedBlockHash
        ) {
            revert CometblsClientLib.ErrPrecomputedRootAndBlockRootMismatch();
        }

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
        if (!ok) {
            revert CometblsClientLib.ErrInvalidZKP();
        }

        return (
            untrustedHeightNumber,
            untrustedTimestamp,
            untrustedValidatorsHash
        );
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
            clientId
        ][header.trusted_height.toUint128()];

        (
            uint64 untrustedHeightNumber,
            uint64 untrustedTimestamp,
            bytes32 untrustedValidatorsHash
        ) = verifyHeader(header, consensusState, clientState);

        // Update states
        if (untrustedHeightNumber > clientState.latest_height.revision_height) {
            clientState.latest_height.revision_height = untrustedHeightNumber;
        }

        IbcCoreClientV1Height.Data
            memory untrustedHeight = IbcCoreClientV1Height.Data({
                revision_number: header.trusted_height.revision_number,
                revision_height: untrustedHeightNumber
            });

        uint128 untrustedHeightIndex = untrustedHeight.toUint128();

        consensusState = consensusStates[clientId][untrustedHeightIndex];
        consensusState.timestamp = untrustedTimestamp;
        consensusState.appHash = header.signed_header.header.app_hash.toBytes32(
            0
        );
        consensusState.nextValidatorsHash = header
            .signed_header
            .header
            .next_validators_hash
            .toBytes32(0);

        processedMoments[clientId][untrustedHeightIndex].timestamp = block
            .timestamp;
        processedMoments[clientId][untrustedHeightIndex].height = block.number;

        ConsensusStateUpdate[] memory updates = new ConsensusStateUpdate[](1);
        updates[0] = ConsensusStateUpdate({
            consensusStateCommitment: consensusState
                .marshalToCommitmentEthABI(),
            height: untrustedHeight
        });

        return (clientState.marshalToCommitmentEthABI(), updates, true);
    }

    function verifyMembership(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height,
        uint64 delayPeriodTime,
        uint64 delayPeriodBlocks,
        bytes calldata proof,
        bytes memory prefix,
        bytes calldata path,
        bytes calldata value
    ) external returns (bool) {
        bytes memory appHash = validateDelayPeriod(
            clientId,
            height,
            delayPeriodTime,
            delayPeriodBlocks
        );
        return
            membershipVerifier.verifyMembership(
                appHash,
                proof,
                prefix,
                path,
                value
            );
    }

    function verifyNonMembership(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height,
        uint64 delayPeriodTime,
        uint64 delayPeriodBlocks,
        bytes calldata proof,
        bytes calldata prefix,
        bytes calldata path
    ) external returns (bool) {
        bytes memory appHash = validateDelayPeriod(
            clientId,
            height,
            delayPeriodTime,
            delayPeriodBlocks
        );
        return
            membershipVerifier.verifyNonMembership(
                appHash,
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
    ) internal view returns (bytes memory) {
        OptimizedConsensusState storage consensusState = consensusStates[
            clientId
        ][height.toUint128()];
        if (consensusState.timestamp == 0) {
            revert CometblsClientLib.ErrTrustedConsensusStateNotFound();
        }
        ProcessedMoment storage moment = processedMoments[clientId][
            height.toUint128()
        ];
        uint64 currentTime = uint64(block.timestamp);
        uint64 validTime = uint64(moment.timestamp) + delayPeriodTime;
        if (delayPeriodTime != 0 && currentTime < validTime) {
            revert CometblsClientLib.ErrDelayPeriodNotExpired();
        }
        uint64 currentHeight = uint64(block.number);
        uint64 validHeight = uint64(moment.height) + delayPeriodBlocks;
        if (delayPeriodBlocks != 0 && currentHeight < validHeight) {
            revert CometblsClientLib.ErrDelayPeriodNotExpired();
        }
        return abi.encodePacked(consensusState.appHash);
    }

    function getClientState(
        string calldata clientId
    ) external view returns (bytes memory, bool) {
        UnionIbcLightclientsCometblsV1ClientState.Data
            memory clientState = clientStates[clientId];
        if (clientState.latest_height.revision_height == 0) {
            return (bytes(""), false);
        }
        return (clientState.marshalEthABI(), true);
    }

    function getConsensusState(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height
    ) external view returns (bytes memory, bool) {
        OptimizedConsensusState memory consensusState = consensusStates[
            clientId
        ][height.toUint128()];
        if (consensusState.timestamp == 0) {
            return (bytes(""), false);
        }
        return (consensusState.marshalEthABI(), true);
    }

    function getTimestampAtHeight(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height
    ) external view override returns (uint64, bool) {
        OptimizedConsensusState memory consensusState = consensusStates[
            clientId
        ][height.toUint128()];
        return (consensusState.timestamp, consensusState.timestamp > 0);
    }

    function getLatestHeight(
        string calldata clientId
    ) external view override returns (IbcCoreClientV1Height.Data memory, bool) {
        UnionIbcLightclientsCometblsV1ClientState.Data
            memory clientState = clientStates[clientId];
        if (clientState.latest_height.revision_height == 0) {
            return (
                IbcCoreClientV1Height.Data({
                    revision_height: 0,
                    revision_number: 0
                }),
                false
            );
        }
        return (clientState.latest_height, true);
    }

    function _onlyIBC() private view {
        if (msg.sender != ibcHandler) {
            revert CometblsClientLib.ErrUnauthorized();
        }
    }

    modifier onlyIBC() {
        _onlyIBC();
        _;
    }
}
