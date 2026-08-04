pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";
import "@openzeppelin-upgradeable/contracts/utils/PausableUpgradeable.sol";
import "solady/utils/LibString.sol";

import "../core/02-client/ILightClient.sol";
import "../core/24-host/IBCStore.sol";
import "../core/24-host/IBCCommitment.sol";
import "../lib/UnionICS23.sol";
import "../lib/BptreeICS23.sol";
import "../internal/Versioned.sol";

// state-lens/ics23/bptree: an L1 (ics23) client tracks an L2 whose own store
// is gno.land's bptree `main` store. Kept separate from
// StateLensIcs23Ics23Client (which assumes an IAVL-shaped L2 store) rather
// than adding a version to it, per the same reasoning as BptreeICS23.sol.
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
    uint64 l2LatestHeight;
    bytes storeKey;
    bytes keyPrefixStorage;
}

struct ConsensusState {
    uint64 timestamp;
    bytes32 appHash;
}

struct NonMembershipProof {
    UnionIcs23.NonExistenceProof l2;
    UnionIcs23.ExistenceProof l1;
}

library StateLensIcs23BptreeLib {
    error ErrNotIBC();
    error ErrClientFrozen();
    error ErrInvalidL1Proof();
    error ErrInvalidInitialConsensusState();
    error ErrInvalidMisbehaviour();

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
            clientState.storeKey,
            clientState.keyPrefixStorage
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

contract StateLensIcs23BptreeClient is
    ILightClient,
    Initializable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    PausableUpgradeable,
    Versioned
{
    using StateLensIcs23BptreeLib for *;
    using LibString for bytes;

    address public immutable IBC_HANDLER;

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
            revert StateLensIcs23BptreeLib.ErrInvalidInitialConsensusState();
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
            revert StateLensIcs23BptreeLib.ErrInvalidL1Proof();
        }

        TendermintConsensusState calldata l2ConsensusState;
        bytes calldata rawL2ConsensusState = header.l2ConsensusState;
        assembly {
            l2ConsensusState := rawL2ConsensusState.offset
        }

        if (header.l2Height > clientState.l2LatestHeight) {
            clientState.l2LatestHeight = header.l2Height;
        }

        ConsensusState storage consensusState =
            consensusStates[clientId][header.l2Height];
        consensusState.timestamp = l2ConsensusState.timestamp;
        consensusState.appHash = l2ConsensusState.appHash;

        return ConsensusStateUpdate({
            clientStateCommitment: clientState.commit(),
            consensusStateCommitment: consensusState.commit(),
            height: header.l2Height
        });
    }

    function misbehaviour(
        address,
        uint32,
        bytes calldata,
        address
    ) external override onlyIBC whenNotPaused {
        revert StateLensIcs23BptreeLib.ErrInvalidMisbehaviour();
    }

    function verifyMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path,
        bytes calldata value
    ) external virtual whenNotPaused returns (bool) {
        if (isFrozenImpl(clientId)) {
            revert StateLensIcs23BptreeLib.ErrClientFrozen();
        }
        ClientState storage clientState = clientStates[clientId];
        bytes32 appHash = consensusStates[clientId][height].appHash;

        UnionIcs23.ExistenceProof[2] calldata proofs;
        assembly {
            proofs := proof.offset
        }

        bytes memory key = abi.encodePacked(
            clientState.keyPrefixStorage, path.toHexStringNoPrefix()
        );

        return BptreeIcs23.verifyChainedMembership(
            proofs, appHash, clientState.storeKey, key, value
        ) == BptreeIcs23.VerifyChainedMembershipError.None;
    }

    function verifyNonMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path
    ) external virtual whenNotPaused returns (bool) {
        if (isFrozenImpl(clientId)) {
            revert StateLensIcs23BptreeLib.ErrClientFrozen();
        }
        ClientState storage clientState = clientStates[clientId];
        bytes32 appHash = consensusStates[clientId][height].appHash;

        NonMembershipProof calldata p;
        assembly {
            p := proof.offset
        }

        // same no-prefix encoding as verifyMembership; gno's own key
        // construction always uses HexUnprefixed regardless of
        // membership/non-membership (v3's toHexString-with-prefix here is
        // untested and looks like a bug, not a convention to replicate).
        bytes memory key = abi.encodePacked(
            clientState.keyPrefixStorage, path.toHexStringNoPrefix()
        );

        return BptreeIcs23.verifyChainedNonMembership(
            p.l2, p.l1, appHash, clientState.storeKey, key
        ) == BptreeIcs23.VerifyChainedNonMembershipError.None;
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
            revert StateLensIcs23BptreeLib.ErrNotIBC();
        }
    }

    modifier onlyIBC() {
        _onlyIBC();
        _;
    }
}
