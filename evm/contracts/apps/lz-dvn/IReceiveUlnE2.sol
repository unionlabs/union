pragma solidity ^0.8.27;

interface IReceiveUlnE2 {
    function verify(
        bytes calldata _packetHeader,
        bytes32 _payloadHash,
        uint64 _confirmations
    ) external;

    function commitVerification(
        bytes calldata _packetHeader,
        bytes32 _payloadHash
    ) external;
}
