pragma solidity ^0.8.27;

library IBCCommitment {
    uint256 public constant CLIENT_STATE = 0x00;
    uint256 public constant CONSENSUS_STATE = 0x01;
    uint256 public constant CONNECTIONS = 0x02;
    uint256 public constant CHANNELS = 0x03;
    uint256 public constant PACKETS = 0x04;
    uint256 public constant PACKET_ACKS = 0x05;

    function clientStatePath(
        uint32 clientId
    ) internal pure returns (bytes memory) {
        return abi.encode(CLIENT_STATE, clientId);
    }

    function consensusStatePath(
        uint32 clientId,
        uint64 height
    ) internal pure returns (bytes memory) {
        return abi.encode(CONSENSUS_STATE, clientId, height);
    }

    function connectionPath(
        uint32 connectionId
    ) internal pure returns (bytes memory) {
        return abi.encode(CONNECTIONS, connectionId);
    }

    function channelPath(
        uint32 channelId
    ) internal pure returns (bytes memory) {
        return abi.encode(CHANNELS, channelId);
    }

    function batchPacketsCommitmentPath(
        uint32 channelId,
        bytes32 batchHash
    ) internal pure returns (bytes memory) {
        return abi.encode(PACKETS, channelId, batchHash);
    }

    function batchReceiptsCommitmentPath(
        uint32 channelId,
        bytes32 batchHash
    ) internal pure returns (bytes memory) {
        return abi.encode(PACKET_ACKS, channelId, batchHash);
    }

    // Key generators for Commitment mapping

    function clientStateCommitmentKey(
        uint32 clientId
    ) internal pure returns (bytes32) {
        return keccak256(clientStatePath(clientId));
    }

    function consensusStateCommitmentKey(
        uint32 clientId,
        uint64 height
    ) internal pure returns (bytes32) {
        return keccak256(consensusStatePath(clientId, height));
    }

    function connectionCommitmentKey(
        uint32 connectionId
    ) internal pure returns (bytes32) {
        return keccak256(connectionPath(connectionId));
    }

    function channelCommitmentKey(
        uint32 channelId
    ) internal pure returns (bytes32) {
        return keccak256(channelPath(channelId));
    }

    function batchPacketsCommitmentKey(
        uint32 channelId,
        bytes32 batchHash
    ) internal pure returns (bytes32) {
        return keccak256(batchPacketsCommitmentPath(channelId, batchHash));
    }

    function batchReceiptsCommitmentKey(
        uint32 channelId,
        bytes32 batchHash
    ) internal pure returns (bytes32) {
        return keccak256(batchReceiptsCommitmentPath(channelId, batchHash));
    }
}
