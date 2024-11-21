// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../core/IBCHandler.sol";
import "../core/Relay.sol";
import "@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol";


contract MockIBCHandler {
     function sendPacket(
        uint32 sourceChannel,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes calldata data
    ) external returns (IBCPacket memory packet) {
        
    }
}
contract UCS01RelayTests is Test {
    UCS01Relay relay;
    MockIBCHandler handler; // Mock IBC handler
    address admin = address(0xABcD);
    address user = address(0x1234);
    address relayer = address(0x5678);

    event mylog(address data);
    function setUp() public {
        // Deploy the mock IBC handler
        handler = new MockIBCHandler();

        // Deploy the UCS01Relay implementation
        UCS01Relay implementation = new UCS01Relay();

        // Deploy the proxy and initialize it with the implementation
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeWithSelector(
                UCS01Relay.initialize.selector,
                IIBCPacket(address(handler)),
                admin
            )
        );

        // Cast proxy to UCS01Relay to interact with it
        relay = UCS01Relay(address(proxy));
    }

    function test_updateMetadata_ok() public {
        vm.startPrank(address(relay));
        address denom = address(new ERC20Denom("TestToken"));
        vm.stopPrank();

        string memory newName = "UpdatedName";
        string memory newSymbol = "UPD";
        uint8 newDecimals = 6;

        vm.startPrank(admin);
        emit mylog(relay.owner());
        relay.updateMetadata(IERC20Denom(denom), newName, newSymbol, newDecimals);
        vm.stopPrank();

        ERC20Denom updatedDenom = ERC20Denom(denom);
        assertEq(updatedDenom.name(), newName);
        assertEq(updatedDenom.symbol(), newSymbol);
        assertEq(updatedDenom.decimals(), newDecimals);
    }

    function test_send_localTokens_ok() public {
        // Arrange
        vm.startPrank(user);
        uint32 sourceChannel = 1;
        address token = address(new ERC20Denom("TestToken"));
        uint128 amount = 1000;
        uint128 fee = 10;

        // Mint tokens to the user
        IERC20Denom(token).mint(user, amount);
        IERC20Denom(token).approve(address(relay), amount);
        vm.stopPrank();

        LocalToken[] memory tokens = new LocalToken[](1);
        tokens[0].denom = token;
        tokens[0].amount = amount;
        tokens[0].fee = fee;

        // Create an IBCPacket structure
        IBCPacket memory packet = IBCPacket({
            sourceChannel: sourceChannel,
            destinationChannel: 2, // Example destination channel
            data: abi.encode(tokens),
            timeoutHeight: 123,
            timeoutTimestamp: 456
        });

        // Act
        vm.startPrank(user);
        relay.send(
            sourceChannel,
            abi.encodePacked(relayer),
            tokens,
            "test_extension",
            123, // timeoutHeight
            456  // timeoutTimestamp
        );
        vm.stopPrank();

        // Assert
        // Check if the event is emitted
        vm.expectEmit();
        emit RelayLib.Sent(
            packet,
            user,
            "test_extension",
            "asd",
            address(token),
            // token,
            amount
        );

        // Check if the escrow amount is correctly tracked
        uint256 outstanding = relay.getOutstanding(sourceChannel, token);
        assertEq(outstanding, amount);
    }


}
