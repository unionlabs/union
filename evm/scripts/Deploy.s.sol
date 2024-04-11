pragma solidity ^0.8.23;

import {Script} from "forge-std/Script.sol";
import {Upgrades} from "@openzeppelin-foundry-upgradeable/Upgrades.sol";
import {ERC1967Proxy} from "@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol";

import "../contracts/core/02-client/IBCClient.sol";
import "../contracts/core/03-connection/IBCConnection.sol";
import "../contracts/core/04-channel/IBCChannelHandshake.sol";
import "../contracts/core/04-channel/IBCPacket.sol";
import "../contracts/core/OwnableIBCHandler.sol";
import "../contracts/clients/CometblsClientV2.sol";
import "../contracts/apps/ucs/01-relay/Relay.sol";
import "../contracts/apps/ucs/02-nft/NFT.sol";

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
        CometblsClient client = CometblsClient(
            address(
                new ERC1967Proxy(
                    address(new CometblsClient()),
                    abi.encodeCall(
                        CometblsClient.initialize, (address(handler))
                    )
                )
            )
        );
        UCS01Relay relay = new UCS01Relay(handler);
        UCS02NFT nft = new UCS02NFT(handler);
        vm.stopBroadcast();
    }
}
