pragma solidity ^0.8.23;

import {
    IbcCoreChannelV1Packet as Packet,
    IbcCoreChannelV1GlobalEnums as ChannelEnums,
    IbcCoreChannelV1Counterparty as ChannelCounterparty
} from "../../../contracts/proto/ibc/core/channel/v1/channel.sol";
import {IIBCModule} from "../../../contracts/core/05-port/IIBCModule.sol";
import {IBCHandler} from "../../../contracts/core/25-handler/IBCHandler.sol";
import {IBCHost} from "../../../contracts/core/24-host/IBCHost.sol";
import {Context} from "@openzeppelin/utils/Context.sol";

contract MockApp is IIBCModule {
    event MockPacketRecv();
    event MockPacketAck();
    event MockPacketTimeout();
    event MockChannelOpenInit(string portId, string channelId);
    event MockChannelOpenTry();
    event MockChannelOpenAck();
    event MockChannelOpenConfirm();
    event MockChannelCloseInit();
    event MockChannelCloseConfirm();

    function onRecvPacket(
        Packet.Data calldata,
        address
    ) external virtual override returns (bytes memory) {
        emit MockPacketRecv();
        return bytes("1");
    }

    function onAcknowledgementPacket(
        Packet.Data calldata packet,
        bytes calldata acknowledgement,
        address relayer
    ) external virtual override {
        emit MockPacketAck();
    }

    function onTimeoutPacket(
        Packet.Data calldata packet,
        address relayer
    ) external virtual override {
        emit MockPacketTimeout();
    }

    function onChanOpenInit(
        ChannelEnums.Order,
        string[] calldata,
        string calldata portId,
        string calldata channelId,
        ChannelCounterparty.Data calldata,
        string calldata
    ) external virtual override {
        emit MockChannelOpenInit(portId, channelId);
    }

    function onChanOpenTry(
        ChannelEnums.Order,
        string[] calldata,
        string calldata,
        string calldata channelId,
        ChannelCounterparty.Data calldata,
        string calldata,
        string calldata
    ) external virtual override {
        emit MockChannelOpenTry();
    }

    function onChanOpenAck(
        string calldata portId,
        string calldata channelId,
        string calldata counterpartyChannelId,
        string calldata counterpartyVersion
    ) external virtual override {
        emit MockChannelOpenAck();
    }

    function onChanOpenConfirm(
        string calldata portId,
        string calldata channelId
    ) external virtual override {
        emit MockChannelOpenConfirm();
    }

    function onChanCloseInit(
        string calldata portId,
        string calldata channelId
    ) external virtual override {
        emit MockChannelCloseInit();
    }

    function onChanCloseConfirm(
        string calldata portId,
        string calldata channelId
    ) external virtual override {
        emit MockChannelCloseConfirm();
    }
}
