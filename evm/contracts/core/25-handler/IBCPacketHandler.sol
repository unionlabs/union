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
        string calldata,
        IbcCoreClientV1Height.Data calldata,
        uint64,
        bytes calldata
    ) external virtual override returns (uint64) {
        passthrough(ibcPacket);
    }

    function recvPacket(IBCMsgs.MsgPacketRecv calldata) external override {
        passthrough(ibcPacket);
    }

    function writeAcknowledgement(
        IbcCoreChannelV1Packet.Data calldata,
        bytes memory
    ) external override {
        passthrough(ibcPacket);
    }

    function acknowledgePacket(IBCMsgs.MsgPacketAcknowledgement calldata)
        external
        override
    {
        passthrough(ibcPacket);
    }

    function timeoutPacket(IBCMsgs.MsgPacketTimeout calldata)
        external
        override
    {
        passthrough(ibcPacket);
    }
}
