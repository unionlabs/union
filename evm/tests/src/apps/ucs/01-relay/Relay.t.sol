pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "solidity-stringutils/strings.sol";
import "solady/utils/LibString.sol";
import "../../../../../contracts/apps/ucs/01-relay/Relay.sol";
import "../../../../../contracts/apps/ucs/01-relay/ERC20Denom.sol";
import "../../../../../contracts/apps/ucs/01-relay/IERC20Denom.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "../../../utils/IBCHandler_Testable.sol";
import {IBCHandler} from "../../../../../contracts/core/25-handler/IBCHandler.sol";
import {IBCConnection} from "../../../../../contracts/core/03-connection/IBCConnection.sol";
import {IBCClient} from "../../../../../contracts/core/02-client/IBCClient.sol";
import {IBCChannelHandshake} from "../../../../../contracts/core/04-channel/IBCChannelHandshake.sol";
import {IIBCPacket} from "../../../../../contracts/core/04-channel/IIBCChannel.sol";
import {IBCPacket} from "../../../../../contracts/core/04-channel/IBCPacket.sol";

contract IBCHandlerFake is IBCHandler {
    constructor()
        IBCHandler(
            address(new IBCClient()),
            address(new IBCConnection()),
            address(new IBCChannelHandshake()),
            address(new IBCPacket())
        )
    {}

    function sendPacket(
        string calldata sourcePort,
        string calldata sourceChannel,
        IbcCoreClientV1Height.Data calldata timeoutHeight,
        uint64 timeoutTimestamp,
        bytes calldata data
    ) external override {}
}

