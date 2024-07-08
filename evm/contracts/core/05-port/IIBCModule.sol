pragma solidity ^0.8.23;

import "../../proto/ibc/core/channel/v1/channel.sol";

// IIBCModule defines an interface that implements all the callbacks
// that modules must define as specified in ICS-26
// https://github.com/cosmos/ibc/blob/2921c5cec7b18e4ef77677e16a6b693051ae3b35/spec/core/ics-026-routing-module/README.md
interface IIBCModule {
    function onChanOpenInit(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata connectionHops,
        string calldata portId,
        string calldata channelId,
        IbcCoreChannelV1Counterparty.Data calldata counterparty,
        string calldata version,
        address relayer
    ) external;

    function onChanOpenTry(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata connectionHops,
        string calldata portId,
        string calldata channelId,
        IbcCoreChannelV1Counterparty.Data calldata counterparty,
        string calldata version,
        string calldata counterpartyVersion,
        address relayer
    ) external;

    function onChanOpenAck(
        string calldata portId,
        string calldata channelId,
        string calldata counterpartyChannelId,
        string calldata counterpartyVersion,
        address relayer
    ) external;

    function onChanOpenConfirm(
        string calldata portId,
        string calldata channelId,
        address relayer
    ) external;

    function onChanCloseInit(
        string calldata portId,
        string calldata channelId,
        address relayer
    ) external;

    function onChanCloseConfirm(
        string calldata portId,
        string calldata channelId,
        address relayer
    ) external;

    function onRecvPacket(
        IbcCoreChannelV1Packet.Data calldata,
        address relayer
    ) external returns (bytes memory);

    function onAcknowledgementPacket(
        IbcCoreChannelV1Packet.Data calldata,
        bytes calldata acknowledgement,
        address relayer
    ) external;

    function onTimeoutPacket(
        IbcCoreChannelV1Packet.Data calldata,
        address relayer
    ) external;
}
