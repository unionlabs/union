pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";
import "@openzeppelin-upgradeable/contracts/utils/PausableUpgradeable.sol";

import "../core/02-client/ILightClient.sol";
import "../core/24-host/IBCStore.sol";
import "../core/24-host/IBCCommitment.sol";
import "../lib/Common.sol";
import "../lib/ICS23.sol";
import "../lib/CometblsZKVerifier.sol";
import "../lib/ICS23Verifier.sol";
import "../internal/Versioned.sol";

struct SignedHeader {
    uint64 height;
    uint64 secs;
    uint64 nanos;
    bytes32 validatorsHash;
    bytes32 nextValidatorsHash;
    bytes32 appHash;
}

struct Header {
    SignedHeader signedHeader;
    uint64 trustedHeight;
    bytes zeroKnowledgeProof;
}

struct ClientState {
    bytes31 chainId;
    uint64 trustingPeriod;
    uint64 maxClockDrift;
    uint64 frozenHeight;
    uint64 latestHeight;
    bytes32 contractAddress;
}

struct ConsensusState {
    uint64 timestamp;
    bytes32 appHash;
    bytes32 nextValidatorsHash;
}

struct Misbehaviour {
    Header headerA;
    Header headerB;
}

library CometblsClientLib {
    error ErrNotIBC();
    error ErrTrustedConsensusStateNotFound();
    error ErrUntrustedHeightLTETrustedHeight();
    error ErrUntrustedTimestampLTETrustedTimestamp();
    error ErrHeaderExpired();
    error ErrMaxClockDriftExceeded();
    error ErrInvalidZKP();
    error ErrInvalidUntrustedValidatorsHash();
    error ErrInvalidMisbehaviourHeadersSequence();
    error ErrInvalidMisbehaviour();
    error ErrClientFrozen();
    error ErrInvalidInitialConsensusState();

    function isExpired(
        uint64 headerTime,
        uint64 trustingPeriod,
        uint64 currentTime
    ) internal pure returns (bool) {
        return uint256(currentTime)
            > (uint256(headerTime) + uint256(trustingPeriod));
    }

    function encodeMemory(
        Header memory header
    ) internal pure returns (bytes memory) {
        return abi.encode(
            header.signedHeader, header.trustedHeight, header.zeroKnowledgeProof
        );
    }

    function encode(
        Header calldata header
    ) internal pure returns (bytes memory) {
        return abi.encode(
            header.signedHeader, header.trustedHeight, header.zeroKnowledgeProof
        );
    }

    function decodeHeader(
        bytes calldata bz
    ) internal pure returns (Header calldata) {
        Header calldata header;
        assembly {
            header := bz.offset
        }
        return header;
    }

    function decodeMisbehaviour(
        bytes calldata bz
    ) internal pure returns (Misbehaviour calldata) {
        Misbehaviour calldata misbehaviour;
        assembly {
            misbehaviour := bz.offset
        }
        return misbehaviour;
    }

    function encodeMemory(
        ClientState memory clientState
    ) internal pure returns (bytes memory) {
        return abi.encode(
            clientState.chainId,
            clientState.trustingPeriod,
            clientState.maxClockDrift,
            clientState.frozenHeight,
            clientState.latestHeight,
            clientState.contractAddress
        );
    }

    function decodeClientState(
        bytes calldata bz
    ) internal pure returns (ClientState calldata) {
        ClientState calldata clientState;
        assembly {
            clientState := bz.offset
        }
        return clientState;
    }

    function encodeMemory(
        ConsensusState memory consensusState
    ) internal pure returns (bytes memory) {
        return abi.encode(
            consensusState.timestamp,
            consensusState.appHash,
            consensusState.nextValidatorsHash
        );
    }

    function decodeConsensusState(
        bytes calldata bz
    ) internal pure returns (ConsensusState calldata) {
        ConsensusState calldata consensusState;
        assembly {
            consensusState := bz.offset
        }
        return consensusState;
    }

    function decodeConsensusStateMemory(
        bytes memory bz
    ) internal pure returns (ConsensusState memory) {
        ConsensusState memory consensusState;
        (uint64 timestamp, bytes32 appHash, bytes32 nextValidatorsHash) =
            abi.decode(bz, (uint64, bytes32, bytes32));
        consensusState.timestamp = timestamp;
        consensusState.appHash = appHash;
        consensusState.nextValidatorsHash = nextValidatorsHash;
        return consensusState;
    }

    function commit(
        ConsensusState memory consensusState
    ) internal pure returns (bytes32) {
        return keccak256(encodeMemory(consensusState));
    }

    function commit(
        ClientState memory clientState
    ) internal pure returns (bytes32) {
        return keccak256(encodeMemory(clientState));
    }

    function chainIdToString(
        bytes31 source
    ) internal pure returns (string memory result) {
        uint8 offset = 0;
        while (source[offset] == 0 && offset < 31) {
            offset++;
        }
        assembly {
            result := mload(0x40)
            // new "memory end" including padding (the string isn't larger than 32 bytes)
            mstore(0x40, add(result, 0x40))
            // store length in memory
            mstore(result, sub(31, offset))
            // write actual data
            mstore(add(result, 0x20), shl(mul(offset, 8), source))
        }
    }
}

