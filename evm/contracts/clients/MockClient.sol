pragma solidity ^0.8.23;

import "../core/02-client/ILightClient.sol";
import "../core/02-client/IBCHeight.sol";
import "../proto/ibc/core/client/v1/client.sol";
import {
    IbcLightclientsMockV1ClientState as ClientState,
    IbcLightclientsMockV1ConsensusState as ConsensusState,
    IbcLightclientsMockV1Header as Header
} from "../proto/MockClient.sol";
import {GoogleProtobufAny as Any} from "../proto/GoogleProtobufAny.sol";
import "solidity-bytes-utils/BytesLib.sol";

// MockClient implements https://github.com/datachainlab/ibc-mock-client
// WARNING: This client is intended to be used for testing purpose. Therefore, it is not generally available in a production, except in a fully trusted environment.
contract MockClient is ILightClient {
    using BytesLib for bytes;
    using IBCHeight for IbcCoreClientV1Height.Data;

    string private constant HEADER_TYPE_URL = "/ibc.lightclients.mock.v1.Header";
    string private constant CLIENT_STATE_TYPE_URL =
        "/ibc.lightclients.mock.v1.ClientState";
    string private constant CONSENSUS_STATE_TYPE_URL =
        "/ibc.lightclients.mock.v1.ConsensusState";

    bytes32 private constant HEADER_TYPE_URL_HASH =
        keccak256(abi.encodePacked(HEADER_TYPE_URL));
    bytes32 private constant CLIENT_STATE_TYPE_URL_HASH =
        keccak256(abi.encodePacked(CLIENT_STATE_TYPE_URL));
    bytes32 private constant CONSENSUS_STATE_TYPE_URL_HASH =
        keccak256(abi.encodePacked(CONSENSUS_STATE_TYPE_URL));

    address internal ibcHandler;
    mapping(string => ClientState.Data) internal clientStates;
    mapping(string => mapping(uint128 => ConsensusState.Data)) internal
        consensusStates;

    constructor(address ibcHandler_) {
        ibcHandler = ibcHandler_;
    }

    /**
     * @dev createClient creates a new client with the given state
     */
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
        ClientState.Data memory clientState;
        ConsensusState.Data memory consensusState;

        (clientState, ok) = unmarshalClientState(clientStateBytes);
        if (!ok) {
            return (clientStateCommitment, update, false);
        }
        (consensusState, ok) = unmarshalConsensusState(consensusStateBytes);
        if (!ok) {
            return (clientStateCommitment, update, false);
        }
        if (
            clientState.latest_height.revision_number != 0
                || clientState.latest_height.revision_height == 0
                || consensusState.timestamp == 0
        ) {
            return (clientStateCommitment, update, false);
        }
        clientStates[clientId] = clientState;
        consensusStates[clientId][clientState.latest_height.toUint128()] =
            consensusState;
        return (
            keccak256(clientStateBytes),
            ConsensusStateUpdate({
                consensusStateCommitment: keccak256(consensusStateBytes),
                height: clientState.latest_height
            }),
            true
        );
    }

    /**
     * @dev getTimestampAtHeight returns the timestamp of the consensus state at the given height.
     */
    function getTimestampAtHeight(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height
    ) external view override returns (uint64) {
        return consensusStates[clientId][height.toUint128()].timestamp;
    }

    /**
     * @dev getLatestHeight returns the latest height of the client state corresponding to `clientId`.
     */
    function getLatestHeight(string calldata clientId)
        external
        view
        override
        returns (IbcCoreClientV1Height.Data memory)
    {
        return clientStates[clientId].latest_height;
    }

    /**
     * @dev updateClient is intended to perform the followings:
     * 1. verify a given client message(e.g. header)
     * 2. check misbehaviour such like duplicate block height
     * 3. if misbehaviour is found, update state accordingly and return
     * 4. update state(s) with the client message
     * 5. persist the state(s) on the host
     */
    function updateClient(
        string calldata clientId,
        bytes calldata clientMessageBytes
    )
        external
        override
        onlyIBC
        returns (
            bytes32 clientStateCommitment,
            ConsensusStateUpdate[] memory updates
        )
    {
        IbcCoreClientV1Height.Data memory height;
        uint64 timestamp;
        Any.Data memory anyClientState;
        Any.Data memory anyConsensusState;

        (height, timestamp) = parseHeader(clientMessageBytes);
        if (height.gt(clientStates[clientId].latest_height)) {
            clientStates[clientId].latest_height = height;
        }
        anyClientState.type_url = CLIENT_STATE_TYPE_URL;
        anyClientState.value = ClientState.encode(clientStates[clientId]);

        ConsensusState.Data storage consensusState =
            consensusStates[clientId][height.toUint128()];
        consensusState.timestamp = timestamp;

        anyConsensusState.type_url = CONSENSUS_STATE_TYPE_URL;
        anyConsensusState.value = ConsensusState.encode(consensusState);

        updates = new ConsensusStateUpdate[](1);
        updates[0] = ConsensusStateUpdate({
            consensusStateCommitment: keccak256(Any.encode(anyConsensusState)),
            height: height
        });
        return (keccak256(Any.encode(anyClientState)), updates);
    }

    /**
     * @dev verifyMembership is a generic proof verification method which verifies a proof of the existence of a value at a given CommitmentPath at the specified height.
     * The caller is expected to construct the full CommitmentPath from a CommitmentPrefix and a standardized path (as defined in ICS 24).
     */
    function verifyMembership(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height,
        uint64,
        uint64,
        bytes calldata proof,
        bytes memory,
        bytes memory,
        bytes calldata value
    ) external view override returns (bool) {
        require(
            consensusStates[clientId][height.toUint128()].timestamp != 0,
            "consensus state not found"
        );
        return sha256(value) == proof.toBytes32(0);
    }

    /**
     * @dev verifyNonMembership is a generic proof verification method which verifies the absence of a given CommitmentPath at a specified height.
     * The caller is expected to construct the full CommitmentPath from a CommitmentPrefix and a standardized path (as defined in ICS 24).
     */
    function verifyNonMembership(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height,
        uint64,
        uint64,
        bytes calldata proof,
        bytes memory,
        bytes memory
    ) external view returns (bool) {
        require(
            consensusStates[clientId][height.toUint128()].timestamp != 0,
            "consensus state not found"
        );
        return proof.length == 0;
    }

    /* State accessors */

    /**
     * @dev getClientState returns the clientState corresponding to `clientId`.
     */
    function getClientState(string calldata clientId)
        external
        view
        returns (bytes memory clientStateBytes)
    {
        return Any.encode(
            Any.Data({
                type_url: CLIENT_STATE_TYPE_URL,
                value: ClientState.encode(clientStates[clientId])
            })
        );
    }

    /**
     * @dev getConsensusState returns the consensusState corresponding to `clientId` and `height`.
     *      If it's not found, the function returns false.
     */
    function getConsensusState(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height
    ) external view returns (bytes memory consensusStateBytes) {
        return Any.encode(
            Any.Data({
                type_url: CONSENSUS_STATE_TYPE_URL,
                value: ConsensusState.encode(
                    consensusStates[clientId][height.toUint128()]
                    )
            })
        );
    }

    function isFrozen(string calldata) external view virtual returns (bool) {
        return false;
    }

    /* Internal functions */

    function parseHeader(bytes memory bz)
        internal
        pure
        returns (IbcCoreClientV1Height.Data memory, uint64)
    {
        Any.Data memory any = Any.decode(bz);
        require(
            keccak256(abi.encodePacked(any.type_url)) == HEADER_TYPE_URL_HASH,
            "invalid header type"
        );
        Header.Data memory header = Header.decode(any.value);
        require(
            header.height.revision_number == 0
                && header.height.revision_height != 0 && header.timestamp != 0,
            "invalid header"
        );
        return (header.height, header.timestamp);
    }

    function unmarshalClientState(bytes calldata bz)
        internal
        pure
        returns (ClientState.Data memory clientState, bool ok)
    {
        Any.Data memory anyClientState = Any.decode(bz);
        if (
            keccak256(abi.encodePacked(anyClientState.type_url))
                != CLIENT_STATE_TYPE_URL_HASH
        ) {
            return (clientState, false);
        }
        return (ClientState.decode(anyClientState.value), true);
    }

    function unmarshalConsensusState(bytes calldata bz)
        internal
        pure
        returns (ConsensusState.Data memory consensusState, bool ok)
    {
        Any.Data memory anyConsensusState = Any.decode(bz);
        if (
            keccak256(abi.encodePacked(anyConsensusState.type_url))
                != CONSENSUS_STATE_TYPE_URL_HASH
        ) {
            return (consensusState, false);
        }
        return (ConsensusState.decode(anyConsensusState.value), true);
    }

    modifier onlyIBC() {
        require(msg.sender == ibcHandler);
        _;
    }
}
