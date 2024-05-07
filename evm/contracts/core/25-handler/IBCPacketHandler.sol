pragma solidity ^0.8.23;

import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCHost.sol";
import "../04-channel/IIBCPacket.sol";
import "../05-port/ModuleManager.sol";

/**
 * @dev IBCPacketHandler is a contract that calls a contract that implements `IIBCPacket` with delegatecall.
 */
abstract contract IBCPacketHandler is IIBCPacket, ModuleManager {
    // IBC Packet contract address
    address ibcPacket;

    function sendPacket(
        string calldata sourceChannel,
        IbcCoreClientV1Height.Data calldata timeoutHeight,
        uint64 timeoutTimestamp,
        bytes calldata data
    ) external virtual override returns (uint64) {
        passthrough(ibcPacket);
    }

    function recvPacket(IBCMsgs.MsgPacketRecv calldata msg_)
        external
        override
    {
        passthrough(ibcPacket);
    }

    function writeAcknowledgement(
        string calldata destinationChannel,
        uint64 sequence,
        bytes calldata acknowledgement
    ) external override {
        passthrough(ibcPacket);
    }

    function acknowledgePacket(IBCMsgs.MsgPacketAcknowledgement calldata msg_)
        external
        override
    {
        passthrough(ibcPacket);
    }

    function timeoutPacket(IBCMsgs.MsgPacketTimeout calldata msg_)
        external
        override
    {
        passthrough(ibcPacket);
    }
}