contract CometblsClient is
    ILightClient,
    Initializable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    PausableUpgradeable,
    Versioned
{
    using CometblsClientLib for *;

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
        ClientState calldata clientState = clientStateBytes.decodeClientState();
        ConsensusState calldata consensusState =
            consensusStateBytes.decodeConsensusState();
        if (clientState.latestHeight == 0 || consensusState.timestamp == 0) {
            revert CometblsClientLib.ErrInvalidInitialConsensusState();
        }
        clientStates[clientId] = clientState;
        consensusStates[clientId][clientState.latestHeight] = consensusState;
        return (
            ConsensusStateUpdate({
                clientStateCommitment: clientState.commit(),
                consensusStateCommitment: consensusState.commit(),
                height: clientState.latestHeight
            }),
            CometblsClientLib.chainIdToString(clientState.chainId)
        );
    }

    function misbehaviour(
        address,
        uint32 clientId,
        bytes calldata clientMessageBytes,
        address
    ) external override onlyIBC whenNotPaused {
        Misbehaviour calldata m = clientMessageBytes.decodeMisbehaviour();
        ClientState storage clientState = clientStates[clientId];
        bool fraud =
            checkMisbehaviour(clientId, clientState, m.headerA, m.headerB);
        if (!fraud) {
            revert CometblsClientLib.ErrInvalidMisbehaviour();
        }
        // Similar to tendermint https://github.com/cosmos/ibc-go/blob/bbdcc8c6e965c8a2f607dfb2b61cd13712dd966a/modules/light-clients/07-tendermint/misbehaviour.go#L19
        clientState.frozenHeight = 1;
    }

    function checkMisbehaviour(
        uint32 clientId,
        ClientState storage clientState,
        Header calldata headerA,
        Header calldata headerB
    ) internal returns (bool) {
        // Ensures that A > B to simplify the misbehaviour of time violation check
        if (headerA.signedHeader.height < headerB.signedHeader.height) {
            revert CometblsClientLib.ErrInvalidMisbehaviourHeadersSequence();
        }

        ConsensusState storage consensusStateA =
            consensusStates[clientId][headerA.trustedHeight];
        ConsensusState storage consensusStateB =
            consensusStates[clientId][headerB.trustedHeight];

        // Check that the headers would have been accepted in an update
        (, uint64 untrustedTimestampA) =
            verifyHeader(headerA, consensusStateA, clientState);
        (, uint64 untrustedTimestampB) =
            verifyHeader(headerB, consensusStateB, clientState);

        if (headerA.signedHeader.height == headerB.signedHeader.height) {
            bytes32 hashA = keccak256(abi.encode(headerA.signedHeader));
            bytes32 hashB = keccak256(abi.encode(headerB.signedHeader));
            if (hashA != hashB) {
                // Misbehaviour of a fork
                return true;
            }
        } else {
            // Guarantee that A > B
            if (untrustedTimestampA <= untrustedTimestampB) {
                // Misbehaviour of time violation
                return true;
            }
        }
        return false;
    }

    function checkOverwriteMisbehavior(
        uint64 untrustedTimestamp,
        bytes32 untrustedAppHash,
        bytes32 untrustedNextValidatorsHash,
        ConsensusState storage overwrittenConsensusState
    ) internal returns (bool) {
        if (
            untrustedTimestamp != overwrittenConsensusState.timestamp
                || untrustedAppHash != overwrittenConsensusState.appHash
                || untrustedNextValidatorsHash
                    != overwrittenConsensusState.nextValidatorsHash
        ) {
            return true;
        }
        return false;
    }

    function verifyHeader(
        Header calldata header,
        ConsensusState storage consensusState,
        ClientState storage clientState
    ) internal returns (uint64, uint64) {
        if (consensusState.timestamp == 0) {
            revert CometblsClientLib.ErrTrustedConsensusStateNotFound();
        }

        uint64 untrustedHeightNumber = header.signedHeader.height;
        uint64 trustedHeightNumber = header.trustedHeight;
        if (untrustedHeightNumber <= trustedHeightNumber) {
            revert CometblsClientLib.ErrUntrustedHeightLTETrustedHeight();
        }

        uint64 trustedTimestamp = consensusState.timestamp;
        // Normalize to nanosecond because ibc-go recvPacket expects nanos...
        uint64 untrustedTimestamp =
            header.signedHeader.secs * 1e9 + header.signedHeader.nanos;
        if (untrustedTimestamp <= trustedTimestamp) {
            revert CometblsClientLib.ErrUntrustedTimestampLTETrustedTimestamp();
        }

        // Normalize to nanosecond because ibc-go recvPacket expects nanos...
        uint64 currentTime = uint64(block.timestamp * 1e9);
        if (
            CometblsClientLib.isExpired(
                trustedTimestamp, clientState.trustingPeriod, currentTime
            )
        ) {
            revert CometblsClientLib.ErrHeaderExpired();
        }

        uint64 maxClockDrift = currentTime + clientState.maxClockDrift;
        if (untrustedTimestamp >= maxClockDrift) {
            revert CometblsClientLib.ErrMaxClockDriftExceeded();
        }

        /*
         We want to verify that 1/3 of trusted valset & 2/3 of untrusted valset signed.
         In adjacent verification, trusted vals = untrusted vals.
         In non adjacent verification, untrusted vals are coming from the untrusted header.
         */
        bytes32 trustedValidatorsHash = consensusState.nextValidatorsHash;
        bool adjacent = untrustedHeightNumber == trustedHeightNumber + 1;
        if (adjacent) {
            if (header.signedHeader.validatorsHash != trustedValidatorsHash) {
                revert CometblsClientLib.ErrInvalidUntrustedValidatorsHash();
            }
        }

        bool ok = internalVerifyZKP(
            header.zeroKnowledgeProof,
            clientState.chainId,
            trustedValidatorsHash,
            header.signedHeader
        );
        if (!ok) {
            revert CometblsClientLib.ErrInvalidZKP();
        }

        return (untrustedHeightNumber, untrustedTimestamp);
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
        ClientState storage clientState = clientStates[clientId];
        if (clientState.frozenHeight > 0) {
            revert CometblsClientLib.ErrClientFrozen();
        }

        Header calldata header = clientMessageBytes.decodeHeader();

        ConsensusState storage consensusState =
            consensusStates[clientId][header.trustedHeight];

        (uint64 untrustedHeightNumber, uint64 untrustedTimestamp) =
            verifyHeader(header, consensusState, clientState);

        // Update states
        if (untrustedHeightNumber > clientState.latestHeight) {
            clientState.latestHeight = untrustedHeightNumber;
        }

        consensusState = consensusStates[clientId][untrustedHeightNumber];
        // Verify misbehavior on overwrite
        if (consensusState.timestamp != 0) {
            if (
                checkOverwriteMisbehavior(
                    untrustedTimestamp,
                    header.signedHeader.appHash,
                    header.signedHeader.nextValidatorsHash,
                    consensusState
                )
            ) {
                clientState.frozenHeight = 1;
            }
            // Noop
        } else {
            consensusState.timestamp = untrustedTimestamp;
            consensusState.appHash = header.signedHeader.appHash;
            consensusState.nextValidatorsHash =
                header.signedHeader.nextValidatorsHash;
        }
        return ConsensusStateUpdate({
            clientStateCommitment: clientState.commit(),
            consensusStateCommitment: consensusState.commit(),
            height: untrustedHeightNumber
        });
    }

    function verifyMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path,
        bytes calldata value
    ) external virtual whenNotPaused returns (bool) {
        if (isFrozenImpl(clientId)) {
            revert CometblsClientLib.ErrClientFrozen();
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
            revert CometblsClientLib.ErrClientFrozen();
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
        return clientStates[clientId].encodeMemory();
    }

    function getConsensusState(
        uint32 clientId,
        uint64 height
    ) external view returns (bytes memory) {
        return consensusStates[clientId][height].encodeMemory();
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
        return isFrozenImpl(clientId);
    }

    function isFrozenImpl(
        uint32 clientId
    ) internal view returns (bool) {
        return clientStates[clientId].frozenHeight > 0;
    }

    // ZKP VERIFICATION
    uint256 constant PRIME_R =
        21888242871839275222246405745257275088548364400416034343698204186575808495617;
    uint256 constant PRIME_R_MINUS_ONE = PRIME_R - 1;

    bytes constant HMAC_I =
        hex"75595B5342747A653636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636";
    bytes constant HMAC_O =
        hex"1F333139281E100F5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C";

    function hmac_keccak(
        bytes memory message
    ) internal pure returns (bytes32) {
        return keccak256(
            abi.encodePacked(
                HMAC_O, keccak256(abi.encodePacked(HMAC_I, message))
            )
        );
    }

    // Union whitepaper: (1) H_{hmac_r}
    function hashToField(
        bytes memory message
    ) internal pure returns (uint256) {
        return (uint256(hmac_keccak(message)) % PRIME_R_MINUS_ONE) + 1;
    }

    struct ZKP {
        uint256[8] proof;
        uint256[2] proofCommitment;
        uint256[2] proofCommitmentPOK;
    }

    function verifyZKP(
        bytes calldata zkpBytes,
        bytes31 chainId,
        bytes32 trustedValidatorsHash,
        SignedHeader calldata header
    ) public virtual returns (bool) {
        return
            internalVerifyZKP(zkpBytes, chainId, trustedValidatorsHash, header);
    }

    function internalVerifyZKP(
        bytes calldata zkpBytes,
        bytes31 chainId,
        bytes32 trustedValidatorsHash,
        SignedHeader calldata header
    ) internal virtual returns (bool) {
        ZKP calldata zkp;
        assembly {
            zkp := zkpBytes.offset
        }

        uint256 commitmentHash =
            hashToField(abi.encodePacked(zkp.proofCommitment));

        // Drop the most significant byte to fit in F_r
        bytes32 inputsHash = sha256(
            abi.encodePacked(
                bytes32(uint256(uint248(chainId))),
                bytes32(uint256(header.height)),
                bytes32(uint256(header.secs)),
                bytes32(uint256(header.nanos)),
                header.validatorsHash,
                header.nextValidatorsHash,
                header.appHash,
                trustedValidatorsHash
            )
        ) & 0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;

        uint256[2] memory publicInputs = [
            uint256(inputsHash),
            // Gnark commitment API extend internal inputs with the following commitment hash and proof commitment
            // See https://github.com/ConsenSys/gnark/issues/652
            commitmentHash
        ];

        return CometblsZKVerifier.verifyProof(
            zkp.proof, zkp.proofCommitment, zkp.proofCommitmentPOK, publicInputs
        );
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
            revert CometblsClientLib.ErrNotIBC();
        }
    }

    modifier onlyIBC() {
        _onlyIBC();
        _;
    }
}
