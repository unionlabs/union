pragma solidity ^0.8.23;

import "solady/utils/LibString.sol";

library IBCCommitment {
    // Commitment path generators that comply with https://github.com/cosmos/ibc/tree/main/spec/core/ics-024-host-requirements#path-space

    function clientStatePath(string memory clientId)
        internal
        pure
        returns (bytes memory)
    {
        return abi.encodePacked("clients/", clientId, "/clientState");
    }

    function consensusStatePath(
        string memory clientId,
        uint64 revisionNumber,
        uint64 revisionHeight
    ) internal pure returns (bytes memory) {
        return abi.encodePacked(
            "clients/",
            clientId,
            "/consensusStates/",
            LibString.toString(revisionNumber),
            "-",
            LibString.toString(revisionHeight)
        );
    }

    function connectionPath(string memory connectionId)
        internal
        pure
        returns (bytes memory)
    {
        return abi.encodePacked("connections/", connectionId);
    }

    function channelPath(
        string memory portId,
        string memory channelId
    ) internal pure returns (bytes memory) {
        return abi.encodePacked(
            "channelEnds/ports/", portId, "/channels/", channelId
        );
    }

    function packetCommitmentPath(
        string memory portId,
        string memory channelId,
        uint64 sequence
    ) internal pure returns (bytes memory) {
        return abi.encodePacked(
            "commitments/ports/",
            portId,
            "/channels/",
            channelId,
            "/sequences/",
            LibString.toString(sequence)
        );
    }

    function packetAcknowledgementCommitmentPath(
        string memory portId,
        string memory channelId,
        uint64 sequence
    ) internal pure returns (bytes memory) {
        return abi.encodePacked(
            "acks/ports/",
            portId,
            "/channels/",
            channelId,
            "/sequences/",
            LibString.toString(sequence)
        );
    }

    function packetReceiptCommitmentPath(
        string memory portId,
        string memory channelId,
        uint64 sequence
    ) internal pure returns (bytes memory) {
        return abi.encodePacked(
            "receipts/ports/",
            portId,
            "/channels/",
            channelId,
            "/sequences/",
            LibString.toString(sequence)
        );
    }

    function nextSequenceSendCommitmentPath(
        string memory portId,
        string memory channelId
    ) internal pure returns (bytes memory) {
        return abi.encodePacked(
            "nextSequenceSend/ports/", portId, "/channels/", channelId
        );
    }

    function nextSequenceRecvCommitmentPath(
        string memory portId,
        string memory channelId
    ) internal pure returns (bytes memory) {
        return abi.encodePacked(
            "nextSequenceRecv/ports/", portId, "/channels/", channelId
        );
    }

    function nextSequenceAckCommitmentPath(
        string memory portId,
        string memory channelId
    ) internal pure returns (bytes memory) {
        return abi.encodePacked(
            "nextSequenceAck/ports/", portId, "/channels/", channelId
        );
    }

    // Key generators for Commitment mapping

    function clientStateCommitmentKey(string memory clientId)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(clientStatePath(clientId));
    }

    function consensusStateCommitmentKey(
        string memory clientId,
        uint64 revisionNumber,
        uint64 revisionHeight
    ) internal pure returns (bytes32) {
        return keccak256(
            consensusStatePath(clientId, revisionNumber, revisionHeight)
        );
    }

    function connectionCommitmentKey(string memory connectionId)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(connectionPath(connectionId));
    }

    function channelCommitmentKey(
        string memory portId,
        string memory channelId
    ) internal pure returns (bytes32) {
        return keccak256(channelPath(portId, channelId));
    }

    function packetCommitmentKey(
        string memory portId,
        string memory channelId,
        uint64 sequence
    ) internal pure returns (bytes32) {
        return keccak256(packetCommitmentPath(portId, channelId, sequence));
    }

    function packetAcknowledgementCommitmentKey(
        string memory portId,
        string memory channelId,
        uint64 sequence
    ) internal pure returns (bytes32) {
        return keccak256(
            packetAcknowledgementCommitmentPath(portId, channelId, sequence)
        );
    }

    function packetReceiptCommitmentKey(
        string memory portId,
        string memory channelId,
        uint64 sequence
    ) internal pure returns (bytes32) {
        return
            keccak256(packetReceiptCommitmentPath(portId, channelId, sequence));
    }

    function nextSequenceSendCommitmentKey(
        string memory portId,
        string memory channelId
    ) internal pure returns (bytes32) {
        return keccak256(nextSequenceSendCommitmentPath(portId, channelId));
    }

    function nextSequenceRecvCommitmentKey(
        string memory portId,
        string memory channelId
    ) internal pure returns (bytes32) {
        return keccak256(nextSequenceRecvCommitmentPath(portId, channelId));
    }

    function nextSequenceAckCommitmentKey(
        string memory portId,
        string memory channelId
    ) internal pure returns (bytes32) {
        return keccak256(nextSequenceAckCommitmentPath(portId, channelId));
    }
}
