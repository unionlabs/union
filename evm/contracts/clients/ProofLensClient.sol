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
import "../lib/Common.sol";
import "../internal/Versioned.sol";

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
}

struct ConsensusState {
    uint64 timestamp;
    uint64 l1Height;
    bytes rawL2ConsensusState;
}

library ProofLensLib {
    error ErrNotIBC();
    error ErrTrustedConsensusStateNotFound();
    error ErrClientFrozen();
    error ErrInvalidL1Proof();
    error ErrInvalidInitialConsensusState();
    error ErrInvalidMisbehaviour();

    function extractUint64(
        bytes calldata input,
        uint16 offset
    ) internal pure returns (uint64) {
        bytes8 val;
        assembly {
            val := calldataload(add(input.offset, offset))
        }
        return uint64(val);
    }

    function encode(
        ConsensusState memory consensusState
    ) internal pure returns (bytes memory) {
        return abi.encode(
            consensusState.timestamp,
            consensusState.l1Height,
            consensusState.rawL2ConsensusState
        );
    }

    function encode(
        ClientState memory clientState
    ) internal pure returns (bytes memory) {
        return abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            clientState.timestampOffset
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

contract ProofLensClient is
    ILightClient,
    Initializable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    PausableUpgradeable,
    Versioned
{
    using ProofLensLib for *;

    address public immutable IBC_HANDLER;

    uint256 private _deprecatedLegacyClientStates;
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
            revert ProofLensLib.ErrInvalidInitialConsensusState();
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
            revert ProofLensLib.ErrInvalidL1Proof();
        }

        bytes calldata rawL2ConsensusState = header.l2ConsensusState;

        if (header.l2Height > clientState.l2LatestHeight) {
            clientState.l2LatestHeight = header.l2Height;
        }

        ConsensusState storage consensusState =
            consensusStates[clientId][header.l2Height];
        consensusState.timestamp = ProofLensLib.extractUint64(
            rawL2ConsensusState, clientState.timestampOffset
        );
        consensusState.l1Height = header.l1Height;
        consensusState.rawL2ConsensusState = rawL2ConsensusState;

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
        revert ProofLensLib.ErrInvalidMisbehaviour();
    }

    function verifyMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path,
        bytes calldata value
    ) external virtual whenNotPaused returns (bool) {
        if (isFrozenImpl(clientId)) {
            revert ProofLensLib.ErrClientFrozen();
        }

        ClientState storage clientState = clientStates[clientId];
        ConsensusState storage consensusState =
            consensusStates[clientId][height];

        ILightClient l1Client =
            IBCStore(IBC_HANDLER).getClient(clientState.l1ClientId);

        return l1Client.verifyMembership(
            clientState.l1ClientId,
            consensusState.l1Height,
            proof,
            abi.encodePacked(
                IBCCommitment.membershipProofCommitmentKey(
                    clientState.l2ClientId, height, path
                )
            ),
            abi.encodePacked(keccak256(value))
        );
    }

    function verifyNonMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path
    ) external virtual whenNotPaused returns (bool) {
        if (isFrozenImpl(clientId)) {
            revert ProofLensLib.ErrClientFrozen();
        }

        ClientState storage clientState = clientStates[clientId];
        ConsensusState storage consensusState =
            consensusStates[clientId][height];

        ILightClient l1Client =
            IBCStore(IBC_HANDLER).getClient(clientState.l1ClientId);

        return l1Client.verifyMembership(
            clientState.l1ClientId,
            consensusState.l1Height,
            proof,
            abi.encodePacked(
                IBCCommitment.nonMembershipProofCommitmentKey(
                    clientState.l2ClientId, height, path
                )
            ),
            abi.encodePacked(IBCCommitment.NON_MEMBERSHIP_COMMITMENT_VALUE)
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
            revert ProofLensLib.ErrNotIBC();
        }
    }

    modifier onlyIBC() {
        _onlyIBC();
        _;
    }
}
