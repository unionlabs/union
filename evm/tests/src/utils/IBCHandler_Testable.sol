pragma solidity ^0.8.23;

import {IBCHandler} from "../../../contracts/core/25-handler/IBCHandler.sol";
import {IbcCoreConnectionV1ConnectionEnd} from
    "../../../contracts/proto/ibc/core/connection/v1/connection.sol";
import {IBCConnection} from
    "../../../contracts/core/03-connection/IBCConnection.sol";
import {IBCClient} from "../../../contracts/core/02-client/IBCClient.sol";
import {IBCChannelHandshake} from
    "../../../contracts/core/04-channel/IBCChannelHandshake.sol";
import {IBCPacket} from "../../../contracts/core/04-channel/IBCPacket.sol";

contract IBCHandler_Testable is IBCHandler {
    constructor()
        IBCHandler(
            address(new IBCClient()),
            address(new IBCConnection()),
            address(new IBCChannelHandshake()),
            address(new IBCPacket())
        )
    {}
}
