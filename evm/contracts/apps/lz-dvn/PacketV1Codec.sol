pragma solidity ^0.8.27;

/// @notice Decoder for LayerZero V2 packet headers.
/// Header layout (81 bytes):
///   [0]   uint8   version (must be 1)
///   [1]   uint64  nonce
///   [9]   uint32  srcEid
///   [13]  bytes32 sender
///   [45]  uint32  dstEid
///   [49]  bytes32 receiver
library PacketV1Codec {
    uint8 internal constant PACKET_VERSION = 1;
    uint256 internal constant PACKET_HEADER_SIZE = 81;

    error ErrInvalidHeaderLength();
    error ErrInvalidHeaderVersion();

    function assertValid(
        bytes memory header
    ) internal pure {
        if (header.length != PACKET_HEADER_SIZE) {
            revert ErrInvalidHeaderLength();
        }
        if (uint8(header[0]) != PACKET_VERSION) {
            revert ErrInvalidHeaderVersion();
        }
    }

    function srcEid(
        bytes memory header
    ) internal pure returns (uint32 out) {
        // data starts at header + 32; srcEid is at byte offset 9 within the data.
        assembly {
            out := shr(224, mload(add(header, 41)))
        }
    }

    function dstEid(
        bytes memory header
    ) internal pure returns (uint32 out) {
        // data + 45
        assembly {
            out := shr(224, mload(add(header, 77)))
        }
    }

    function sender(
        bytes memory header
    ) internal pure returns (bytes32 out) {
        // data + 13
        assembly {
            out := mload(add(header, 45))
        }
    }

    function receiverB32(
        bytes memory header
    ) internal pure returns (bytes32 out) {
        // data + 49
        assembly {
            out := mload(add(header, 81))
        }
    }

    function receiverAddress(
        bytes memory header
    ) internal pure returns (address) {
        return address(uint160(uint256(receiverB32(header))));
    }
}
