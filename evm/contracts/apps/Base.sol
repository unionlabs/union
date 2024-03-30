pragma solidity ^0.8.23;

import "@openzeppelin/utils/Context.sol";
import "../core/05-port/IIBCModule.sol";
import {IBCChannelLib} from
    "../../contracts/core/04-channel/IBCChannelHandshake.sol";

library IBCAppLib {
    error ErrNotIBC();
}

/**
 * @dev Base contract of the IBC App protocol
 */
abstract contract IBCAppBase is Context, IIBCModule {
    /**
     * @dev Throws if called by any account other than the IBC contract.
     */
    modifier onlyIBC() {
        _checkIBC();
        _;
    }

    /**
     * @dev Returns the address of the IBC contract.
     */
    function ibcAddress() public view virtual returns (address);

    /**
     * @dev Throws if the sender is not the IBC contract.
     */
    function _checkIBC() internal view virtual {
        if (ibcAddress() != _msgSender()) {
            revert IBCAppLib.ErrNotIBC();
        }
    }

    /**
     * @dev See IIBCModule-onChanOpenInit
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onChanOpenInit(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata connectionHops,
        string calldata portId,
        ChannelId channelId,
        IBCChannelTypes.Counterparty calldata counterpartyEndpoint,
        string calldata version
    ) external virtual override onlyIBC {}

    /**
     * @dev See IIBCModule-onChanOpenTry
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onChanOpenTry(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata connectionHops,
        string calldata portId,
        ChannelId channelId,
        IBCChannelTypes.Counterparty calldata counterpartyEndpoint,
        string calldata version,
        string calldata counterpartyVersion
    ) external virtual override onlyIBC {}

    /**
     * @dev See IIBCModule-onChanOpenAck
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onChanOpenAck(
        string calldata portId,
        ChannelId channelId,
        ChannelId counterpartyChannelId,
        string calldata counterpartyVersion
    ) external virtual override onlyIBC {}

    /**
     * @dev See IIBCModule-onChanOpenConfirm
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onChanOpenConfirm(
        string calldata portId,
        ChannelId channelId
    ) external virtual override onlyIBC {}

    /**
     * @dev See IIBCModule-onChanCloseInit
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onChanCloseInit(
        string calldata portId,
        ChannelId channelId
    ) external virtual override onlyIBC {}

    /**
     * @dev See IIBCModule-onChanCloseConfirm
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onChanCloseConfirm(
        string calldata portId,
        ChannelId channelId
    ) external virtual override onlyIBC {}

    /**
     * @dev See IIBCModule-onRecvPacket
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onRecvPacket(
        IbcCoreChannelV1Packet.Data calldata packet,
        address relayer
    )
        external
        virtual
        override
        onlyIBC
        returns (bytes memory acknowledgement)
    {}

    /**
     * @dev See IIBCModule-onAcknowledgementPacket
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onAcknowledgementPacket(
        IbcCoreChannelV1Packet.Data calldata packet,
        bytes calldata acknowledgement,
        address relayer
    ) external virtual override onlyIBC {}

    /**
     * @dev See IIBCModule-onTimeoutPacket
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onTimeoutPacket(
        IbcCoreChannelV1Packet.Data calldata packet,
        address relayer
    ) external virtual override onlyIBC {}
}
