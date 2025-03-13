pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../core/IBCHandler.sol";
import "../core/Relay.sol";
import "../../../contracts/clients/StateLensIcs23MptClient.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

contract StateLensIcs23MptClientTest is Test {
    StateLensIcs23MptClient client;
    address admin = address(0xABCD);
    address ibcHandler;

    function setUp() public {
        ibcHandler = address(0xC0DE);
        StateLensIcs23MptClient implementation = new StateLensIcs23MptClient();
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeWithSelector(
                StateLensIcs23MptClient.initialize.selector, ibcHandler, admin
            )
        );
        client = StateLensIcs23MptClient(address(proxy));
    }

    function test_initialize_ok() public {
        assertEq(client.owner(), admin);
    }
}
