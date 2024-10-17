pragma solidity ^0.8.27;

import "../Types.sol";

// IIBCModule defines an interface that implements all the callbacks
// that modules must define as specified in ICS-26
// https://github.com/cosmos/ibc/blob/2921c5cec7b18e4ef77677e16a6b693051ae3b35/spec/core/ics-026-routing-module/README.md
interface IIBCModule {
    function onChanOpenInit(
        IBCChannelOrder order,
        uint32 connectionId,
        uint32 channelId,
        string calldata version,
        address relayer
    ) external;

    function onChanOpenTry(
        IBCChannelOrder order,
        uint32 connectionId,
        uint32 channelId,
        uint32 counterpartyChannelId,
        string calldata version,
        string calldata counterpartyVersion,
        address relayer
    ) external;

    function onChanOpenAck(
        uint32 channelId,
        uint32 counterpartyChannelId,
        string calldata counterpartyVersion,
        address relayer
    ) external;

    function onChanOpenConfirm(uint32 channelId, address relayer) external;

    function onChanCloseInit(uint32 channelId, address relayer) external;

    function onChanCloseConfirm(uint32 channelId, address relayer) external;

    function onRecvIntentPacket(
        IBCPacket calldata packet,
        address marketMaker,
        bytes calldata marketMakerMsg
    ) external returns (bytes memory);

    function onRecvPacket(
        IBCPacket calldata packet,
        address relayer,
        bytes calldata relayerMsg
    ) external returns (bytes memory);

    function onAcknowledgementPacket(
        IBCPacket calldata packet,
        bytes calldata acknowledgement,
        address relayer
    ) external;

    function onTimeoutPacket(IBCPacket calldata, address relayer) external;
}
