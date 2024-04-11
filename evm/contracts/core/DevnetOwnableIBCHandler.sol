pragma solidity ^0.8.23;

import "./OwnableIBCHandler.sol";
import "./DevnetIBCHandlerInit.sol";

/**
 * @dev DevnetOwnableIBCHandler is a contract that implements [ICS-25](https://github.com/cosmos/ibc/tree/main/spec/core/ics-025-handler-interface).
 */
contract DevnetOwnableIBCHandler is OwnableIBCHandler {
    address ibcHandlerInit;

    constructor() {
        _disableInitializers();
    }

    /**
     * @dev The arguments of constructor must satisfy the followings:
     * @param ibcClient is the address of a contract that implements `IIBCClient`.
     * @param ibcConnection is the address of a contract that implements `IIBCConnectionHandshake`.
     * @param ibcChannel is the address of a contract that implements `IIBCChannelHandshake`.
     * @param ibcPacket is the address of a contract that implements `IIBCPacket`.
     */
    function initialize(
        address ibcClient,
        address ibcConnection,
        address ibcChannel,
        address ibcPacket,
        address ibcHandlerInit_
    ) public initializer {
        OwnableIBCHandler.initialize(
            ibcClient, ibcConnection, ibcChannel, ibcPacket
        );
        ibcHandlerInit = ibcHandlerInit_;
    }

    function setupInitialChannel(
        string calldata connectionId,
        IbcCoreConnectionV1ConnectionEnd.Data calldata connection,
        string calldata portId,
        string calldata channelId,
        IbcCoreChannelV1Channel.Data calldata channel,
        address moduleAddress
    ) public onlyOwner {
        passthrough(ibcHandlerInit);
    }
}