contract RelayTests is Test {
    using LibString for *;
    using strings for *;

    IBCHandler ibcHandler;

    constructor() {
        ibcHandler = new IBCHandlerFake();
    }

    function testRelay_isRemote_ok() public {
        assertEq(RelayLib.isRemote("a", "b", "a/b/X"), true);
        assertEq(RelayLib.isRemote("aa.bb", "c", "aa.bb/c/X"), true);
    }

    function testRelay_isRemote_ko() public {
        assertEq(RelayLib.isRemote("a", "b", "b/b/X"), false);
        assertEq(RelayLib.isRemote("aa.bb", "c", "aa.b/c/X"), false);
    }

    function testRelay_makeForeignDenom() public {
        assertEq(RelayLib.makeForeignDenom("a", "b", "BLA"), "a/b/BLA");
        assertEq(
            RelayLib.makeForeignDenom("wasm.xyz", "channel-1", "muno"),
            "wasm.xyz/channel-1/muno"
        );
    }

    function testRelay_makeDenomPrefix() public {
        assertEq(RelayLib.makeDenomPrefix("a", "b"), "a/b/");
        assertEq(
            RelayLib.makeDenomPrefix("wasm.xyz", "channel-99"),
            "wasm.xyz/channel-99/"
        );
    }

    function testRelay_hexToAddress(address addr) public {
        assertEq(RelayLib.hexToAddress(addr.toHexString()), addr);
    }

    function testRelay_onRecvPacketProcessing_onlySelf(
        uint64 sequence,
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel,
        uint64 timeoutRevisionNumber,
        uint64 timeoutRevisionHeight,
        uint64 timeoutTimestamp,
        address relayer
    ) public {
        vm.startPrank(address(ibcHandler));

        UCS01Relay relay = new UCS01Relay(ibcHandler);
        vm.expectRevert();
        relay.onRecvPacketProcessing(
            IbcCoreChannelV1Packet.Data({
                sequence: sequence,
                source_port: sourcePort,
                source_channel: sourceChannel,
                destination_port: destinationPort,
                destination_channel: destinationChannel,
                data: hex"00",
                timeout_height: IbcCoreClientV1Height.Data({
                    revision_number: timeoutRevisionNumber,
                    revision_height: timeoutRevisionHeight
                }),
                timeout_timestamp: timeoutTimestamp
            }),
            relayer
        );
    }

    function testRelay_onRecvPacket_invalidIdentity(
        uint64 sequence,
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel,
        uint64 timeoutRevisionNumber,
        uint64 timeoutRevisionHeight,
        uint64 timeoutTimestamp,
        address relayer
    ) public {
        vm.startPrank(address(ibcHandler));

        UCS01Relay relay = new UCS01Relay(ibcHandler);
        vm.record();
        bytes memory acknowledgement = relay.onRecvPacket(
            IbcCoreChannelV1Packet.Data({
                sequence: sequence,
                source_port: sourcePort,
                source_channel: sourceChannel,
                destination_port: destinationPort,
                destination_channel: destinationChannel,
                data: hex"00",
                timeout_height: IbcCoreClientV1Height.Data({
                    revision_number: timeoutRevisionNumber,
                    revision_height: timeoutRevisionHeight
                }),
                timeout_timestamp: timeoutTimestamp
            }),
            relayer
        );
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(
            address(relay)
        );
        assertEq(writes.length, 0);
        assertEq(acknowledgement, abi.encodePacked(RelayLib.ACK_FAILURE));
    }

    function testRelay_send_local(
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel,
        bytes memory sender,
        address relayer,
        string memory denomName,
        uint128 amount
    ) public {
        vm.assume(relayer != address(0));
        vm.assume(amount > 0);

        UCS01Relay relay = new UCS01Relay(ibcHandler);

        vm.prank(address(ibcHandler));
        relay.onChanOpenTry(
            IbcCoreChannelV1GlobalEnums.Order.ORDER_ORDERED,
            new string[](0),
            destinationPort,
            destinationChannel,
            IbcCoreChannelV1Counterparty.Data({
                port_id: sourcePort,
                channel_id: sourceChannel
            }),
            RelayLib.VERSION,
            RelayLib.VERSION
        );

        ERC20Denom denomAddress = new ERC20Denom(denomName);
        IERC20Denom(denomAddress).mint(address(this), amount);
        IERC20Denom(denomAddress).approve(address(relay), amount);

        LocalToken[] memory localTokens = new LocalToken[](1);
        localTokens[0].denom = address(denomAddress);
        localTokens[0].amount = amount;

        vm.expectEmit(false, false, false, false);
        emit RelayLib.Sent(address(0), "", "", address(0), 0);

        relay.send(
            destinationPort,
            destinationChannel,
            sender,
            localTokens,
            0,
            0
        );
    }

    function testRelay_onRecvPacket_localToken(
        uint64 sequence,
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel,
        uint64 timeoutRevisionNumber,
        uint64 timeoutRevisionHeight,
        uint64 timeoutTimestamp,
        bytes memory sender,
        address receiver,
        address relayer,
        string memory denomName,
        uint128 amount
    ) public {
        vm.assume(receiver != address(0));
        vm.assume(relayer != address(0));
        vm.assume(amount > 0);

        UCS01Relay relay = new UCS01Relay(ibcHandler);

        vm.prank(address(ibcHandler));
        relay.onChanOpenTry(
            IbcCoreChannelV1GlobalEnums.Order.ORDER_ORDERED,
            new string[](0),
            destinationPort,
            destinationChannel,
            IbcCoreChannelV1Counterparty.Data({
                port_id: sourcePort,
                channel_id: sourceChannel
            }),
            RelayLib.VERSION,
            RelayLib.VERSION
        );

        ERC20Denom denomAddress = new ERC20Denom(denomName);
        IERC20Denom(denomAddress).mint(address(this), amount);
        IERC20Denom(denomAddress).approve(address(relay), amount);

        LocalToken[] memory localTokens = new LocalToken[](1);
        localTokens[0].denom = address(denomAddress);
        localTokens[0].amount = amount;

        vm.expectEmit(false, false, false, false);
        emit RelayLib.Sent(address(0), "", "", address(0), 0);

        relay.send(
            destinationPort,
            destinationChannel,
            sender,
            localTokens,
            0,
            0
        );

        Token[] memory tokens = new Token[](1);
        tokens[0].denom = RelayLib.makeForeignDenom(
            destinationPort,
            destinationChannel,
            address(denomAddress).toHexString()
        );
        tokens[0].amount = amount;

        vm.expectEmit(false, false, false, false);
        emit RelayLib.Received("", address(0), "", address(0), 0);

        vm.prank(address(relay));
        relay.onRecvPacketProcessing(
            IbcCoreChannelV1Packet.Data({
                sequence: sequence,
                source_port: sourcePort,
                source_channel: sourceChannel,
                destination_port: destinationPort,
                destination_channel: destinationChannel,
                data: RelayPacketLib.encode(
                    RelayPacket({
                        sender: sender,
                        receiver: abi.encodePacked(receiver),
                        tokens: tokens
                    })
                ),
                timeout_height: IbcCoreClientV1Height.Data({
                    revision_number: timeoutRevisionNumber,
                    revision_height: timeoutRevisionHeight
                }),
                timeout_timestamp: timeoutTimestamp
            }),
            relayer
        );
    }

    function testRelay_onRecvPacket_remoteToken(
        uint64 sequence,
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel,
        uint64 timeoutRevisionNumber,
        uint64 timeoutRevisionHeight,
        uint64 timeoutTimestamp,
        bytes memory sender,
        address receiver,
        address relayer,
        string memory denomName,
        uint128 amount
    ) public {
        vm.assume(receiver != address(0));
        vm.assume(relayer != address(0));
        vm.assume(amount > 0);

        UCS01Relay relay = new UCS01Relay(ibcHandler);

        Token[] memory tokens = new Token[](1);
        tokens[0].denom = denomName;
        tokens[0].amount = amount;

        vm.expectEmit(false, false, false, false);
        emit RelayLib.DenomCreated("", address(0));

        vm.expectEmit(false, false, false, false);
        emit RelayLib.Received("", address(0), "", address(0), 0);

        vm.prank(address(relay));
        relay.onRecvPacketProcessing(
            IbcCoreChannelV1Packet.Data({
                sequence: sequence,
                source_port: sourcePort,
                source_channel: sourceChannel,
                destination_port: destinationPort,
                destination_channel: destinationChannel,
                data: RelayPacketLib.encode(
                    RelayPacket({
                        sender: sender,
                        receiver: abi.encodePacked(receiver),
                        tokens: tokens
                    })
                ),
                timeout_height: IbcCoreClientV1Height.Data({
                    revision_number: timeoutRevisionNumber,
                    revision_height: timeoutRevisionHeight
                }),
                timeout_timestamp: timeoutTimestamp
            }),
            relayer
        );
    }

    function testRelay_send_remote(
        uint64 sequence,
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel,
        uint64 timeoutRevisionNumber,
        uint64 timeoutRevisionHeight,
        uint64 timeoutTimestamp,
        bytes memory sender,
        address receiver,
        address relayer,
        string memory denomName,
        uint128 amount
    ) public {
        vm.assume(receiver != address(0));
        vm.assume(relayer != address(0));
        vm.assume(amount > 0);

        UCS01Relay relay = new UCS01Relay(ibcHandler);

        {
            Token[] memory tokens = new Token[](1);
            tokens[0].denom = denomName;
            tokens[0].amount = amount;

            vm.expectEmit(false, false, false, false);
            emit RelayLib.DenomCreated("", address(0));

            vm.expectEmit(false, false, false, false);
            emit IERC20.Transfer(address(0), address(0), 0);

            vm.expectEmit(false, false, false, false);
            emit RelayLib.Received("", address(0), "", address(0), 0);

            vm.prank(address(relay));
            relay.onRecvPacketProcessing(
                IbcCoreChannelV1Packet.Data({
                    sequence: sequence,
                    source_port: sourcePort,
                    source_channel: sourceChannel,
                    destination_port: destinationPort,
                    destination_channel: destinationChannel,
                    data: RelayPacketLib.encode(
                        RelayPacket({
                            sender: sender,
                            receiver: abi.encodePacked(receiver),
                            tokens: tokens
                        })
                    ),
                    timeout_height: IbcCoreClientV1Height.Data({
                        revision_number: timeoutRevisionNumber,
                        revision_height: timeoutRevisionHeight
                    }),
                    timeout_timestamp: timeoutTimestamp
                }),
                relayer
            );
        }

        {
            address denomAddress = relay.getDenomAddress(
                RelayLib.makeForeignDenom(sourcePort, sourceChannel, denomName)
            );

            LocalToken[] memory localTokens = new LocalToken[](1);
            localTokens[0].denom = denomAddress;
            localTokens[0].amount = amount;

            vm.startPrank(receiver);
            IERC20Denom(denomAddress).approve(address(relay), amount);

            vm.expectEmit(false, false, false, false);
            emit RelayLib.Sent(address(0), "", "", address(0), 0);

            relay.send(
                destinationPort,
                destinationChannel,
                sender,
                localTokens,
                0,
                0
            );
        }
    }
}
