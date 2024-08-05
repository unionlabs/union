pragma solidity ^0.8.23;

import "@openzeppelin-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin-upgradeable/utils/PausableUpgradeable.sol";
import "solidity-bytes-utils/BytesLib.sol";

import "../core/02-client/ILightClient.sol";
import "../core/02-client/IBCHeight.sol";
import "../core/24-host/IBCStore.sol";
import "../core/24-host/IBCCommitment.sol";
import "../proto/ibc/core/client/v1/client.sol";
import {IbcLightclientsTendermintV1ConsensusState as TendermintConsensusState}
    from "../proto/ibc/lightclients/tendermint/v1/tendermint.sol";
import "../proto/cosmos/ics23/v1/proofs.sol";
import "../proto/tendermint/types/types.sol";
import "../proto/tendermint/types/canonical.sol";
import {
    UnionIbcLightclientsCosmosincosmosV1ClientState as CosmosInCosmosClientState,
    UnionIbcLightclientsCosmosincosmosV1Header as CosmosInCosmosHeader
} from "../proto/union/ibc/lightclients/cosmosincosmos/v1/cosmosincosmos.sol";
import {UnionIbcLightclientsCometblsV1ClientState as CometblsClientState} from
    "../proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";
import "../proto/ibc/lightclients/wasm/v1/wasm.sol";
import "../lib/ICS23.sol";
import "./ICS23MembershipVerifier.sol";
import {
    OptimizedConsensusState as CometblsOptimizedConsensusState,
    ProcessedMoment,
    CometblsHelp
} from "../lib/CometblsHelp.sol";

struct OptimizedCosmosInCosmosConsensusState {
    uint64 timestamp;
    bytes32 appHash;
}

library CosmosInCosmosLib {
    error ErrNotIBC();
    error ErrTrustedConsensusStateNotFound();
    error ErrDelayPeriodNotExpired();
    error ErrClientFrozen();
    error ErrInvalidL1Proof();

    function encode(OptimizedCosmosInCosmosConsensusState memory consensusState)
        internal
        pure
        returns (bytes memory)
    {
        return abi.encode(consensusState.timestamp, consensusState.appHash);
    }

    function encode(CosmosInCosmosClientState.Data memory clientState)
        internal
        pure
        returns (bytes memory)
    {
        return abi.encode(
            clientState.l2_chain_id,
            clientState.l1_client_id,
            clientState.l2_client_id,
            clientState.latest_height
        );
    }

    function commit(OptimizedCosmosInCosmosConsensusState memory consensusState)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(encode(consensusState));
    }

    function commit(CosmosInCosmosClientState.Data memory clientState)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(encode(clientState));
    }
}

