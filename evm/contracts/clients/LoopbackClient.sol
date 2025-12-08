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
import "../lib/MPTVerifier.sol";
import "../internal/Versioned.sol";

import "solady/utils/BlockHashLib.sol";

struct EvmHeader {
    bytes32 stateRoot;
    uint256 timestamp;
}

struct Header {
    uint64 height;
    bytes encodedHeader;
}

struct ClientState {
    string chainId;
    uint64 latestHeight;
}

struct ConsensusState {
    uint64 timestamp;
    bytes32 stateRoot;
}

library LoopbackClientLib {
    error ErrNotIBC();
    error ErrInvalidInitialConsensusState();
    error ErrUnsupported();

    function encode(
        ConsensusState memory consensusState
    ) internal pure returns (bytes memory) {
        return abi.encode(consensusState.timestamp, consensusState.stateRoot);
    }

    function encode(
        ClientState memory clientState
    ) internal pure returns (bytes memory) {
        return abi.encode(clientState.chainId, clientState.latestHeight);
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

    error InvalidBlockHeaderEncoding();

    function toEvmHeader(
        bytes calldata encodedHeader
    ) internal pure returns (EvmHeader memory result) {
        // Validate minimum length and RLP list prefix
        if (encodedHeader.length <= 447 || encodedHeader[0] != 0xf9) {
            revert InvalidBlockHeaderEncoding();
        }
        // Fixed offsets for standard EVM block header fields (after 3-byte list prefix f9XXXX):
        // parentHash:       bytes 4-35   (a0 prefix at byte 3)
        // ommersHash:       bytes 37-68  (a0 prefix at byte 36)
        // beneficiary:      bytes 70-89  (94 prefix at byte 69)
        // stateRoot:        bytes 91-122 (a0 prefix at byte 90)
        // transactionsRoot: bytes 124-155 (a0 prefix at byte 123)
        // receiptsRoot:     bytes 157-188 (a0 prefix at byte 156)
        // logsBloom:        bytes 192-447 (b90100 prefix at bytes 189-191)
        result.stateRoot = bytes32(encodedHeader[91:123]);
        // Parse variable-length fields starting at offset 448
        // Fields: difficulty, number, gasLimit, gasUsed, timestamp
        uint256 ptr = 448;
        // Skip: difficulty, number, gasLimit, gasUsed
        for (uint256 i = 0; i < 4; i++) {
            ptr += _rlpFieldLength(encodedHeader, ptr);
        }
        // Decode timestamp
        result.timestamp = _decodeRlpUint(encodedHeader, ptr);
    }

    function _rlpFieldLength(
        bytes calldata data,
        uint256 offset
    ) private pure returns (uint256) {
        uint8 prefix = uint8(data[offset]);
        if (prefix < 0x80) {
            // Single byte value
            return 1;
        } else if (prefix <= 0xb7) {
            // Short string: 0x80 + length
            return 1 + (prefix - 0x80);
        } else {
            // Long string: 0xb7 + length_of_length, then length bytes
            uint8 lenOfLen = prefix - 0xb7;
            uint256 length = 0;
            for (uint8 j = 0; j < lenOfLen; j++) {
                length = (length << 8) | uint8(data[offset + 1 + j]);
            }
            return 1 + lenOfLen + length;
        }
    }

    function _decodeRlpUint(
        bytes calldata data,
        uint256 offset
    ) private pure returns (uint256) {
        uint8 prefix = uint8(data[offset]);
        if (prefix < 0x80) {
            // Single byte value
            return prefix;
        } else if (prefix <= 0xb7) {
            // Short string
            uint256 len = prefix - 0x80;
            uint256 value = 0;
            for (uint256 j = 0; j < len; j++) {
                value = (value << 8) | uint8(data[offset + 1 + j]);
            }
            return value;
        } else {
            // Long string (unlikely for timestamp)
            uint8 lenOfLen = prefix - 0xb7;
            uint256 length = 0;
            for (uint8 j = 0; j < lenOfLen; j++) {
                length = (length << 8) | uint8(data[offset + 1 + j]);
            }
            uint256 value = 0;
            for (uint256 j = 0; j < length; j++) {
                value = (value << 8) | uint8(data[offset + 1 + lenOfLen + j]);
            }
            return value;
        }
    }
}

contract LoopbackClient is
    ILightClient,
    Initializable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    PausableUpgradeable,
    Versioned
{
    using LoopbackClientLib for *;

    address public immutable IBC_HANDLER;

    mapping(uint32 => ClientState) private clientStates;
    mapping(uint32 => mapping(uint64 => ConsensusState)) private consensusStates;

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
        if (clientState.latestHeight == 0 || consensusState.timestamp == 0) {
            revert LoopbackClientLib.ErrInvalidInitialConsensusState();
        }
        clientStates[clientId] = clientState;
        consensusStates[clientId][clientState.latestHeight] = consensusState;
        return (
            ConsensusStateUpdate({
                clientStateCommitment: clientState.commit(),
                consensusStateCommitment: consensusState.commit(),
                height: clientState.latestHeight
            }),
            clientState.chainId
        );
    }

    function updateClient(
        address,
        uint32 clientId,
        bytes calldata clientMessageBytes,
        address
    ) external override whenNotPaused returns (ConsensusStateUpdate memory) {
        Header calldata header;
        assembly {
            header := clientMessageBytes.offset
        }

        BlockHashLib.verifyBlock(header.encodedHeader, uint256(header.height));

        EvmHeader memory evmHeader =
            LoopbackClientLib.toEvmHeader(header.encodedHeader);

        ClientState storage clientState = clientStates[clientId];
        if (header.height > clientState.latestHeight) {
            clientState.latestHeight = header.height;
        }

        ConsensusState storage consensusState =
            consensusStates[clientId][header.height];
        consensusState.timestamp = uint64(evmHeader.timestamp);
        consensusState.stateRoot = evmHeader.stateRoot;

        return ConsensusStateUpdate({
            clientStateCommitment: clientState.commit(),
            consensusStateCommitment: consensusState.commit(),
            height: header.height
        });
    }

    function misbehaviour(
        address,
        uint32 clientId,
        bytes calldata clientMessageBytes,
        address
    ) external override {
        revert LoopbackClientLib.ErrUnsupported();
    }

    function verifyMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path,
        bytes calldata value
    ) external virtual whenNotPaused returns (bool) {
        bytes32 stateRoot = consensusStates[clientId][height].stateRoot;
        (bool exists, bytes calldata provenValue) =
            MPTVerifier.verifyTrieValue(proof, bytes32(path), stateRoot);
        return exists && keccak256(value) == keccak256(provenValue);
    }

    function verifyNonMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path
    ) external virtual whenNotPaused returns (bool) {
        bytes32 stateRoot = consensusStates[clientId][height].stateRoot;
        (bool exists, bytes calldata provenValue) =
            MPTVerifier.verifyTrieValue(proof, bytes32(path), stateRoot);
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
        return clientStates[clientId].latestHeight;
    }

    function isFrozen(
        uint32 clientId
    ) external view virtual whenNotPaused returns (bool) {
        return false;
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
            revert LoopbackClientLib.ErrNotIBC();
        }
    }

    modifier onlyIBC() {
        _onlyIBC();
        _;
    }
}
