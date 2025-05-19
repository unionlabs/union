pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";
import "@openzeppelin-upgradeable/contracts/utils/PausableUpgradeable.sol";
import "solidity-bytes-utils/BytesLib.sol";

import "../core/02-client/ILightClient.sol";
import "../core/24-host/IBCStore.sol";
import "../core/24-host/IBCCommitment.sol";
import "../lib/ICS23.sol";
import "../lib/Common.sol";
import "../lib/ICS23Verifier.sol";
import "../internal/Versioned.sol";

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

struct LegacyClientState {
    string l2ChainId;
    uint32 l1ClientId;
    uint32 l2ClientId;
    uint64 l2LatestHeight;
    bytes32 contractAddress;
}

struct ClientState {
    string l2ChainId;
    uint32 l1ClientId;
    uint32 l2ClientId;
    uint64 l2LatestHeight;
    uint256 version;
    bytes state;
}

struct ExtraV1 {
    bytes storeKey;
    bytes keyPrefixStorage;
}

struct ConsensusState {
    uint64 timestamp;
    bytes32 appHash;
}

library StateLensIcs23Ics23Lib {
    error ErrNotIBC();
    error ErrTrustedConsensusStateNotFound();
    error ErrClientFrozen();
    error ErrInvalidL1Proof();
    error ErrInvalidInitialConsensusState();
    error ErrInvalidMisbehaviour();
    // only V1 is supported for new clients
    error ErrUnsupportedVersion();
    error ErrUnknownClientStateVersion(uint256 version);

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
            clientState.l2LatestHeight,
            clientState.version,
            clientState.state
        );
    }

    function encode(
        ExtraV1 memory extraV1
    ) internal pure returns (bytes memory) {
        return abi.encode(extraV1.storeKey, extraV1.keyPrefixStorage);
    }

    function encode(
        ExtraV1 storage extraV1
    ) internal view returns (bytes memory) {
        return abi.encode(extraV1.storeKey, extraV1.keyPrefixStorage);
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

contract StateLensIcs23Ics23Client is
    ILightClient,
    Initializable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    PausableUpgradeable,
    Versioned
{
    using StateLensIcs23Ics23Lib for *;

    address public immutable IBC_HANDLER;

    mapping(uint32 => LegacyClientState) private legacyClientStates;
    mapping(uint32 => mapping(uint64 => ConsensusState)) private consensusStates;

    mapping(uint32 => ClientState) private clientStates;

    constructor(
        address _ibcHandler
    ) {
        _disableInitializers();
        IBC_HANDLER = _ibcHandler;
    }

    function initialize(
        address authority
    ) public initializer {
        __AccessManaged_init(authority);
        __UUPSUpgradeable_init();
        __Pausable_init();
    }

    function migrateClientStateToV1(
        uint32[] calldata clientIds
    ) public restricted {
        for (uint256 i = 0; i < clientIds.length; i++) {
            LegacyClientState storage legacyState =
                legacyClientStates[clientIds[i]];
            ClientState storage newState = clientStates[clientIds[i]];

            newState.l2ChainId = legacyState.l2ChainId;
            newState.l1ClientId = legacyState.l1ClientId;
            newState.l2ClientId = legacyState.l2ClientId;
            newState.l2LatestHeight = legacyState.l2LatestHeight;
            newState.version = uint256(1);
            newState.state = ExtraV1({
                storeKey: IBCStoreLib.WASMD_MODULE_STORE_KEY,
                keyPrefixStorage: abi.encodePacked(
                    IBCStoreLib.WASMD_CONTRACT_STORE_PREFIX,
                    legacyState.contractAddress,
                    IBCStoreLib.IBC_UNION_COSMWASM_COMMITMENT_PREFIX
                )
            }).encode();
        }
    }

    function createClient(
        address,
        uint32 clientId,
        bytes calldata clientStateBytes,
        bytes calldata consensusStateBytes,
        address
    )
        external
        override
        onlyIBC
        whenNotPaused
        returns (
            ConsensusStateUpdate memory update,
            string memory counterpartyChainId
        )
    {
        ClientState calldata clientState;
        assembly {
            clientState := clientStateBytes.offset
        }
        ConsensusState calldata consensusState;
        assembly {
            consensusState := consensusStateBytes.offset
        }
        if (clientState.l2LatestHeight == 0 || consensusState.timestamp == 0) {
            revert StateLensIcs23Ics23Lib.ErrInvalidInitialConsensusState();
        }
        clientStates[clientId] = clientState;
        consensusStates[clientId][clientState.l2LatestHeight] = consensusState;

        emit CreateLensClient(
            clientId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2ChainId
        );

        return (
            ConsensusStateUpdate({
                clientStateCommitment: clientState.commit(),
                consensusStateCommitment: consensusState.commit(),
                height: clientState.l2LatestHeight
            }),
            clientState.l2ChainId
        );
    }

    /*
     * We update the L₂ client through the L₁ client.
     * Given an L₂ and L₁ heights (H₂, H₁), we prove that L₂[H₂] ∈ L₁[H₁].
     */
    function updateClient(
        address,
        uint32 clientId,
        bytes calldata clientMessageBytes,
        address
    )
        external
        override
        onlyIBC
        whenNotPaused
        returns (ConsensusStateUpdate memory)
    {
        Header calldata header;
        assembly {
            header := clientMessageBytes.offset
        }
        ClientState storage clientState = clientStates[clientId];
        ILightClient l1Client =
            IBCStore(IBC_HANDLER).getClient(clientState.l1ClientId);
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
            revert StateLensIcs23Ics23Lib.ErrInvalidL1Proof();
        }

        TendermintConsensusState calldata l2ConsensusState;
        bytes calldata rawL2ConsensusState = header.l2ConsensusState;
        assembly {
            l2ConsensusState := rawL2ConsensusState.offset
        }

        if (header.l2Height > clientState.l2LatestHeight) {
            clientState.l2LatestHeight = header.l2Height;
        }

        // L₂[H₂] = S₂
        // We use ethereum native encoding to make it more efficient.
        ConsensusState storage consensusState =
            consensusStates[clientId][header.l2Height];
        consensusState.timestamp = l2ConsensusState.timestamp;
        consensusState.appHash = l2ConsensusState.appHash;

        // commit(S₂)
        return ConsensusStateUpdate({
            clientStateCommitment: clientState.commit(),
            consensusStateCommitment: consensusState.commit(),
            height: header.l2Height
        });
    }

    function misbehaviour(
        address,
        uint32 clientId,
        bytes calldata clientMessageBytes,
        address
    ) external override onlyIBC whenNotPaused {
        revert StateLensIcs23Ics23Lib.ErrInvalidMisbehaviour();
    }

    function verifyMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path,
        bytes calldata value
    ) external virtual whenNotPaused returns (bool) {
        if (isFrozenImpl(clientId)) {
            revert StateLensIcs23Ics23Lib.ErrClientFrozen();
        }
        ClientState storage clientState = clientStates[clientId];
        bytes32 appHash = consensusStates[clientId][height].appHash;

        if (clientState.version != 1) {
            revert StateLensIcs23Ics23Lib.ErrUnknownClientStateVersion(
                clientState.version
            );
        }
        (bytes memory storeKey, bytes memory keyPrefixStorage) =
            abi.decode(clientState.state, (bytes, bytes));

        return ICS23Verifier.verifyMembership(
            appHash,
            proof,
            storeKey,
            abi.encodePacked(keyPrefixStorage, path),
            value
        );
    }

    function verifyNonMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path
    ) external virtual whenNotPaused returns (bool) {
        if (isFrozenImpl(clientId)) {
            revert StateLensIcs23Ics23Lib.ErrClientFrozen();
        }
        ClientState storage clientState = clientStates[clientId];
        bytes32 appHash = consensusStates[clientId][height].appHash;

        if (clientState.version != 1) {
            revert StateLensIcs23Ics23Lib.ErrUnknownClientStateVersion(
                clientState.version
            );
        }
        (bytes memory storeKey, bytes memory keyPrefixStorage) =
            abi.decode(clientState.state, (bytes, bytes));

        return ICS23Verifier.verifyNonMembership(
            appHash, proof, storeKey, abi.encodePacked(keyPrefixStorage, path)
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
        return clientStates[clientId].l2LatestHeight;
    }

    function isFrozen(
        uint32 clientId
    ) external view virtual whenNotPaused returns (bool) {
        return isFrozenImpl(clientId);
    }

    function isFrozenImpl(
        uint32 clientId
    ) internal view returns (bool) {
        uint32 l1ClientId = clientStates[clientId].l1ClientId;
        return IBCStore(IBC_HANDLER).getClient(l1ClientId).isFrozen(l1ClientId);
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override restricted {}

    function pause() public restricted {
        _pause();
    }

    function unpause() public restricted {
        _unpause();
    }

    function _onlyIBC() internal view {
        if (msg.sender != IBC_HANDLER) {
            revert StateLensIcs23Ics23Lib.ErrNotIBC();
        }
    }

    modifier onlyIBC() {
        _onlyIBC();
        _;
    }
}
