pragma solidity ^0.8.23;

import "@openzeppelin-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin-upgradeable/utils/PausableUpgradeable.sol";
import "solidity-bytes-utils/BytesLib.sol";

import "../core/02-client/ILightClient.sol";
import "../core/02-client/IBCHeight.sol";
import "../proto/ibc/core/client/v1/client.sol";
import "../proto/ibc/lightclients/tendermint/v1/tendermint.sol";
import "../proto/cosmos/ics23/v1/proofs.sol";
import "../proto/tendermint/types/types.sol";
import "../proto/tendermint/types/canonical.sol";
import "../proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";
import "../proto/ibc/lightclients/wasm/v1/wasm.sol";
import "../lib/CometblsHelp.sol";
import "../lib/ICS23.sol";
import "./ICS23MembershipVerifier.sol";
import "./Verifier.sol";

library CometblsClientLib {
    error ErrNotIBC();
    error ErrTrustedConsensusStateNotFound();
    error ErrUntrustedHeightLTETrustedHeight();
    error ErrUntrustedTimestampLTETrustedTimestamp();
    error ErrHeaderExpired();
    error ErrMaxClockDriftExceeded();
    error ErrInvalidZKP();
    error ErrDelayPeriodNotExpired();
    error ErrInvalidUntrustedValidatorsHash();
    error ErrInvalidMisbehaviorHeadersSequence();
    error ErrInvalidMisbehavior();
    error ErrClientFrozen();
}

