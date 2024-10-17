pragma solidity ^0.8.27;

library IBCCommitment {
    uint256 public constant CLIENT_STATE = 0x00;
    uint256 public constant CONSENSUS_STATE = 0x01;
    uint256 public constant CONNECTIONS = 0x02;
    uint256 public constant CHANNELS = 0x03;
    uint256 public constant PACKETS = 0x04;
    uint256 public constant PACKET_ACKS = 0x05;
    uint256 public constant NEXT_SEQ_SEND = 0x06;
    uint256 public constant NEXT_SEQ_RECV = 0x07;
    uint256 public constant NEXT_SEQ_ACK = 0x08;

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

    function packetCommitmentPath(
        uint32 channelId,
        uint64 sequence
    ) internal pure returns (bytes memory) {
        return abi.encode(PACKETS, channelId, sequence);
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

    function nextSequenceSendCommitmentPath(
        uint32 channelId
    ) internal pure returns (bytes memory) {
        return abi.encode(NEXT_SEQ_SEND, channelId);
    }

    function nextSequenceRecvCommitmentPath(
        uint32 channelId
    ) internal pure returns (bytes memory) {
        return abi.encode(NEXT_SEQ_RECV, channelId);
    }

    function nextSequenceAckCommitmentPath(
        uint32 channelId
    ) internal pure returns (bytes memory) {
        return abi.encode(NEXT_SEQ_ACK, channelId);
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

    function packetCommitmentKey(
        uint32 channelId,
        uint64 sequence
    ) internal pure returns (bytes32) {
        return keccak256(packetCommitmentPath(channelId, sequence));
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

    function nextSequenceSendCommitmentKey(
        uint32 channelId
    ) internal pure returns (bytes32) {
        return keccak256(nextSequenceSendCommitmentPath(channelId));
    }

    function nextSequenceRecvCommitmentKey(
        uint32 channelId
    ) internal pure returns (bytes32) {
        return keccak256(nextSequenceRecvCommitmentPath(channelId));
    }

    function nextSequenceAckCommitmentKey(
        uint32 channelId
    ) internal pure returns (bytes32) {
        return keccak256(nextSequenceAckCommitmentPath(channelId));
    }
}
