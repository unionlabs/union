pragma solidity ^0.8.23;

import {Script} from "forge-std/Script.sol";
import {Upgrades} from "openzeppelin-foundry-upgrades/Upgrades.sol";
import {ERC1967Proxy} from "@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol";

import "../contracts/core/02-client/IBCClient.sol";
import "../contracts/core/03-connection/IBCConnection.sol";
import "../contracts/core/04-channel/IBCChannelHandshake.sol";
import "../contracts/core/04-channel/IBCPacket.sol";
import "../contracts/core/OwnableIBCHandler.sol";
import "../contracts/clients/CometblsClientV2.sol";
import "../contracts/apps/ucs/01-relay/Relay.sol";

contract DeployIBCStack is Script {
    function setUp() public {}

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(privateKey);

        IBCHandler handler = IBCHandler(
                                      address(
                                              new ERC1967Proxy(
                                                               address(new OwnableIBCHandler()),
                                                               abi.encodeCall(
                                                                              IBCHandler.initialize,
                                                                              (
                                                                               address(new IBCClient()),
                                                                               address(new IBCConnection()),
                                                                               address(new IBCChannelHandshake()),
                                                                               address(new IBCPacket())
                                                                              )
                                                               )
                                              )
                                      )
        );

              /* { path = "core/02-client/IBCClient.sol"; name = "IBCClient"; } */
              /* { path = "core/03-connection/IBCConnection.sol"; name = "IBCConnection"; } */
              /* { path = "core/04-channel/IBCChannelHandshake.sol"; name = "IBCChannelHandshake"; } */
              /* { path = "core/04-channel/IBCPacket.sol"; name = "IBCPacket"; } */
              /* { path = "core/DevnetIBCHandlerInit.sol"; name = "DevnetIBCHandlerInit"; } */
              /* { path = "core/DevnetOwnableIBCHandler.sol"; name = "DevnetOwnableIBCHandler"; args = ''--constructor-args "$IBCCLIENT" "$IBCCONNECTION" "$IBCCHANNELHANDSHAKE" "$IBCPACKET" "$DEVNETIBCHANDLERINIT"''; } */

              /* { path = "clients/CometblsClientV2.sol"; name = "CometblsClient"; args = ''--constructor-args "$DEVNETOWNABLEIBCHANDLER"''; } */

              /* { path = "apps/ucs/01-relay/Relay.sol"; name = "UCS01Relay"; args = ''--constructor-args "$DEVNETOWNABLEIBCHANDLER" "1"'';} */
        vm.stopBroadcast();
    }
}
