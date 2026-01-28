pragma solidity ^0.8.27;

library IBCCommitment {
    uint256 public constant CLIENT_STATE = 0x00;
    uint256 public constant CONSENSUS_STATE = 0x01;
    uint256 public constant CONNECTIONS = 0x02;
    uint256 public constant CHANNELS = 0x03;
    uint256 public constant PACKETS = 0x04;
    uint256 public constant PACKET_ACKS = 0x05;
    uint256 public constant MEMBERSHIP_PROOF = 0x06;
    uint256 public constant NON_MEMBERSHIP_PROOF = 0x07;
    uint256 public constant PACKET_TIMEOUTS = 0x08;

    bytes32 public constant NON_MEMBERSHIP_COMMITMENT_VALUE =
        0x0000000000000000000000000000000000000000000000000000000000000001;

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
        bytes32 batchHash
    ) internal pure returns (bytes memory) {
        return abi.encode(PACKETS, batchHash);
    }

    function batchReceiptsCommitmentPath(
        bytes32 batchHash
    ) internal pure returns (bytes memory) {
        return abi.encode(PACKET_ACKS, batchHash);
    }

    function membershipProofPath(
        uint32 clientId,
        uint64 proofHeight,
        bytes calldata path
    ) internal pure returns (bytes memory) {
        return abi.encodePacked(
            MEMBERSHIP_PROOF, uint256(clientId), uint256(proofHeight), path
        );
    }

    function nonMembershipProofPath(
        uint32 clientId,
        uint64 proofHeight,
        bytes calldata path
    ) internal pure returns (bytes memory) {
        return abi.encodePacked(
            NON_MEMBERSHIP_PROOF, uint256(clientId), uint256(proofHeight), path
        );
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
        bytes32 batchHash
    ) internal pure returns (bytes32) {
        return keccak256(batchPacketsCommitmentPath(batchHash));
    }

    function batchReceiptsCommitmentKey(
        bytes32 batchHash
    ) internal pure returns (bytes32) {
        return keccak256(batchReceiptsCommitmentPath(batchHash));
    }

    function membershipProofCommitmentKey(
        uint32 clientId,
        uint64 proofHeight,
        bytes calldata path
    ) internal pure returns (bytes32) {
        return keccak256(membershipProofPath(clientId, proofHeight, path));
    }

    function nonMembershipProofCommitmentKey(
        uint32 clientId,
        uint64 proofHeight,
        bytes calldata path
    ) internal pure returns (bytes32) {
        return keccak256(nonMembershipProofPath(clientId, proofHeight, path));
    }
}
