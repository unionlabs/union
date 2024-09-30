pragma solidity ^0.8.23;

import "../core/05-port/IIBCModule.sol";

library IBCAppLib {
    error ErrNotIBC();
}

/**
 * @dev Base contract of the IBC App protocol
 */
abstract contract IBCAppBase is IIBCModule {
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
        if (ibcAddress() != msg.sender) {
            revert IBCAppLib.ErrNotIBC();
        }
    }

    /**
     * @dev See IIBCModule-onChanOpenInit
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onChanOpenInit(
        IBCChannelOrder ordering,
        uint32 connectionId,
        uint32 channelId,
        IBCChannelCounterparty calldata counterpartyEndpoint,
        bytes32 version,
        address relayer
    ) external virtual override onlyIBC {}

    /**
     * @dev See IIBCModule-onChanOpenTry
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onChanOpenTry(
        IBCChannelOrder ordering,
        uint32 connectionId,
        uint32 channelId,
        IBCChannelCounterparty calldata counterpartyEndpoint,
        bytes32 version,
        bytes32 counterpartyVersion,
        address relayer
    ) external virtual override onlyIBC {}

    /**
     * @dev See IIBCModule-onChanOpenAck
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onChanOpenAck(
        uint32 channelId,
        uint32 counterpartyChannelId,
        bytes32 counterpartyVersion,
        address relayer
    ) external virtual override onlyIBC {}

    /**
     * @dev See IIBCModule-onChanOpenConfirm
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onChanOpenConfirm(
        uint32 channelId,
        address relayer
    ) external virtual override onlyIBC {}

    /**
     * @dev See IIBCModule-onChanCloseInit
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onChanCloseInit(
        uint32 channelId,
        address relayer
    ) external virtual override onlyIBC {}

    /**
     * @dev See IIBCModule-onChanCloseConfirm
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onChanCloseConfirm(
        uint32 channelId,
        address relayer
    ) external virtual override onlyIBC {}

    /**
     * @dev See IIBCModule-onRecvPacket
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onRecvPacket(
        IBCPacket calldata packet,
        address relayer
    )
        external
        virtual
        override
        onlyIBC
        returns (bytes memory acknowledgement)
    {}

    function onRecvIntentPacket(
        IBCPacket calldata,
        address,
        bytes calldata
    ) external virtual override onlyIBC returns (bytes memory) {
        revert("unsupported");
    }

    /**
     * @dev See IIBCModule-onAcknowledgementPacket
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onAcknowledgementPacket(
        IBCPacket calldata packet,
        bytes calldata acknowledgement,
        address relayer
    ) external virtual override onlyIBC {}

    /**
     * @dev See IIBCModule-onTimeoutPacket
     *
     * NOTE: You should apply an `onlyIBC` modifier to the function if a derived contract overrides it.
     */
    function onTimeoutPacket(
        IBCPacket calldata packet,
        address relayer
    ) external virtual override onlyIBC {}
}
