pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin-upgradeable/contracts/access/Ownable2StepUpgradeable.sol";
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

struct ClientState {
    string l2ChainId;
    uint32 l1ClientId;
    uint32 l2ClientId;
    uint64 l2LatestHeight;
    bytes32 contractAddress;
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
            clientState.contractAddress
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

contract StateLensIcs23Ics23Client is
    ILightClient,
    Initializable,
    UUPSUpgradeable,
    Ownable2StepUpgradeable,
    PausableUpgradeable,
    Versioned
{
    using StateLensIcs23Ics23Lib for *;

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
        address,
        uint32 clientId,
        bytes calldata clientStateBytes,
        bytes calldata consensusStateBytes,
        address
    )
        external
        override
        onlyIBC
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
        // Normalize to nanosecond because ibc-go recvPacket expects nanos...
        processedMoments[clientId][clientState.l2LatestHeight] = ProcessedMoment({
            timestamp: block.timestamp * 1e9,
            height: block.number
        });

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
        bytes32 contractAddress = clientStates[clientId].contractAddress;
        bytes32 appHash = consensusStates[clientId][height].appHash;
        return ICS23Verifier.verifyMembership(
            appHash,
            proof,
            IBCStoreLib.WASMD_MODULE_STORE_KEY,
            abi.encodePacked(
                IBCStoreLib.WASMD_CONTRACT_STORE_PREFIX,
                contractAddress,
                IBCStoreLib.IBC_UNION_COSMWASM_COMMITMENT_PREFIX,
                path
            ),
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
        bytes32 contractAddress = clientStates[clientId].contractAddress;
        bytes32 appHash = consensusStates[clientId][height].appHash;
        return ICS23Verifier.verifyNonMembership(
            appHash,
            proof,
            IBCStoreLib.WASMD_MODULE_STORE_KEY,
            abi.encodePacked(
                IBCStoreLib.WASMD_CONTRACT_STORE_PREFIX,
                contractAddress,
                IBCStoreLib.IBC_UNION_COSMWASM_COMMITMENT_PREFIX,
                path
            )
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
        return IBCStore(ibcHandler).getClient(l1ClientId).isFrozen(l1ClientId);
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}

    function pause() public onlyOwner {
        _pause();
    }

    function unpause() public onlyOwner {
        _unpause();
    }

    function _onlyIBC() internal view {
        if (msg.sender != ibcHandler) {
            revert StateLensIcs23Ics23Lib.ErrNotIBC();
        }
    }

    modifier onlyIBC() {
        _onlyIBC();
        _;
    }
}
