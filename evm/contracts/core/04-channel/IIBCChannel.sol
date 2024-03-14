pragma solidity ^0.8.23;

import "../25-handler/IBCMsgs.sol";

interface IIBCChannelHandshake {
    /**
     * @dev channelOpenInit is called by a module to initiate a channel opening handshake with a module on another chain.
     */
    function channelOpenInit(IBCMsgs.MsgChannelOpenInit calldata msg_)
        external
        returns (string memory);

    /**
     * @dev channelOpenTry is called by a module to accept the first step of a channel opening handshake initiated by a module on another chain.
     */
    function channelOpenTry(IBCMsgs.MsgChannelOpenTry calldata msg_)
        external
        returns (string memory);

    /**
     * @dev channelOpenAck is called by the handshake-originating module to acknowledge the acceptance of the initial request by the counterparty module on the other chain.
     */
    function channelOpenAck(IBCMsgs.MsgChannelOpenAck calldata msg_) external;

    /**
     * @dev channelOpenConfirm is called by the counterparty module to close their end of the channel, since the other end has been closed.
     */
    function channelOpenConfirm(IBCMsgs.MsgChannelOpenConfirm calldata msg_)
        external;

    /**
     * @dev channelCloseInit is called by either module to close their end of the channel. Once closed, channels cannot be reopened.
     */
    function channelCloseInit(IBCMsgs.MsgChannelCloseInit calldata msg_)
        external;

    /**
     * @dev channelCloseConfirm is called by the counterparty module to close their end of the
     * channel, since the other end has been closed.
     */
    function channelCloseConfirm(IBCMsgs.MsgChannelCloseConfirm calldata msg_)
        external;
}
