pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin-upgradeable/utils/PausableUpgradeable.sol";
import "solidity-bytes-utils/BytesLib.sol";

import "../core/02-client/ILightClient.sol";
import "../core/24-host/IBCStore.sol";
import "../core/24-host/IBCCommitment.sol";
import "../lib/ICS23.sol";
import "../lib/Common.sol";

import "./ICS23MembershipVerifier.sol";

struct TendermintConsensusState {
    uint64 timestamp;
    bytes32 appHash;
    bytes32 nextValidatorsHash;
}

struct Header {
    uint64 l1Height;
    uint64 l2Height;
    bytes l2InclusionProof;
    bytes l2ConsensusState;
}

struct ClientState {
    string l2ChainId;
    uint32 l1ClientId;
    uint32 l2ClientId;
    uint64 latestHeight;
}

struct ConsensusState {
    uint64 timestamp;
    bytes32 appHash;
}

library CosmosInCosmosLib {
    error ErrNotIBC();
    error ErrTrustedConsensusStateNotFound();
    error ErrClientFrozen();
    error ErrInvalidL1Proof();
    error ErrInvalidInitialConsensusState();

    function encode(
        ConsensusState memory consensusState
    ) internal pure returns (bytes memory) {
        return abi.encode(consensusState.timestamp, consensusState.appHash);
    }

    function encode(
        ClientState memory clientState
    ) internal pure returns (bytes memory) {
        return abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.latestHeight
        );
    }

    function commit(
        ConsensusState memory consensusState
    ) internal pure returns (bytes32) {
        return keccak256(encode(consensusState));
    }

    function commit(
        ClientState memory clientState
    ) internal pure returns (bytes32) {
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
    using CosmosInCosmosLib for *;

    address private ibcHandler;

    mapping(uint32 => ClientState) private clientStates;
    mapping(uint32 => mapping(uint64 => ConsensusState)) private consensusStates;
    mapping(uint32 => mapping(uint64 => ProcessedMoment)) private
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
        uint32 clientId,
        bytes calldata clientStateBytes,
        bytes calldata consensusStateBytes
    ) external override onlyIBC returns (ConsensusStateUpdate memory update) {
        ClientState calldata clientState;
        assembly {
            clientState := clientStateBytes.offset
        }
        ConsensusState calldata consensusState;
        assembly {
            consensusState := consensusStateBytes.offset
        }
        if (clientState.latestHeight == 0 || consensusState.timestamp == 0) {
            revert CosmosInCosmosLib.ErrInvalidInitialConsensusState();
        }
        clientStates[clientId] = clientState;
        consensusStates[clientId][clientState.latestHeight] = consensusState;
        // Normalize to nanosecond because ibc-go recvPacket expects nanos...
        processedMoments[clientId][clientState.latestHeight] = ProcessedMoment({
            timestamp: block.timestamp * 1e9,
            height: block.number
        });
        return ConsensusStateUpdate({
            clientStateCommitment: clientState.commit(),
            consensusStateCommitment: consensusState.commit(),
            height: clientState.latestHeight
        });
    }

    /*
     * We update the L₂ client through the L₁ client.
     * Given an L₂ and L₁ heights (H₂, H₁), we prove that L₂[H₂] ∈ L₁[H₁].
     */
    function updateClient(
        uint32 clientId,
        bytes calldata clientMessageBytes
    ) external override onlyIBC returns (ConsensusStateUpdate memory) {
        Header calldata header;
        assembly {
            header := clientMessageBytes.offset
        }
        ClientState memory clientState = clientStates[clientId];
        ILightClient l1Client =
            IBCStore(ibcHandler).getClient(clientState.l1ClientId);
        // L₂[H₂] ∈ L₁[H₁]
        if (
            !l1Client.verifyMembership(
                clientState.l1ClientId,
                header.l1Height,
                header.l2InclusionProof,
                abi.encodePacked(
                    IBCCommitment.consensusStateCommitmentKey(
                        clientState.l2ClientId, header.l2Height
                    )
                ),
                abi.encodePacked(keccak256(abi.encode(header.l2ConsensusState)))
            )
        ) {
            revert CosmosInCosmosLib.ErrInvalidL1Proof();
        }

        TendermintConsensusState calldata l2ConsensusState;
        bytes calldata rawL2ConsensusState = header.l2ConsensusState;
        assembly {
            l2ConsensusState := rawL2ConsensusState.offset
        }

        if (header.l2Height > clientState.latestHeight) {
            clientState.latestHeight = header.l2Height;
        }

        // L₂[H₂] = S₂
        // We use ethereum native encoding to make it more efficient.
        ConsensusState storage consensusState =
            consensusStates[clientId][header.l2Height];
        consensusState.timestamp = l2ConsensusState.timestamp;
        consensusState.appHash = l2ConsensusState.appHash;

        // P[H₂] = now()
        ProcessedMoment storage processed =
            processedMoments[clientId][header.l2Height];
        processed.timestamp = block.timestamp * 1e9;
        processed.height = block.number;

        // commit(S₂)
        return ConsensusStateUpdate({
            clientStateCommitment: clientState.commit(),
            consensusStateCommitment: consensusState.commit(),
            height: header.l2Height
        });
    }

    function verifyMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path,
        bytes calldata value
    ) external virtual returns (bool) {
        if (isFrozenImpl(clientId)) {
            revert CosmosInCosmosLib.ErrClientFrozen();
        }
        bytes32 appHash = consensusStates[clientId][height].appHash;
        return ICS23MembershipVerifier.verifyMembership(
            appHash,
            proof,
            abi.encodePacked(IBCStoreLib.COMMITMENT_PREFIX),
            path,
            value
        );
    }

    function verifyNonMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path
    ) external virtual returns (bool) {
        if (isFrozenImpl(clientId)) {
            revert CosmosInCosmosLib.ErrClientFrozen();
        }
        bytes32 appHash = consensusStates[clientId][height].appHash;
        return ICS23MembershipVerifier.verifyNonMembership(
            appHash,
            proof,
            abi.encodePacked(IBCStoreLib.COMMITMENT_PREFIX),
            path
        );
    }

    function getClientState(
        uint32 clientId
    ) external view returns (bytes memory) {
        return clientStates[clientId].encode();
    }

    function getConsensusState(
        uint32 clientId,
        uint64 height
    ) external view returns (bytes memory) {
        return consensusStates[clientId][height].encode();
    }

    function getTimestampAtHeight(
        uint32 clientId,
        uint64 height
    ) external view override returns (uint64) {
        return consensusStates[clientId][height].timestamp;
    }

    function getLatestHeight(
        uint32 clientId
    ) external view override returns (uint64) {
        return clientStates[clientId].latestHeight;
    }

    function isFrozen(
        uint32 clientId
    ) external view virtual returns (bool) {
        return isFrozenImpl(clientId);
    }

    function isFrozenImpl(
        uint32 clientId
    ) internal view returns (bool) {
        uint32 l1ClientId = clientStates[clientId].l1ClientId;
        return IBCStore(ibcHandler).getClient(l1ClientId).isFrozen(l1ClientId);
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}

    function _onlyIBC() internal view {
        if (msg.sender != ibcHandler) {
            revert CosmosInCosmosLib.ErrNotIBC();
        }
    }

    modifier onlyIBC() {
        _onlyIBC();
        _;
    }
}
