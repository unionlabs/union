pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin-upgradeable/utils/PausableUpgradeable.sol";

import "./ICS23MembershipVerifier.sol";
import "./Verifier.sol";

import "../core/02-client/ILightClient.sol";
import "../core/24-host/IBCStore.sol";
import "../core/24-host/IBCCommitment.sol";
import "../lib/Common.sol";
import "../lib/ICS23.sol";

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
}

struct ConsensusState {
    uint64 timestamp;
    bytes32 appHash;
    bytes32 nextValidatorsHash;
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
    error ErrInvalidMisbehaviorHeadersSequence();
    error ErrInvalidMisbehavior();
    error ErrClientFrozen();
    error ErrInvalidInitialConsensusState();

    function isExpired(
        uint64 headerTime,
        uint64 trustingPeriod,
        uint64 currentTime
    ) internal pure returns (bool) {
        return currentTime > (headerTime + trustingPeriod);
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

    function encodeMemory(
        ClientState memory clientState
    ) internal pure returns (bytes memory) {
        return abi.encode(
            clientState.chainId,
            clientState.trustingPeriod,
            clientState.maxClockDrift,
            clientState.frozenHeight,
            clientState.latestHeight
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
}

contract CometblsClient is
    ILightClient,
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    PausableUpgradeable
{
    using CometblsClientLib for *;

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
        ClientState calldata clientState = clientStateBytes.decodeClientState();
        ConsensusState calldata consensusState =
            consensusStateBytes.decodeConsensusState();
        if (clientState.latestHeight == 0 || consensusState.timestamp == 0) {
            revert CometblsClientLib.ErrInvalidInitialConsensusState();
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

    function misbehavior(
        uint32 clientId,
        Header calldata headerA,
        Header calldata headerB
    ) public {
        ClientState storage clientState = clientStates[clientId];
        bool fraud = checkMisbehavior(clientId, clientState, headerA, headerB);
        if (!fraud) {
            revert CometblsClientLib.ErrInvalidMisbehavior();
        }
        // Similar to tendermint https://github.com/cosmos/ibc-go/blob/bbdcc8c6e965c8a2f607dfb2b61cd13712dd966a/modules/light-clients/07-tendermint/misbehaviour.go#L19
        clientState.frozenHeight = 1;
    }

    function checkMisbehavior(
        uint32 clientId,
        ClientState storage clientState,
        Header calldata headerA,
        Header calldata headerB
    ) internal returns (bool) {
        // Ensures that A > B to simplify the misbehavior of time violation check
        if (headerA.signedHeader.height < headerB.signedHeader.height) {
            revert CometblsClientLib.ErrInvalidMisbehaviorHeadersSequence();
        }

        ConsensusState storage consensusStateA =
            consensusStates[clientId][headerA.trustedHeight];
        ConsensusState storage consensusStateB =
            consensusStates[clientId][headerB.trustedHeight];

        // Check that the headers would have been accepted in an update
        (, uint64 untrustedTimestampA,) =
            verifyHeader(headerA, consensusStateA, clientState);
        (, uint64 untrustedTimestampB,) =
            verifyHeader(headerB, consensusStateB, clientState);

        if (headerA.signedHeader.height == headerB.signedHeader.height) {
            bytes32 hashA = keccak256(abi.encode(headerA.signedHeader));
            bytes32 hashB = keccak256(abi.encode(headerB.signedHeader));
            if (hashA != hashB) {
                // Misbehavior of a fork
                return true;
            }
        } else {
            // Guarantee that A > B
            if (untrustedTimestampA <= untrustedTimestampB) {
                // Misbehavior of time violation
                return true;
            }
        }
        return false;
    }

    function verifyHeader(
        Header calldata header,
        ConsensusState storage consensusState,
        ClientState storage clientState
    ) internal returns (uint64, uint64, bytes32) {
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
                untrustedTimestamp, clientState.trustingPeriod, currentTime
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
        bytes32 untrustedValidatorsHash;
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

        return
            (untrustedHeightNumber, untrustedTimestamp, untrustedValidatorsHash);
    }

    function updateClient(
        uint32 clientId,
        bytes calldata clientMessageBytes
    ) external override onlyIBC returns (ConsensusStateUpdate memory) {
        ClientState storage clientState = clientStates[clientId];

        if (clientState.frozenHeight > 0) {
            revert CometblsClientLib.ErrClientFrozen();
        }

        Header calldata header = clientMessageBytes.decodeHeader();

        ConsensusState storage consensusState =
            consensusStates[clientId][header.trustedHeight];

        (uint64 untrustedHeightNumber, uint64 untrustedTimestamp,) =
            verifyHeader(header, consensusState, clientState);

        // Update states
        if (untrustedHeightNumber > clientState.latestHeight) {
            clientState.latestHeight = untrustedHeightNumber;
        }

        consensusState = consensusStates[clientId][header.trustedHeight];
        consensusState.timestamp = untrustedTimestamp;
        consensusState.appHash = header.signedHeader.appHash;
        consensusState.nextValidatorsHash =
            header.signedHeader.nextValidatorsHash;

        ProcessedMoment storage processed =
            processedMoments[clientId][header.trustedHeight];
        processed.timestamp = block.timestamp * 1e9;
        processed.height = block.number;

        return ConsensusStateUpdate({
            clientStateCommitment: clientState.commit(),
            consensusStateCommitment: consensusState.commit(),
            height: header.trustedHeight
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
            revert CometblsClientLib.ErrClientFrozen();
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
            revert CometblsClientLib.ErrClientFrozen();
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
    ) external view virtual returns (bool) {
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
                bytes32(chainId),
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

        return Verifier.verifyProof(
            zkp.proof, zkp.proofCommitment, zkp.proofCommitmentPOK, publicInputs
        );
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}

    function _onlyIBC() internal view {
        if (msg.sender != ibcHandler) {
            revert CometblsClientLib.ErrNotIBC();
        }
    }

    modifier onlyIBC() {
        _onlyIBC();
        _;
    }
}
