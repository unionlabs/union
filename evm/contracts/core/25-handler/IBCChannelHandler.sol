pragma solidity ^0.8.23;

import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCHost.sol";
import "../04-channel/IIBCChannel.sol";
import "../05-port/IIBCModule.sol";
import "../05-port/ModuleManager.sol";

/**
 * @dev IBCChannelHandler is a contract that calls a contract that implements `IIBCChannelHandshake` with delegatecall.
 */
abstract contract IBCChannelHandler is IIBCChannelHandshake {
    address immutable ibcChannel;

    constructor(address _ibcChannel) {
        ibcChannel = _ibcChannel;
    }

    function channelOpenInit(IBCMsgs.MsgChannelOpenInit calldata msg_)
        external
        override
        returns (ChannelId)
    {
        passthrough(ibcChannel);
    }

    function channelOpenTry(IBCMsgs.MsgChannelOpenTry calldata msg_)
        external
        override
        returns (ChannelId)
    {
        passthrough(ibcChannel);
    }

    function channelOpenAck(IBCMsgs.MsgChannelOpenAck calldata msg_)
        external
        override
    {
        passthrough(ibcChannel);
    }

    function channelOpenConfirm(IBCMsgs.MsgChannelOpenConfirm calldata msg_)
        external
        override
    {
        passthrough(ibcChannel);
    }

    function channelCloseInit(IBCMsgs.MsgChannelCloseInit calldata msg_)
        external
        override
    {
        passthrough(ibcChannel);
    }

    function channelCloseConfirm(IBCMsgs.MsgChannelCloseConfirm calldata msg_)
        external
        override
    {
        passthrough(ibcChannel);
    }
}