contract CosmosInCosmosClient is
    ILightClient,
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    PausableUpgradeable
{
    using BytesLib for bytes;
    using IBCHeight for IbcCoreClientV1Height.Data;
    using CosmosInCosmosLib for *;

    address private ibcHandler;

    mapping(string => CosmosInCosmosClientState.Data) private clientStates;
    mapping(string => mapping(uint128 => OptimizedCosmosInCosmosConsensusState))
        private consensusStates;
    mapping(string => mapping(uint128 => ProcessedMoment)) private
        processedMoments;

    constructor() {
        _disableInitializers();
    }

    function initialize(
        address _ibcHandler,
        address admin
    ) public initializer {
        __Ownable_init(admin);
        ibcHandler = _ibcHandler;
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
        CosmosInCosmosClientState.Data calldata clientState;
        assembly {
            clientState := clientStateBytes.offset
        }
        OptimizedCosmosInCosmosConsensusState calldata consensusState;
        assembly {
            consensusState := consensusStateBytes.offset
        }
        if (
            clientState.latest_height.revision_height == 0
                || consensusState.timestamp == 0
        ) {
            return (clientStateCommitment, update, false);
        }
        clientStates[clientId] = clientState;
        uint128 latestHeight = clientState.latest_height.toUint128();
        consensusStates[clientId][latestHeight] = consensusState;
        // Normalize to nanosecond because ibc-go recvPacket expects nanos...
        processedMoments[clientId][latestHeight] = ProcessedMoment({
            timestamp: block.timestamp * 1e9,
            height: block.number
        });
        return (
            clientState.commit(),
            ConsensusStateUpdate({
                consensusStateCommitment: consensusState.commit(),
                height: clientState.latest_height
            }),
            true
        );
    }

    /*
     * We update the L₂ client through the L₁ client.
     * Given an L₂ and L₁ heights (H₂, H₁), we prove that L₂[H₂] ∈ L₁[H₁].
     */
    function updateClient(
        string calldata clientId,
        bytes calldata clientMessageBytes
    )
        external
        override
        onlyIBC
        returns (bytes32, ConsensusStateUpdate[] memory)
    {
        CosmosInCosmosHeader.Data calldata header;
        assembly {
            header := clientMessageBytes.offset
        }
        CosmosInCosmosClientState.Data memory clientState =
            clientStates[clientId];
        ILightClient l1Client =
            IBCStore(ibcHandler).getClient(clientState.l1_client_id);
        // L₂[H₂] ∈ L₁[H₁]
        if (
            !l1Client.verifyMembership(
                clientState.l1_client_id,
                header.l1_height,
                0,
                0,
                header.l2_inclusion_proof,
                bytes(IBCStoreLib.COMMITMENT_PREFIX),
                IBCCommitment.consensusStatePath(
                    clientState.l2_client_id,
                    header.l2_height.revision_number,
                    header.l2_height.revision_height
                ),
                header.l2_consensus_state
            )
        ) {
            revert CosmosInCosmosLib.ErrInvalidL1Proof();
        }
        TendermintConsensusState.Data memory l2ConsensusState =
            TendermintConsensusState.decode(header.l2_consensus_state);

        if (header.l2_height.gt(clientState.latest_height)) {
            clientState.latest_height = header.l2_height;
        }

        uint128 l2HeightIndex = header.l2_height.toUint128();

        // Cosmos expects nanos...
        uint64 l2Timestamp = uint64(l2ConsensusState.timestamp.secs) * 1e9
            + uint64(l2ConsensusState.timestamp.nanos);

        // L₂[H₂] = optimize(S₂)
        // The default tendermint consensus state is stored as protobuf.
        // We use ethereum native encoding to make it more efficient.
        OptimizedCosmosInCosmosConsensusState storage consensusState =
            consensusStates[clientId][l2HeightIndex];
        consensusState.timestamp = l2Timestamp;
        consensusState.appHash = l2ConsensusState.root.hash.toBytes32(0);

        // P[H₂] = now()
        ProcessedMoment storage processed =
            processedMoments[clientId][l2HeightIndex];
        processed.timestamp = block.timestamp * 1e9;
        processed.height = block.number;

        // commit(optimize(S₂))
        ConsensusStateUpdate[] memory updates = new ConsensusStateUpdate[](1);
        updates[0] = ConsensusStateUpdate({
            consensusStateCommitment: consensusState.commit(),
            height: header.l2_height
        });

        return (clientState.commit(), updates);
    }

    function verifyMembership(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height,
        uint64 delayPeriodTime,
        uint64 delayPeriodBlocks,
        bytes calldata proof,
        bytes calldata prefix,
        bytes calldata path,
        bytes calldata value
    ) external virtual returns (bool) {
        if (isFrozenImpl(clientId)) {
            revert CosmosInCosmosLib.ErrClientFrozen();
        }
        bytes32 appHash = validateDelayPeriod(
            clientId, height, delayPeriodTime, delayPeriodBlocks
        );
        return ICS23MembershipVerifier.verifyMembership(
            appHash, proof, prefix, path, value
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
    ) external virtual returns (bool) {
        if (isFrozenImpl(clientId)) {
            revert CosmosInCosmosLib.ErrClientFrozen();
        }
        bytes32 appHash = validateDelayPeriod(
            clientId, height, delayPeriodTime, delayPeriodBlocks
        );
        return ICS23MembershipVerifier.verifyNonMembership(
            appHash, proof, prefix, path
        );
    }

    function validateDelayPeriod(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height,
        uint64 delayPeriodTime,
        uint64 delayPeriodBlocks
    ) internal view returns (bytes32) {
        OptimizedCosmosInCosmosConsensusState storage consensusState =
            consensusStates[clientId][height.toUint128()];
        if (consensusState.timestamp == 0) {
            revert CosmosInCosmosLib.ErrTrustedConsensusStateNotFound();
        }
        ProcessedMoment storage moment =
            processedMoments[clientId][height.toUint128()];
        uint64 currentTime = uint64(block.timestamp * 1e9);
        uint64 validTime = uint64(moment.timestamp) + delayPeriodTime;
        if (delayPeriodTime != 0 && currentTime < validTime) {
            revert CosmosInCosmosLib.ErrDelayPeriodNotExpired();
        }
        uint64 currentHeight = uint64(block.number);
        uint64 validHeight = uint64(moment.height) + delayPeriodBlocks;
        if (delayPeriodBlocks != 0 && currentHeight < validHeight) {
            revert CosmosInCosmosLib.ErrDelayPeriodNotExpired();
        }
        return consensusState.appHash;
    }

    function getClientState(string calldata clientId)
        external
        view
        returns (bytes memory)
    {
        return clientStates[clientId].encode();
    }

    function getConsensusState(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height
    ) external view returns (bytes memory) {
        return consensusStates[clientId][height.toUint128()].encode();
    }

    function getTimestampAtHeight(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height
    ) external view override returns (uint64) {
        return consensusStates[clientId][height.toUint128()].timestamp;
    }

    function getLatestHeight(string calldata clientId)
        external
        view
        override
        returns (IbcCoreClientV1Height.Data memory)
    {
        return clientStates[clientId].latest_height;
    }

    function isFrozen(string calldata clientId)
        external
        view
        virtual
        returns (bool)
    {
        return isFrozenImpl(clientId);
    }

    function isFrozenImpl(string calldata clientId)
        internal
        view
        returns (bool)
    {
        string memory l1ClientId = clientStates[clientId].l1_client_id;
        return IBCStore(ibcHandler).getClient(l1ClientId).isFrozen(l1ClientId);
    }

    function _authorizeUpgrade(address newImplementation)
        internal
        override
        onlyOwner
    {}

    function _onlyIBC() private view {
        if (msg.sender != ibcHandler) {
            revert CosmosInCosmosLib.ErrNotIBC();
        }
    }

    modifier onlyIBC() {
        _onlyIBC();
        _;
    }
}
