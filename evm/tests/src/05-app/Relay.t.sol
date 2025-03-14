// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../core/IBCHandler.sol";
import "../core/Relay.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "solady/utils/LibString.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "solidity-stringutils/strings.sol";
import "solidity-bytes-utils/BytesLib.sol";

contract MockIBCHandler {
    function sendPacket(
        uint32 sourceChannel,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes calldata data
    ) external returns (IBCPacket memory packet) {
        return IBCPacket({
            sourceChannelId: sourceChannel,
            destinationChannelId: 0,
            data: bytes(""),
            timeoutHeight: timeoutHeight,
            timeoutTimestamp: timeoutTimestamp
        });
    }
}

contract MockERC20 is ERC20 {
    constructor(string memory name, string memory symbol) ERC20(name, symbol) {}

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}

contract UCS01RelayTests is Test {
    UCS01Relay relay;
    MockIBCHandler handler; // Mock IBC handler
    address admin = address(0xABcD);
    address user = address(0x1234);
    address relayer = address(0x5678);
    address random_user = address(0x444444);

    error OwnableUnauthorizedAccount(address caller);

    error StringsInsufficientHexLength(uint256 value, uint256 length);

    bytes16 private constant HEX_DIGITS = "0123456789abcdef";

    function toHexString(
        uint256 value,
        uint256 length
    ) internal pure returns (string memory) {
        uint256 localValue = value;
        bytes memory buffer = new bytes(2 * length + 2);
        buffer[0] = "0";
        buffer[1] = "x";
        for (uint256 i = 2 * length + 1; i > 1; --i) {
            buffer[i] = HEX_DIGITS[localValue & 0xf];
            localValue >>= 4;
        }
        if (localValue != 0) {
            revert StringsInsufficientHexLength(value, length);
        }
        return string(buffer);
    }

    function toHexString(
        address addr
    ) internal pure returns (string memory) {
        return toHexString(uint256(uint160(addr)), 20);
    }

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
        relay.updateMetadata(
            IERC20Denom(denom), newName, newSymbol, newDecimals
        );
        vm.stopPrank();

        ERC20Denom updatedDenom = ERC20Denom(denom);
        assertEq(updatedDenom.name(), newName);
        assertEq(updatedDenom.symbol(), newSymbol);
        assertEq(updatedDenom.decimals(), newDecimals);
    }

    function test_initialize_ok() public {
        // Deploy the implementation
        UCS01Relay implementation = new UCS01Relay();

        // Mock IBC Handler
        MockIBCHandler handler = new MockIBCHandler();

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
        UCS01Relay relay = UCS01Relay(address(proxy));

        // Verify that the admin was set correctly
        assertEq(relay.owner(), admin);

        // Verify that the IBC handler was set correctly
        assertEq(address(relay.ibcAddress()), address(handler));
    }

    function test_updateMetadata_nonOwner_reverts() public {
        address denom = address(new ERC20Denom("TestToken"));

        string memory newName = "InvalidName";
        string memory newSymbol = "INV";
        uint8 newDecimals = 0;

        // Attempt to call `updateMetadata` from a non-owner account
        vm.startPrank(user); // Set `user` as the caller
        vm.expectRevert(
            abi.encodeWithSelector(OwnableUnauthorizedAccount.selector, user)
        );
        relay.updateMetadata(
            IERC20Denom(denom), newName, newSymbol, newDecimals
        );
        vm.stopPrank();
    }

    function test_sendToken_success() public {
        // Prepare mocks and setup
        MockIBCHandler handler = new MockIBCHandler();
        MockERC20 token = new MockERC20("TestToken", "TTKN");

        // Mint tokens for the sender
        token.mint(user, 1000 ether);

        LocalToken memory localToken =
            LocalToken({denom: address(token), amount: 500 ether, fee: 0});

        LocalToken[] memory tokens = new LocalToken[](1);
        tokens[0] = localToken;

        vm.startPrank(user);
        token.approve(address(relay), 500 ether); // Approve tokens for transfer
        relay.send(1, abi.encodePacked(relayer), tokens, "", 0, 0);
        vm.stopPrank();

        // Assertions
        assertEq(token.balanceOf(user), 500 ether); // 500 tokens deducted
        assertEq(token.balanceOf(address(relay)), 500 ether); // 500 tokens escrowed
    }

    function test_sendToken_invalidAmount_reverts() public {
        MockERC20 token = new MockERC20("TestToken", "TTKN");

        // Prepare a LocalToken with an invalid amount (0)
        LocalToken memory localToken =
            LocalToken({denom: address(token), amount: 0, fee: 0});

        LocalToken[] memory tokens = new LocalToken[](1);
        tokens[0] = localToken;

        // Attempt to call `send` with an invalid amount
        vm.startPrank(user);
        vm.expectRevert(
            abi.encodeWithSelector(RelayLib.ErrInvalidAmount.selector)
        );
        relay.send(1, abi.encodePacked(relayer), tokens, "", 0, 0);
        vm.stopPrank();
    }

    function test_sendToken_emitsSentEvent() public {
        MockERC20 token = new MockERC20("TestToken", "TTKN");

        // Mint tokens for the sender
        token.mint(user, 1000 ether);

        LocalToken memory localToken =
            LocalToken({denom: address(token), amount: 500 ether, fee: 0});

        LocalToken[] memory tokens = new LocalToken[](1);
        tokens[0] = localToken;

        vm.startPrank(user);
        token.approve(address(relay), 500 ether); // Approve tokens for transfer

        // Expect the `Sent` event
        vm.expectEmit(true, true, true, true);
        emit RelayLib.Sent(
            IBCPacket({
                sourceChannelId: 1,
                destinationChannelId: 0,
                data: "",
                timeoutHeight: 0,
                timeoutTimestamp: 0
            }),
            user,
            toHexString(relayer),
            toHexString(address(token)),
            address(token),
            500 ether
        );

        relay.send(1, abi.encodePacked(relayer), tokens, "", 0, 0);
        vm.stopPrank();
    }
    // function test_refundTokens() public {
    //     MockERC20 token = new MockERC20("TestToken", "TTKN");

    //     // Mint tokens for the relay contract (simulate escrow)
    //     token.mint(address(relay), 500 ether);

    //     Token[] memory tokens = new Token[](1);
    //     tokens[0] = Token({
    //         denom: toHexString(address(token)),
    //         amount: 500 ether,
    //         fee: 0
    //     });

    //     // Prepare the IBCPacket for refund
    //     IBCPacket memory ibcPacket = IBCPacket({
    //         sourceChannelId: 1,
    //         destinationChannelId: 0,
    //         data: abi.encode(
    //             RelayPacket({
    //                 sender: abi.encodePacked(user),
    //                 receiver: abi.encodePacked(user),
    //                 tokens: tokens,
    //                 extension: ""
    //             })
    //         ),
    //         timeoutHeight: 0,
    //         timeoutTimestamp: 0
    //     });

    //     // Simulate token refund
    //     vm.startPrank(admin);
    //     relay.refundTokens(ibcPacket);
    //     vm.stopPrank();

    //     // Assert that the tokens were refunded
    //     assertEq(token.balanceOf(user), 500 ether);
    //     assertEq(token.balanceOf(address(relay)), 0 ether);
    // }

    function test_outstandingUpdates() public {
        MockERC20 token = new MockERC20("TestToken", "TTKN");

        // Mint tokens for the sender
        token.mint(user, 1000 ether);

        LocalToken memory localToken =
            LocalToken({denom: address(token), amount: 500 ether, fee: 0});

        LocalToken[] memory tokens = new LocalToken[](1);
        tokens[0] = localToken;

        vm.startPrank(user);
        token.approve(address(relay), 500 ether);
        relay.send(1, abi.encodePacked(relayer), tokens, "", 0, 0);
        vm.stopPrank();

        // Verify outstanding amount
        uint256 outstandingAmount = relay.getOutstanding(1, address(token));
        assertEq(outstandingAmount, 500 ether);
    }

    function increase_outstanding(MockERC20 token, uint128 amount) public {
        token.mint(user, amount * 2);

        LocalToken memory localToken =
            LocalToken({denom: address(token), amount: amount, fee: 0});

        LocalToken[] memory tokens = new LocalToken[](1);
        tokens[0] = localToken;

        vm.startPrank(user);
        token.approve(address(relay), 500 ether);
        relay.send(1, abi.encodePacked(relayer), tokens, "", 0, 0);
        vm.stopPrank();
    }

    function test_getOutstanding_success() public {
        MockERC20 token = new MockERC20("TestToken", "TTKN");

        // Mint tokens for the sender
        token.mint(user, 1000 ether);

        LocalToken memory localToken =
            LocalToken({denom: address(token), amount: 500 ether, fee: 0});

        LocalToken[] memory tokens = new LocalToken[](1);
        tokens[0] = localToken;

        // Perform the token send
        vm.startPrank(user);
        token.approve(address(relay), 500 ether);
        relay.send(1, abi.encodePacked(relayer), tokens, "", 0, 0);
        vm.stopPrank();

        // Verify the outstanding amount
        uint256 outstanding = relay.getOutstanding(1, address(token));
        assertEq(outstanding, 500 ether, "Outstanding amount is incorrect");
    }

    function test_onChanOpenInit_invalidVersion_reverts() public {
        vm.startPrank(address(handler));
        string memory invalidVersion = "invalid-version";

        // Expect revert due to invalid version
        vm.expectRevert(
            abi.encodeWithSelector(RelayLib.ErrInvalidProtocolVersion.selector)
        );
        relay.onChanOpenInit(1, 1, invalidVersion, user);
        vm.stopPrank();
    }

    function test_onChanOpenTry_invalidVersion_reverts() public {
        vm.startPrank(address(handler));
        string memory validVersion = "ucs01-relay-1";
        string memory invalidCounterpartyVersion = "invalid-version";

        // Expect revert due to invalid counterparty version
        vm.expectRevert(
            abi.encodeWithSelector(
                RelayLib.ErrInvalidCounterpartyProtocolVersion.selector
            )
        );
        relay.onChanOpenTry(
            1, 1, 1, validVersion, invalidCounterpartyVersion, user
        );
        vm.stopPrank();
    }

    function test_onChanOpenTry_invalidVersion_reverts_protocol() public {
        vm.startPrank(address(handler));
        string memory validVersion = "invalid-version-ucs01-relay-1";
        string memory invalidCounterpartyVersion = "invalid-version";

        // Expect revert due to invalid counterparty version
        vm.expectRevert(
            abi.encodeWithSelector(RelayLib.ErrInvalidProtocolVersion.selector)
        );
        relay.onChanOpenTry(
            1, 1, 1, validVersion, invalidCounterpartyVersion, user
        );
        vm.stopPrank();
    }

    function test_onTimeoutPacket_success() public {
        MockERC20 token = new MockERC20("TestToken", "TTKN");

        // Mint tokens for the relay contract (simulate escrow)
        token.mint(address(relay), 500 ether);

        // Prepare a Token array for the RelayPacket
        Token[] memory tokens = new Token[](1);

        tokens[0] = Token({
            denom: toHexString(address(token)),
            amount: 500 ether,
            fee: 0
        });

        // Prepare the RelayPacket and encode it
        RelayPacket memory packet = RelayPacket({
            sender: abi.encodePacked(user),
            receiver: abi.encodePacked(user),
            tokens: tokens,
            extension: ""
        });

        bytes memory encodedPacket = RelayPacketLib.encode(
            RelayPacket({
                sender: abi.encodePacked(user),
                receiver: abi.encodePacked(user),
                tokens: tokens,
                extension: ""
            })
        );

        // Prepare the IBCPacket for timeout
        IBCPacket memory ibcPacket = IBCPacket({
            sourceChannelId: 1,
            destinationChannelId: 0,
            data: RelayPacketLib.encode(packet),
            timeoutHeight: 0,
            timeoutTimestamp: 0
        });

        // Simulate timeout
        uint128 extra_amount = 500 ether;
        increase_outstanding(token, extra_amount);
        vm.startPrank(address(handler));
        relay.onTimeoutPacket(ibcPacket, admin);
        vm.stopPrank();

        // Assert that the tokens were refunded
        assertEq(
            token.balanceOf(user),
            500 ether + extra_amount,
            "Tokens were not refunded correctly"
        );
        assertEq(
            token.balanceOf(address(relay)),
            extra_amount,
            "Relay contract still holds tokens"
        );
    }

    function test_onTimeoutPacket_multipleTokens() public {
        MockERC20 token1 = new MockERC20("Token1", "TK1");
        MockERC20 token2 = new MockERC20("Token2", "TK2");

        // Mint tokens for the relay contract (simulate escrow)
        token1.mint(address(relay), 300 ether);
        token2.mint(address(relay), 200 ether);

        // Prepare multiple tokens for the RelayPacket
        Token[] memory tokens = new Token[](2);

        tokens[0] = Token({
            denom: toHexString(address(token1)),
            amount: 300 ether,
            fee: 0
        });
        tokens[1] = Token({
            denom: toHexString(address(token2)),
            amount: 200 ether,
            fee: 0
        });

        // Encode the RelayPacket
        RelayPacket memory packet = RelayPacket({
            sender: abi.encodePacked(user),
            receiver: abi.encodePacked(user),
            tokens: tokens,
            extension: ""
        });

        // Prepare the IBCPacket for timeout
        IBCPacket memory ibcPacket = IBCPacket({
            sourceChannelId: 1,
            destinationChannelId: 0,
            data: RelayPacketLib.encode(packet),
            timeoutHeight: 0,
            timeoutTimestamp: 0
        });

        // Simulate timeout
        uint128 extra_amount_tok1 = 300 ether;
        increase_outstanding(token1, extra_amount_tok1);
        uint128 extra_amount_tok2 = 200 ether;
        increase_outstanding(token2, extra_amount_tok2);
        vm.startPrank(address(handler));
        relay.onTimeoutPacket(ibcPacket, admin);
        vm.stopPrank();

        // Assert that the tokens were refunded
        assertEq(
            token1.balanceOf(user),
            300 ether + extra_amount_tok1,
            "Token1 was not refunded correctly"
        );
        assertEq(
            token2.balanceOf(user),
            200 ether + extra_amount_tok2,
            "Token2 was not refunded correctly"
        );
        assertEq(
            token1.balanceOf(address(relay)),
            0 ether + extra_amount_tok1,
            "Relay still holds Token1"
        );
        assertEq(
            token2.balanceOf(address(relay)),
            0 ether + extra_amount_tok2,
            "Relay still holds Token2"
        );
    }

    function test_onRecvPacketProcessing_localTransfer_unauthorized() public {
        MockERC20 token = new MockERC20("TestToken", "TTKN");

        // Mint tokens to the relay to simulate escrow
        token.mint(address(relay), 500 ether);

        // Prepare a local token for the RelayPacket
        Token[] memory tokens = new Token[](1);
        string memory prefix = RelayLib.makeDenomPrefix(1);
        string memory denom_str =
            string(abi.encodePacked(prefix, toHexString(address(token))));
        tokens[0] = Token({denom: denom_str, amount: 500 ether, fee: 50 ether});

        // Encode the RelayPacket
        RelayPacket memory packet = RelayPacket({
            sender: abi.encodePacked(user),
            receiver: abi.encodePacked(relayer),
            tokens: tokens,
            extension: ""
        });

        // Prepare the IBCPacket for processing
        IBCPacket memory ibcPacket = IBCPacket({
            sourceChannelId: 1,
            destinationChannelId: 1,
            data: RelayPacketLib.encode(packet),
            timeoutHeight: 0,
            timeoutTimestamp: 0
        });

        // Simulate packet processing
        vm.startPrank(address(handler));
        vm.expectRevert(
            abi.encodeWithSelector(RelayLib.ErrUnauthorized.selector)
        );
        relay.onRecvPacketProcessing(ibcPacket, address(relayer));
        vm.stopPrank();
    }

    function test_onRecvPacketProcessing_localTransfer() public {
        MockERC20 token = new MockERC20("TestToken", "TTKN");

        // Mint tokens to the relay to simulate escrow
        token.mint(address(relay), 500 ether);

        // Prepare a local token for the RelayPacket
        Token[] memory tokens = new Token[](1);
        string memory prefix = RelayLib.makeDenomPrefix(1);
        string memory denom_str =
            string(abi.encodePacked(prefix, toHexString(address(token))));
        tokens[0] = Token({denom: denom_str, amount: 450 ether, fee: 0});

        // Encode the RelayPacket
        RelayPacket memory packet = RelayPacket({
            sender: abi.encodePacked(user),
            receiver: abi.encodePacked(random_user),
            tokens: tokens,
            extension: ""
        });

        // Prepare the IBCPacket for processing
        IBCPacket memory ibcPacket = IBCPacket({
            sourceChannelId: 1,
            destinationChannelId: 1,
            data: RelayPacketLib.encode(packet),
            timeoutHeight: 0,
            timeoutTimestamp: 0
        });

        // Simulate packet processing
        uint128 extra_amount = 500 ether;
        increase_outstanding(token, extra_amount);
        vm.startPrank(address(handler));
        relay.onRecvPacket(ibcPacket, address(relayer), bytes(""));
        vm.stopPrank();

        // Assertions
        assertEq(
            token.balanceOf(random_user),
            450 ether,
            "Relayer did not receive the correct amount"
        );
    }

    function test_onRecvPacketProcessing_remoteTransfer() public {
        MockERC20 token = new MockERC20("TestToken", "TTKN");

        // Prepare a remote-origin token for the RelayPacket
        Token[] memory tokens = new Token[](1);
        string memory denom_str = toHexString(address(token));
        tokens[0] = Token({denom: denom_str, amount: 450 ether, fee: 0});

        // Encode the RelayPacket
        RelayPacket memory packet = RelayPacket({
            sender: abi.encodePacked(user),
            receiver: abi.encodePacked(random_user),
            tokens: tokens,
            extension: ""
        });

        // Prepare the IBCPacket for processing
        IBCPacket memory ibcPacket = IBCPacket({
            sourceChannelId: 1,
            destinationChannelId: 1,
            data: RelayPacketLib.encode(packet),
            timeoutHeight: 0,
            timeoutTimestamp: 0
        });

        // Simulate packet processing
        vm.startPrank(address(handler));
        relay.onRecvPacket(ibcPacket, address(relayer), bytes(""));
        vm.stopPrank();

        // Assertions
        string memory created_denom = RelayLib.makeForeignDenom(1, denom_str);
        address denomAddress = relay.getDenomAddress(1, created_denom);

        assertEq(
            IERC20(denomAddress).balanceOf(random_user),
            450 ether,
            "Random user did not receive the correct amount"
        );
    }

    function test_onAcknowledgementPacket_success() public {
        MockERC20 token = new MockERC20("TestToken", "TTKN");

        // Prepare a RelayPacket
        Token[] memory tokens = new Token[](1);
        string memory prefix = RelayLib.makeDenomPrefix(1);
        string memory denom_str =
            string(abi.encodePacked(prefix, toHexString(address(token))));
        tokens[0] = Token({denom: denom_str, amount: 500 ether, fee: 0});

        RelayPacket memory packet = RelayPacket({
            sender: abi.encodePacked(user),
            receiver: abi.encodePacked(random_user),
            tokens: tokens,
            extension: ""
        });

        // Prepare the IBCPacket
        IBCPacket memory ibcPacket = IBCPacket({
            sourceChannelId: 1,
            destinationChannelId: 1,
            data: RelayPacketLib.encode(packet),
            timeoutHeight: 0,
            timeoutTimestamp: 0
        });

        // Call `onAcknowledgementPacket` with a success acknowledgment
        bytes memory ack = abi.encodePacked(RelayLib.ACK_SUCCESS);

        vm.startPrank(address(handler));
        relay.onAcknowledgementPacket(ibcPacket, ack, address(relayer));
        vm.stopPrank();

        // Assert that nothing is refunded since it's a success
        assertEq(
            token.balanceOf(address(relay)), 0, "Tokens should not be refunded"
        );
    }

    function test_onAcknowledgementPacket_invalidAck_reverts() public {
        // Prepare a dummy IBCPacket
        IBCPacket memory ibcPacket = IBCPacket({
            sourceChannelId: 1,
            destinationChannelId: 1,
            data: bytes(""),
            timeoutHeight: 0,
            timeoutTimestamp: 0
        });

        // Invalid acknowledgment (length not 1)
        bytes memory invalidAck = hex"02";

        vm.startPrank(address(handler));
        vm.expectRevert(
            abi.encodeWithSelector(RelayLib.ErrInvalidAcknowledgement.selector)
        );
        relay.onAcknowledgementPacket(ibcPacket, invalidAck, address(relayer));
        vm.stopPrank();
    }

    function test_onChanOpenAck_validVersion() public {
        string memory validVersion = "ucs01-relay-1";

        vm.startPrank(address(handler));
        relay.onChanOpenAck(1, 1, validVersion, address(0));
        vm.stopPrank();

        // No revert means the test passes for valid versions
        assertTrue(true, "onChanOpenAck did not revert for a valid version");
    }

    function test_onChanOpenAck_invalidVersion_reverts() public {
        string memory invalidVersion = "invalid-version";

        vm.startPrank(address(handler));
        vm.expectRevert(
            abi.encodeWithSelector(
                RelayLib.ErrInvalidCounterpartyProtocolVersion.selector
            )
        );
        relay.onChanOpenAck(1, 1, invalidVersion, address(0));
        vm.stopPrank();
    }

    function test_onChanCloseInit_reverts() public {
        vm.startPrank(address(handler));
        vm.expectRevert(
            abi.encodeWithSelector(RelayLib.ErrUnstoppable.selector)
        );
        relay.onChanCloseInit(1, address(0));
        vm.stopPrank();
    }

    function test_onChanCloseConfirm_reverts() public {
        vm.startPrank(address(handler));
        vm.expectRevert(
            abi.encodeWithSelector(RelayLib.ErrUnstoppable.selector)
        );
        relay.onChanCloseConfirm(1, address(0));
        vm.stopPrank();
    }

    function test_onlyIBC() public {
        vm.startPrank(address(relayer));
        vm.expectRevert(abi.encodeWithSelector(IBCAppLib.ErrNotIBC.selector));
        relay.onChanCloseConfirm(1, address(0));
        vm.stopPrank();
    }

    function test_onRecvIntentPacket() public {
        vm.startPrank(address(handler));
        vm.expectRevert(
            abi.encodeWithSelector(IBCAppLib.ErrNotImplemented.selector)
        );
        relay.onRecvIntentPacket(
            IBCPacket({
                sourceChannelId: 1,
                destinationChannelId: 1,
                data: bytes(""),
                timeoutHeight: 0,
                timeoutTimestamp: 0
            }),
            address(0),
            bytes("")
        );
        vm.stopPrank();
    }
}
