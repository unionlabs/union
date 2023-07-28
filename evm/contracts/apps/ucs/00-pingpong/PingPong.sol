pragma solidity ^0.8.18;

import "../../Base.sol";
import "../../../core/25-handler/IBCHandler.sol";

contract PingPong is IBCAppBase {
    IBCHandler private ibcHandler;

    constructor(IBCHandler _ibcHandler) {
        ibcHandler = _ibcHandler;
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }

    function initiate(
        string calldata sourcePort,
        string calldata sourceChannel,
        bool ping
    ) public {
        ibcHandler.sendPacket(
            sourcePort,
            sourceChannel,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: type(uint64).max
            }),
            0,
            abi.encode(ping)
        );
    }

    function onRecvPacket(
        IbcCoreChannelV1Packet.Data calldata packet,
        address relayer
    ) external virtual override onlyIBC returns (bytes memory acknowledgement) {
        bool ping = abi.decode(packet.data, (bool));
        initiate(packet.destination_port, packet.destination_channel, !ping);
        return hex"01";
    }

    function onAcknowledgementPacket(
        IbcCoreChannelV1Packet.Data calldata packet,
        bytes calldata acknowledgement,
        address relayer
    ) external virtual override onlyIBC {}

    function onChanOpenInit(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata,
        string calldata,
        string calldata channelId,
        IbcCoreChannelV1Counterparty.Data calldata,
        string calldata
    ) external virtual override onlyIBC {}

    function onChanOpenTry(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata,
        string calldata,
        string calldata channelId,
        IbcCoreChannelV1Counterparty.Data calldata,
        string calldata,
        string calldata
    ) external virtual override onlyIBC {}

    function onChanOpenAck(
        string calldata portId,
        string calldata channelId,
        string calldata counterpartyVersion
    ) external virtual override onlyIBC {}

    function onChanOpenConfirm(
        string calldata portId,
        string calldata channelId
    ) external virtual override onlyIBC {}

    function onChanCloseInit(
        string calldata portId,
        string calldata channelId
    ) external virtual override onlyIBC {}

    function onChanCloseConfirm(
        string calldata portId,
        string calldata channelId
    ) external virtual override onlyIBC {}
}
