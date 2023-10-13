pragma solidity ^0.8.21;

import "../../proto/ibc/core/channel/v1/channel.sol";

// IIBCModule defines an interface that implements all the callbacks
// that modules must define as specified in ICS-26
interface IIBCModule {
    // onChanOpenInit will verify that the relayer-chosen parameters
    // are valid and perform any custom INIT logic.
    // It may return an error if the chosen parameters are invalid
    // in which case the handshake is aborted.
    // If the provided version string is non-empty, OnChanOpenInit should return
    // the version string if valid or an error if the provided version is invalid.
    // If the version string is empty, OnChanOpenInit is expected to
    // return a default version string representing the version(s) it supports.
    // If there is no default version string for the application,
    // it should return an error if provided version is empty string.
    function onChanOpenInit(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata connectionHops,
        string calldata portId,
        string calldata channelId,
        IbcCoreChannelV1Counterparty.Data calldata counterparty,
        string calldata version
    ) external;

    // OnChanOpenTry will verify the relayer-chosen parameters along with the
    // counterparty-chosen version string and perform custom TRY logic.
    // If the relayer-chosen parameters are invalid, the callback must return
    // an error to abort the handshake. If the counterparty-chosen version is not
    // compatible with this modules supported versions, the callback must return
    // an error to abort the handshake. If the versions are compatible, the try callback
    // must select the final version string and return it to core IBC.
    // OnChanOpenTry may also perform custom initialization logic
    function onChanOpenTry(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata connectionHops,
        string calldata portId,
        string calldata channelId,
        IbcCoreChannelV1Counterparty.Data calldata counterparty,
        string calldata version,
        string calldata counterpartyVersion
    ) external;

    // OnChanOpenAck will error if the counterparty selected version string
    // is invalid to abort the handshake. It may also perform custom ACK logic.
    function onChanOpenAck(
        string calldata portId,
        string calldata channelId,
        string calldata counterpartyVersion
    ) external;

    // OnChanOpenConfirm will perform custom CONFIRM logic and may error to abort the handshake.
    function onChanOpenConfirm(
        string calldata portId,
        string calldata channelId
    ) external;

    function onChanCloseInit(
        string calldata portId,
        string calldata channelId
    ) external;

    function onChanCloseConfirm(
        string calldata portId,
        string calldata channelId
    ) external;

    // OnRecvPacket must return an acknowledgement that implements the Acknowledgement interface.
    // In the case of an asynchronous acknowledgement, nil should be returned.
    // If the acknowledgement returned is successful, the state changes on callback are written,
    // otherwise the application state changes are discarded. In either case the packet is received
    // and the acknowledgement is written (in synchronous cases).
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
