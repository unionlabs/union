pragma solidity ^0.8.27;

import "../Types.sol";

struct ConsensusStateUpdate {
    bytes32 clientStateCommitment;
    bytes32 consensusStateCommitment;
    uint64 height;
}

/**
 * @dev This defines an interface for Light Client contract can be integrated with ibc-solidity.
 * You can register the Light Client contract that implements this through `registerClient` on IBCHandler.
 */
interface ILightClient {
    /**
     * @dev createClient creates a new client with the given state.
     * If succeeded, it returns a commitment for the initial state.
     */
    function createClient(
        uint32 clientId,
        bytes calldata clientStateBytes,
        bytes calldata consensusStateBytes
    ) external returns (ConsensusStateUpdate memory update);

    /**
     * @dev getTimestampAtHeight returns the timestamp of the consensus state at the given height.
     */
    function getTimestampAtHeight(
        uint32 clientId,
        uint64 height
    ) external view returns (uint64);

    /**
     * @dev getLatestHeight returns the latest height of the client state corresponding to `clientId`.
     */
    function getLatestHeight(
        uint32 clientId
    ) external view returns (uint64 height);

    /**
     * @dev updateClient updates the client corresponding to `clientId`.
     * If succeeded, it returns a commitment for the updated state.
     * If there are no updates for consensus state, this function should returns an empty array as `updates`.
     *
     * NOTE: updateClient is intended to perform the followings:
     * 1. verify a given client message(e.g. header)
     * 2. check misbehaviour such like duplicate block height
     * 3. if misbehaviour is found, update state accordingly and return
     * 4. update state(s) with the client message
     * 5. persist the state(s) on the host
     */
    function updateClient(
        uint32 clientId,
        bytes calldata clientMessageBytes
    ) external returns (ConsensusStateUpdate memory update);

    /**
     * @dev verifyMembership is a generic proof verification method which verifies a proof of the existence of a value at a given CommitmentPath at the specified height.
     * The caller is expected to construct the full CommitmentPath from a CommitmentPrefix and a standardized path (as defined in ICS 24).
     */
    function verifyMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path,
        bytes calldata value
    ) external returns (bool);

    /**
     * @dev verifyNonMembership is a generic proof verification method which verifies the absence of a given CommitmentPath at a specified height.
     * The caller is expected to construct the full CommitmentPath from a CommitmentPrefix and a standardized path (as defined in ICS 24).
     */
    function verifyNonMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path
    ) external returns (bool);

    /**
     * @dev getClientState returns the clientState corresponding to `clientId`.
     */
    function getClientState(
        uint32 clientId
    ) external view returns (bytes memory);

    /**
     * @dev getConsensusState returns the consensusState corresponding to `clientId` and `height`.
     */
    function getConsensusState(
        uint32 clientId,
        uint64 height
    ) external view returns (bytes memory);

    /**
     * @dev isFrozen returns whether the `clientId` is frozen or not.
     */
    function isFrozen(
        uint32 clientId
    ) external view returns (bool);
}
