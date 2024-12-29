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
import "../lib/MPTVerifier.sol";

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
    uint64 l2LatestHeight;
    uint16 timestampOffset;
    uint16 stateRootOffset;
    uint16 storageRootOffset;
}

struct ConsensusState {
    uint64 timestamp;
    bytes32 stateRoot;
    bytes32 storageRoot;
}

library EvmInCosmosLib {
    uint256 public constant EVM_IBC_COMMITMENT_SLOT = 0;

    error ErrNotIBC();
    error ErrTrustedConsensusStateNotFound();
    error ErrClientFrozen();
    error ErrInvalidL1Proof();
    error ErrInvalidInitialConsensusState();
    error ErrUnsupported();

    function encode(
        ConsensusState memory consensusState
    ) internal pure returns (bytes memory) {
        return abi.encode(consensusState);
    }

    function encode(
        ClientState memory clientState
    ) internal pure returns (bytes memory) {
        return abi.encode(clientState);
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

    function extract(
        bytes calldata input,
        uint16 offset
    ) internal pure returns (bytes32 val) {
        assembly {
            val := calldataload(add(input.offset, offset))
        }
    }
}

contract EvmInCosmosClient is
    ILightClient,
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    PausableUpgradeable
{
    using EvmInCosmosLib for *;

    address private ibcHandler;

    mapping(uint32 => ClientState) private clientStates;
    mapping(uint32 => mapping(uint64 => ConsensusState)) private consensusStates;

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
        if (clientState.l2LatestHeight == 0 || consensusState.timestamp == 0) {
            revert EvmInCosmosLib.ErrInvalidInitialConsensusState();
        }
        clientStates[clientId] = clientState;
        consensusStates[clientId][clientState.l2LatestHeight] = consensusState;
        return ConsensusStateUpdate({
            clientStateCommitment: clientState.commit(),
            consensusStateCommitment: consensusState.commit(),
            height: clientState.l2LatestHeight
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
        ClientState storage clientState = clientStates[clientId];
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
                abi.encodePacked(keccak256(header.l2ConsensusState))
            )
        ) {
            revert EvmInCosmosLib.ErrInvalidL1Proof();
        }

        bytes calldata rawL2ConsensusState = header.l2ConsensusState;
        uint64 l2Timestamp = uint64(
            uint256(
                EvmInCosmosLib.extract(
                    rawL2ConsensusState, clientState.timestampOffset
                )
            )
        );
        bytes32 l2StateRoot = EvmInCosmosLib.extract(
            rawL2ConsensusState, clientState.stateRootOffset
        );
        bytes32 l2StorageRoot = EvmInCosmosLib.extract(
            rawL2ConsensusState, clientState.storageRootOffset
        );

        if (header.l2Height > clientState.l2LatestHeight) {
            clientState.l2LatestHeight = header.l2Height;
        }

        // L₂[H₂] = S₂
        // We use ethereum native encoding to make it more efficient.
        ConsensusState storage consensusState =
            consensusStates[clientId][header.l2Height];
        consensusState.timestamp = l2Timestamp;
        consensusState.stateRoot = l2StateRoot;
        consensusState.storageRoot = l2StorageRoot;

        // commit(S₂)
        return ConsensusStateUpdate({
            clientStateCommitment: clientState.commit(),
            consensusStateCommitment: consensusState.commit(),
            height: header.l2Height
        });
    }

    function misbehaviour(
        uint32 clientId,
        bytes calldata clientMessageBytes
    ) external override onlyIBC {
        revert EvmInCosmosLib.ErrUnsupported();
    }

    function verifyMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path,
        bytes calldata value
    ) external virtual returns (bool) {
        if (isFrozenImpl(clientId)) {
            revert EvmInCosmosLib.ErrClientFrozen();
        }
        bytes32 storageRoot = consensusStates[clientId][height].storageRoot;
        bytes32 slot = keccak256(
            abi.encodePacked(
                keccak256(abi.encodePacked(path)),
                EvmInCosmosLib.EVM_IBC_COMMITMENT_SLOT
            )
        );
        (bool exists, bytes calldata provenValue) =
            MPTVerifier.verifyTrieValue(proof, slot, storageRoot);
        return exists && keccak256(value) == keccak256(provenValue);
    }

    function verifyNonMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path
    ) external virtual returns (bool) {
        if (isFrozenImpl(clientId)) {
            revert EvmInCosmosLib.ErrClientFrozen();
        }
        bytes32 storageRoot = consensusStates[clientId][height].storageRoot;
        bytes32 slot = keccak256(
            abi.encodePacked(
                keccak256(abi.encodePacked(path)),
                EvmInCosmosLib.EVM_IBC_COMMITMENT_SLOT
            )
        );
        (bool exists,) = MPTVerifier.verifyTrieValue(proof, slot, storageRoot);
        return !exists;
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
        return clientStates[clientId].l2LatestHeight;
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
            revert EvmInCosmosLib.ErrNotIBC();
        }
    }

    modifier onlyIBC() {
        _onlyIBC();
        _;
    }
}