contract CometblsClient is
    ILightClient,
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    PausableUpgradeable
{
    using BytesLib for bytes;
    using IBCHeight for IbcCoreClientV1Height.Data;
    using CometblsHelp for TendermintTypesHeader.Data;
    using CometblsHelp for TendermintTypesCommit.Data;
    using CometblsHelp for UnionIbcLightclientsCometblsV1ClientState.Data;
    using CometblsHelp for OptimizedConsensusState;
    using CometblsHelp for bytes;

    address private ibcHandler;

    mapping(string => UnionIbcLightclientsCometblsV1ClientState.Data) private
        clientStates;
    mapping(string => mapping(uint128 => OptimizedConsensusState)) private
        consensusStates;
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
        UnionIbcLightclientsCometblsV1ClientState.Data calldata clientState =
            clientStateBytes.unmarshalClientStateEthABI();
        OptimizedConsensusState calldata consensusState =
            consensusStateBytes.unmarshalConsensusStateEthABI();
        if (
            clientState.latest_height.revision_height == 0
                || consensusState.timestamp == 0
        ) {
            return (clientStateCommitment, update, false);
        }
        // ChainID can't exceed 31 bytes (ensures its big-endian repr fit in F_r)
        if (bytes(clientState.chain_id).length > 31) {
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
            clientState.marshalToCommitmentEthABI(),
            ConsensusStateUpdate({
                consensusStateCommitment: consensusState.marshalToCommitmentEthABI(),
                height: clientState.latest_height
            }),
            true
        );
    }

    function misbehavior(
        string calldata clientId,
        UnionIbcLightclientsCometblsV1Header.Data calldata headerA,
        UnionIbcLightclientsCometblsV1Header.Data calldata headerB
    ) public {
        UnionIbcLightclientsCometblsV1ClientState.Data storage clientState =
            clientStates[clientId];
        bool fraud = checkMisbehavior(clientId, clientState, headerA, headerB);
        if (!fraud) {
            revert CometblsClientLib.ErrInvalidMisbehavior();
        }
        // Similar to tendermint https://github.com/cosmos/ibc-go/blob/bbdcc8c6e965c8a2f607dfb2b61cd13712dd966a/modules/light-clients/07-tendermint/misbehaviour.go#L19
        clientState.frozen_height =
            IbcCoreClientV1Height.Data({revision_number: 0, revision_height: 1});
    }

    function checkMisbehavior(
        string calldata clientId,
        UnionIbcLightclientsCometblsV1ClientState.Data storage clientState,
        UnionIbcLightclientsCometblsV1Header.Data calldata headerA,
        UnionIbcLightclientsCometblsV1Header.Data calldata headerB
    ) internal returns (bool) {
        // Ensures that A >= B to simplify the misbehavior of time violation check
        if (!headerA.trusted_height.gte(headerB.trusted_height)) {
            revert CometblsClientLib.ErrInvalidMisbehaviorHeadersSequence();
        }

        OptimizedConsensusState storage consensusStateA =
            consensusStates[clientId][headerA.trusted_height.toUint128()];
        OptimizedConsensusState storage consensusStateB =
            consensusStates[clientId][headerB.trusted_height.toUint128()];

        // Check that the headers would have been accepted in an update
        (
            ,
            uint64 untrustedTimestampA,
            
        ) = verifyHeader(headerA, consensusStateA, clientState);
        (
            ,
            uint64 untrustedTimestampB,
            
        ) = verifyHeader(headerB, consensusStateB, clientState);

        if (headerA.trusted_height.eq(headerB.trusted_height)) {
            bytes32 hashA = keccak256(abi.encode(headerA.signed_header));
            bytes32 hashB = keccak256(abi.encode(headerB.signed_header));
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
        UnionIbcLightclientsCometblsV1Header.Data calldata header,
        OptimizedConsensusState storage consensusState,
        UnionIbcLightclientsCometblsV1ClientState.Data storage clientState
    ) internal returns (uint64, uint64, bytes32) {
        if (consensusState.timestamp == 0) {
            revert CometblsClientLib.ErrTrustedConsensusStateNotFound();
        }

        uint64 untrustedHeightNumber = uint64(header.signed_header.height);
        uint64 trustedHeightNumber = header.trusted_height.revision_height;
        if (untrustedHeightNumber <= trustedHeightNumber) {
            revert CometblsClientLib.ErrUntrustedHeightLTETrustedHeight();
        }

        uint64 trustedTimestamp = consensusState.timestamp;
        // Normalize to nanosecond because ibc-go recvPacket expects nanos...
        uint64 untrustedTimestamp = uint64(header.signed_header.time.secs) * 1e9
            + uint64(header.signed_header.time.nanos);
        if (untrustedTimestamp <= trustedTimestamp) {
            revert CometblsClientLib.ErrUntrustedTimestampLTETrustedTimestamp();
        }

        // Normalize to nanosecond because ibc-go recvPacket expects nanos...
        uint64 currentTime = uint64(block.timestamp * 1e9);
        if (
            CometblsHelp.isExpired(
                untrustedTimestamp, clientState.trusting_period, currentTime
            )
        ) {
            revert CometblsClientLib.ErrHeaderExpired();
        }

        uint64 maxClockDrift = currentTime + clientState.max_clock_drift;

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
            if (
                keccak256(header.signed_header.validators_hash)
                    != keccak256(abi.encodePacked(trustedValidatorsHash))
            ) {
                revert CometblsClientLib.ErrInvalidUntrustedValidatorsHash();
            }
        }

        bool ok = verifyZKP(
            header.zero_knowledge_proof,
            clientState.chain_id,
            trustedValidatorsHash,
            header.signed_header
        );
        if (!ok) {
            revert CometblsClientLib.ErrInvalidZKP();
        }

        return
            (untrustedHeightNumber, untrustedTimestamp, untrustedValidatorsHash);
    }

    function updateClient(
        string calldata clientId,
        bytes calldata clientMessageBytes
    )
        external
        override
        onlyIBC
        returns (bytes32, ConsensusStateUpdate[] memory)
    {
        UnionIbcLightclientsCometblsV1Header.Data calldata header =
            clientMessageBytes.unmarshalEthABI();
        UnionIbcLightclientsCometblsV1ClientState.Data storage clientState =
            clientStates[clientId];

        if (!clientState.frozen_height.isZero()) {
            revert CometblsClientLib.ErrClientFrozen();
        }

        OptimizedConsensusState storage consensusState =
            consensusStates[clientId][header.trusted_height.toUint128()];

        (
            uint64 untrustedHeightNumber,
            uint64 untrustedTimestamp,
            
        ) = verifyHeader(header, consensusState, clientState);

        // Update states
        if (untrustedHeightNumber > clientState.latest_height.revision_height) {
            clientState.latest_height.revision_height = untrustedHeightNumber;
        }

        IbcCoreClientV1Height.Data memory untrustedHeight =
        IbcCoreClientV1Height.Data({
            revision_number: header.trusted_height.revision_number,
            revision_height: untrustedHeightNumber
        });

        uint128 untrustedHeightIndex = untrustedHeight.toUint128();

        consensusState = consensusStates[clientId][untrustedHeightIndex];
        consensusState.timestamp = untrustedTimestamp;
        consensusState.appHash = header.signed_header.app_hash.toBytes32(0);
        consensusState.nextValidatorsHash =
            header.signed_header.next_validators_hash.toBytes32(0);

        ProcessedMoment storage processed =
            processedMoments[clientId][untrustedHeightIndex];
        processed.timestamp = block.timestamp * 1e9;
        processed.height = block.number;

        ConsensusStateUpdate[] memory updates = new ConsensusStateUpdate[](1);
        updates[0] = ConsensusStateUpdate({
            consensusStateCommitment: consensusState.marshalToCommitmentEthABI(),
            height: untrustedHeight
        });

        return (clientState.marshalToCommitmentEthABI(), updates);
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
        OptimizedConsensusState storage consensusState =
            consensusStates[clientId][height.toUint128()];
        if (consensusState.timestamp == 0) {
            revert CometblsClientLib.ErrTrustedConsensusStateNotFound();
        }
        ProcessedMoment storage moment =
            processedMoments[clientId][height.toUint128()];
        uint64 currentTime = uint64(block.timestamp * 1e9);
        uint64 validTime = uint64(moment.timestamp) + delayPeriodTime;
        if (delayPeriodTime != 0 && currentTime < validTime) {
            revert CometblsClientLib.ErrDelayPeriodNotExpired();
        }
        uint64 currentHeight = uint64(block.number);
        uint64 validHeight = uint64(moment.height) + delayPeriodBlocks;
        if (delayPeriodBlocks != 0 && currentHeight < validHeight) {
            revert CometblsClientLib.ErrDelayPeriodNotExpired();
        }
        return consensusState.appHash;
    }

    function getClientState(string calldata clientId)
        external
        view
        returns (bytes memory)
    {
        return clientStates[clientId].marshalEthABI();
    }

    function getConsensusState(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height
    ) external view returns (bytes memory) {
        return consensusStates[clientId][height.toUint128()].marshalEthABI();
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

    // ZKP VERIFICATION
    uint256 constant PRIME_R =
        21888242871839275222246405745257275088548364400416034343698204186575808495617;
    uint256 constant PRIME_R_MINUS_ONE = PRIME_R - 1;

    bytes constant HMAC_I =
        hex"75595B5342747A653636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636";
    bytes constant HMAC_O =
        hex"1F333139281E100F5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C";

    function hmac_keccak(bytes memory message)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(
            abi.encodePacked(HMAC_O, keccak256(HMAC_I.concat(message)))
        );
    }

    // Union whitepaper: (1) H_{hmac_r}
    function hashToField(bytes memory message)
        internal
        pure
        returns (uint256)
    {
        return (uint256(hmac_keccak(message)) % PRIME_R_MINUS_ONE) + 1;
    }

    struct ZKP {
        uint256[8] proof;
        uint256[2] proofCommitment;
        uint256[2] proofCommitmentPOK;
    }

    function verifyZKP(
        bytes calldata zkpBytes,
        string memory chainId,
        bytes32 trustedValidatorsHash,
        UnionIbcLightclientsCometblsV1LightHeader.Data memory header
    ) public virtual returns (bool) {
        ZKP calldata zkp;
        assembly {
            zkp := zkpBytes.offset
        }

        uint256 commitmentHash =
            hashToField(abi.encodePacked(zkp.proofCommitment));

        uint256 l = bytes(chainId).length;
        bytes memory paddedChainId = new bytes(32 - l).concat(bytes(chainId));

        // Drop the most significant byte to fit in F_r
        bytes32 inputsHash = sha256(
            abi.encodePacked(
                bytes32(paddedChainId),
                bytes32(uint256(int256(header.height))),
                bytes32(uint256(int256(header.time.secs))),
                bytes32(uint256(int256(header.time.nanos))),
                bytes32(header.validators_hash),
                bytes32(header.next_validators_hash),
                bytes32(header.app_hash),
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

    function _authorizeUpgrade(address newImplementation)
        internal
        override
        onlyOwner
    {}

    function _onlyIBC() private view {
        if (msg.sender != ibcHandler) {
            revert CometblsClientLib.ErrNotIBC();
        }
    }

    modifier onlyIBC() {
        _onlyIBC();
        _;
    }
}
