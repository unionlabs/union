pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../core/IBCHandler.sol";
import "../core/Relay.sol";
import "../../../contracts/clients/EvmInCosmosClient.sol";
import "@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol";

contract EvmInCosmosClientTest is Test {
    EvmInCosmosClient client;
    address admin = address(0xABCD);
    address ibcHandler;

    function setUp() public {
        ibcHandler = address(0xC0DE);
        EvmInCosmosClient implementation = new EvmInCosmosClient();
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeWithSelector(
                EvmInCosmosClient.initialize.selector, ibcHandler, admin
            )
        );
        client = EvmInCosmosClient(address(proxy));
    }

    function test_initialize_ok() public {
        assertEq(client.owner(), admin);
    }
}
