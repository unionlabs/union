pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "solidity-stringutils/strings.sol";
import "solady/utils/LibString.sol";
import "@openzeppelin/token/ERC20/IERC20.sol";
import "@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol";
import "../../../../../contracts/lib/Hex.sol";
import "../../../../../contracts/apps/Base.sol";
import "../../../../../contracts/apps/ucs/01-relay/Relay.sol";
import "../../../../../contracts/apps/ucs/01-relay/ERC20Denom.sol";
import "../../../../../contracts/apps/ucs/01-relay/IERC20Denom.sol";
import "../../../utils/IBCHandler_Testable.sol";
import {IBCHandler} from
    "../../../../../contracts/core/25-handler/IBCHandler.sol";
import {IBCConnection} from
    "../../../../../contracts/core/03-connection/IBCConnection.sol";
import {IBCClient} from "../../../../../contracts/core/02-client/IBCClient.sol";
import {IBCHeight} from "../../../../../contracts/core/02-client/IBCHeight.sol";
import {IBCChannelHandshake} from
    "../../../../../contracts/core/04-channel/IBCChannelHandshake.sol";
import {IIBCPacket} from
    "../../../../../contracts/core/04-channel/IIBCPacket.sol";
import {IBCPacket} from "../../../../../contracts/core/04-channel/IBCPacket.sol";

contract IBCHandlerFake is IBCHandler {
    using LibString for *;

    IbcCoreChannelV1Packet.Data[] packets;
    uint64 sequence;

    function sendPacket(
        string calldata sourceChannel,
        IbcCoreClientV1Height.Data calldata timeoutHeight,
        uint64 timeoutTimestamp,
        bytes calldata data
    ) external override returns (uint64) {
        uint64 packetSequence = sequence;
        sequence++;
        packets.push(
            IbcCoreChannelV1Packet.Data({
                sequence: packetSequence,
                source_port: msg.sender.toHexString(),
                source_channel: sourceChannel,
                destination_port: "dummy-port",
                destination_channel: "dummy-channel",
                data: data,
                timeout_height: timeoutHeight,
                timeout_timestamp: timeoutTimestamp
            })
        );
        return packetSequence;
    }

    function lastPacket()
        public
        view
        returns (IbcCoreChannelV1Packet.Data memory)
    {
        return packets[packets.length - 1];
    }
}

contract RelayTests is Test {
    using LibString for *;
    using strings for *;

    IBCHandlerFake ibcHandler;
    IRelay relay;

    function setUp() public {
        ibcHandler = IBCHandlerFake(
            address(
                new ERC1967Proxy(
                    address(new IBCHandlerFake()),
                    abi.encodeCall(
                        IBCHandler.initialize,
                        (
                            address(new IBCClient()),
                            address(new IBCConnection()),
                            address(new IBCChannelHandshake()),
                            address(new IBCPacket()),
                            address(this)
                        )
                    )
                )
            )
        );
        relay = IRelay(
            address(
                new ERC1967Proxy(
                    address(new UCS01Relay()),
                    abi.encodeCall(
                        UCS01Relay.initialize, (ibcHandler, address(this))
                    )
                )
            )
        );
    }

    function initChannel(
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel
    ) public {
        vm.prank(address(ibcHandler));
        relay.onChanOpenTry(
            IbcCoreChannelV1GlobalEnums.Order.ORDER_UNORDERED,
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
    }

    function sendLocalToken(
        string memory,
        string memory sourceChannel,
        address sender,
        bytes memory receiver,
        string memory denomName,
        uint128 amount,
        string memory extension
    ) public returns (address) {
        address denomAddress = address(new ERC20Denom(denomName));
        IERC20Denom(denomAddress).mint(address(sender), amount);

        vm.prank(sender);
        IERC20Denom(denomAddress).approve(address(relay), amount);

        LocalToken[] memory localTokens = new LocalToken[](1);
        localTokens[0].denom = denomAddress;
        localTokens[0].amount = amount;

        vm.expectEmit();
        emit IERC20.Transfer(address(sender), address(relay), amount);

        vm.expectEmit(false, false, false, false);
        emit RelayLib.Sent(0, sourceChannel, address(0), "", "", address(0), 0);

        vm.prank(sender);
        relay.send(
            sourceChannel, receiver, localTokens, extension, IBCHeight.zero(), 0
        );

        return denomAddress;
    }

    function sendRemoteToken(
        string memory,
        string memory sourceChannel,
        bytes memory sender,
        address receiver,
        address denomAddress,
        uint128 amount,
        string memory extension
    ) public {
        vm.prank(receiver);
        IERC20Denom(denomAddress).approve(address(relay), amount);

        LocalToken[] memory localTokens = new LocalToken[](1);
        localTokens[0].denom = denomAddress;
        localTokens[0].amount = amount;

        // Burn from user to zero
        vm.expectEmit();
        emit IERC20.Transfer(address(receiver), address(0), amount);

        vm.expectEmit(false, false, false, false);
        emit RelayLib.Sent(0, sourceChannel, address(0), "", "", address(0), 0);

        vm.prank(receiver);
        relay.send(
            sourceChannel, sender, localTokens, extension, IBCHeight.zero(), 0
        );
    }

    function receiveRemoteToken(
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
        uint128 amount,
        string memory extension
    ) public {
        Token[] memory tokens = new Token[](1);
        tokens[0].denom = denomName;
        tokens[0].amount = amount;

        vm.expectEmit(false, false, false, false);
        emit RelayLib.DenomCreated(sequence, sourceChannel, "", address(0));

        vm.expectEmit(false, false, false, false);
        emit IERC20.Transfer(address(0), address(0), 0);

        vm.expectEmit(false, false, false, false);
        emit RelayLib.Received(
            sequence, sourceChannel, "", address(0), "", address(0), 0
        );

        vm.prank(address(ibcHandler));
        relay.onRecvPacket(
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
                        tokens: tokens,
                        extension: extension
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

    function test_isRemote_ok() public pure {
        assertEq(RelayLib.isFromChannel("a", "b", "a/b/X"), true);
        assertEq(RelayLib.isFromChannel("aa.bb", "c", "aa.bb/c/X"), true);
    }

    function test_isRemote_ko() public pure {
        assertEq(RelayLib.isFromChannel("a", "b", "b/b/X"), false);
        assertEq(RelayLib.isFromChannel("aa.bb", "c", "aa.b/c/X"), false);
    }

    function test_makeForeignDenom() public pure {
        assertEq(RelayLib.makeForeignDenom("a", "b", "BLA"), "a/b/BLA");
        assertEq(
            RelayLib.makeForeignDenom("wasm.xyz", "channel-1", "muno"),
            "wasm.xyz/channel-1/muno"
        );
    }

    function test_makeDenomPrefix() public pure {
        assertEq(RelayLib.makeDenomPrefix("a", "b"), "a/b/");
        assertEq(
            RelayLib.makeDenomPrefix("wasm.xyz", "channel-99"),
            "wasm.xyz/channel-99/"
        );
    }

    function test_hexToAddress(address addr) public pure {
        assertEq(Hex.hexToAddress(addr.toHexString()), addr);
    }

    function test_hexToUint256(uint256 v) public pure {
        assertEq(Hex.hexToUint256(v.toHexString()), v);
    }

    function test_openInit_onlyIBC(
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel
    ) public {
        vm.expectRevert(IBCAppLib.ErrNotIBC.selector);
        relay.onChanOpenInit(
            IbcCoreChannelV1GlobalEnums.Order.ORDER_UNORDERED,
            new string[](0),
            destinationPort,
            destinationChannel,
            IbcCoreChannelV1Counterparty.Data({
                port_id: sourcePort,
                channel_id: sourceChannel
            }),
            RelayLib.VERSION
        );
    }

    function test_openInit_wrongVersion(
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel
    ) public {
        vm.expectRevert(RelayLib.ErrInvalidProtocolVersion.selector);
        vm.prank(address(ibcHandler));
        relay.onChanOpenInit(
            IbcCoreChannelV1GlobalEnums.Order.ORDER_UNORDERED,
            new string[](0),
            destinationPort,
            destinationChannel,
            IbcCoreChannelV1Counterparty.Data({
                port_id: sourcePort,
                channel_id: sourceChannel
            }),
            "blabla"
        );
    }

    function test_openInit_wrongOrdering(
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel
    ) public {
        vm.expectRevert(RelayLib.ErrInvalidProtocolOrdering.selector);
        vm.prank(address(ibcHandler));
        relay.onChanOpenInit(
            IbcCoreChannelV1GlobalEnums.Order.ORDER_ORDERED,
            new string[](0),
            destinationPort,
            destinationChannel,
            IbcCoreChannelV1Counterparty.Data({
                port_id: sourcePort,
                channel_id: sourceChannel
            }),
            RelayLib.VERSION
        );
    }

    function test_openTry_onlyIBC(
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel
    ) public {
        vm.expectRevert(IBCAppLib.ErrNotIBC.selector);
        relay.onChanOpenTry(
            IbcCoreChannelV1GlobalEnums.Order.ORDER_UNORDERED,
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
    }

    function test_openTry_wrongVersion(
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel
    ) public {
        vm.expectRevert(RelayLib.ErrInvalidProtocolVersion.selector);
        vm.prank(address(ibcHandler));
        relay.onChanOpenTry(
            IbcCoreChannelV1GlobalEnums.Order.ORDER_UNORDERED,
            new string[](0),
            destinationPort,
            destinationChannel,
            IbcCoreChannelV1Counterparty.Data({
                port_id: sourcePort,
                channel_id: sourceChannel
            }),
            "0xDEADC0DE",
            RelayLib.VERSION
        );
    }

    function test_openTry_wrongOrdering(
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel
    ) public {
        vm.expectRevert(RelayLib.ErrInvalidProtocolOrdering.selector);
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
    }

    function test_openTry_wrongCounterpartyVersion(
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel
    ) public {
        vm.expectRevert(RelayLib.ErrInvalidCounterpartyProtocolVersion.selector);
        vm.prank(address(ibcHandler));
        relay.onChanOpenTry(
            IbcCoreChannelV1GlobalEnums.Order.ORDER_UNORDERED,
            new string[](0),
            destinationPort,
            destinationChannel,
            IbcCoreChannelV1Counterparty.Data({
                port_id: sourcePort,
                channel_id: sourceChannel
            }),
            RelayLib.VERSION,
            "ok"
        );
    }

    function test_openAck_onlyIBC(
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel
    ) public {
        vm.expectRevert(IBCAppLib.ErrNotIBC.selector);
        relay.onChanOpenAck(
            destinationPort, destinationChannel, sourceChannel, RelayLib.VERSION
        );
    }

    function test_openAck_wrongVersion(
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel
    ) public {
        vm.expectRevert(RelayLib.ErrInvalidCounterpartyProtocolVersion.selector);
        vm.prank(address(ibcHandler));
        relay.onChanOpenAck(
            destinationPort, destinationChannel, sourceChannel, "ucs01version"
        );
    }

    function test_openConfirm_onlyIBC(
        string memory destinationPort,
        string memory destinationChannel
    ) public {
        vm.expectRevert(IBCAppLib.ErrNotIBC.selector);
        relay.onChanOpenConfirm(destinationPort, destinationChannel);
    }

    function test_openConfirm(
        string memory destinationPort,
        string memory destinationChannel
    ) public {
        vm.prank(address(ibcHandler));
        relay.onChanOpenConfirm(destinationPort, destinationChannel);
    }

    function test_closeInit_onlyIBC(
        string memory destinationPort,
        string memory destinationChannel
    ) public {
        vm.expectRevert(IBCAppLib.ErrNotIBC.selector);
        relay.onChanCloseInit(destinationPort, destinationChannel);
    }

    function test_closeInit_impossible(
        string memory destinationPort,
        string memory destinationChannel
    ) public {
        vm.expectRevert(RelayLib.ErrUnstoppable.selector);
        vm.prank(address(ibcHandler));
        relay.onChanCloseInit(destinationPort, destinationChannel);
    }

    function test_closeConfirm_onlyIBC(
        string memory destinationPort,
        string memory destinationChannel
    ) public {
        vm.expectRevert(IBCAppLib.ErrNotIBC.selector);
        relay.onChanCloseConfirm(destinationPort, destinationChannel);
    }

    function test_closeConfirm_impossible(
        string memory destinationPort,
        string memory destinationChannel
    ) public {
        vm.expectRevert(RelayLib.ErrUnstoppable.selector);
        vm.prank(address(ibcHandler));
        relay.onChanCloseConfirm(destinationPort, destinationChannel);
    }

    function test_onRecvPacketProcessing_onlySelf(
        address malicious,
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
        vm.assume(malicious != address(relay));
        vm.expectRevert(RelayLib.ErrUnauthorized.selector);
        vm.prank(address(malicious));
        UCS01Relay(address(relay)).onRecvPacketProcessing(
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

    function test_onRecvPacket_onlyIBC(
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
        vm.record();
        vm.expectRevert(IBCAppLib.ErrNotIBC.selector);
        relay.onRecvPacket(
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

    function test_onRecvPacket_revertProcessing_noop(
        uint64 sequence,
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel,
        address denom,
        address sender,
        uint128 amount,
        uint64 timeoutRevisionNumber,
        uint64 timeoutRevisionHeight,
        uint64 timeoutTimestamp,
        address relayer,
        string memory extension
    ) public {
        vm.assume(sender != address(0));
        vm.assume(denom != address(0));
        vm.assume(amount > 0);

        // Receive a token that hasn't been escrowed
        Token[] memory tokens = new Token[](1);
        tokens[0].denom = RelayLib.makeForeignDenom(
            sourcePort, sourceChannel, denom.toHexString()
        );
        tokens[0].amount = amount;

        vm.record();
        vm.prank(address(ibcHandler));
        bytes memory acknowledgement = relay.onRecvPacket(
            IbcCoreChannelV1Packet.Data({
                sequence: sequence,
                source_port: sourcePort,
                source_channel: sourceChannel,
                destination_port: destinationPort,
                destination_channel: destinationChannel,
                data: RelayPacketLib.encode(
                    RelayPacket({
                        sender: abi.encodePacked(sender),
                        receiver: abi.encodePacked(sender),
                        tokens: tokens,
                        extension: extension
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

        assertEq(acknowledgement, abi.encodePacked(RelayLib.ACK_FAILURE));
        (, bytes32[] memory writes) = vm.accesses(address(relay));
        assertEq(writes.length, 0);
    }

    struct ReceiveLocalToken {
        uint64 sequence;
        string sourcePort;
        string sourceChannel;
        string destinationPort;
        string destinationChannel;
        uint64 timeoutRevisionNumber;
        uint64 timeoutRevisionHeight;
        uint64 timeoutTimestamp;
        address sender;
        bytes receiver;
        address relayer;
        string denomName;
        uint128 amount;
        string extension;
    }

    function test_receive_localToken(ReceiveLocalToken memory args) public {
        vm.assume(args.sender != address(0));
        vm.assume(args.relayer != address(0));
        vm.assume(args.amount > 0);

        initChannel(
            args.sourcePort,
            args.sourceChannel,
            args.destinationPort,
            args.destinationChannel
        );

        address denomAddress = address(new ERC20Denom(args.denomName));
        IERC20Denom(denomAddress).mint(address(args.sender), args.amount);

        LocalToken[] memory localTokens = new LocalToken[](1);
        localTokens[0].denom = denomAddress;
        localTokens[0].amount = args.amount;

        vm.prank(args.sender);
        IERC20Denom(denomAddress).approve(address(relay), args.amount);

        // A single transfer without mint as the token was previously escrowed
        vm.expectEmit();
        emit IERC20.Transfer(address(args.sender), address(relay), args.amount);

        vm.expectEmit(false, false, false, false);
        emit RelayLib.Sent(
            args.sequence, args.sourceChannel, address(0), "", "", address(0), 0
        );

        vm.prank(args.sender);
        relay.send(
            args.destinationChannel,
            args.receiver,
            localTokens,
            args.extension,
            IBCHeight.zero(),
            0
        );

        Token[] memory tokens = new Token[](1);
        tokens[0].denom = RelayLib.makeForeignDenom(
            args.sourcePort, args.sourceChannel, denomAddress.toHexString()
        );
        tokens[0].amount = args.amount;

        // A single transfer without mint as the token was previously escrowed
        vm.expectEmit(false, false, false, false);
        emit IERC20.Transfer(address(0), address(args.sender), args.amount);

        vm.expectEmit(false, false, false, false);
        emit RelayLib.Received(
            args.sequence, args.sourceChannel, "", address(0), "", address(0), 0
        );

        uint256 outstandingBefore =
            relay.getOutstanding(args.destinationChannel, denomAddress);

        vm.prank(address(ibcHandler));
        relay.onRecvPacket(
            IbcCoreChannelV1Packet.Data({
                sequence: args.sequence,
                source_port: args.sourcePort,
                source_channel: args.sourceChannel,
                destination_port: args.destinationPort,
                destination_channel: args.destinationChannel,
                data: RelayPacketLib.encode(
                    RelayPacket({
                        sender: args.receiver,
                        receiver: abi.encodePacked(args.sender),
                        tokens: tokens,
                        extension: args.extension
                    })
                    ),
                timeout_height: IbcCoreClientV1Height.Data({
                    revision_number: args.timeoutRevisionNumber,
                    revision_height: args.timeoutRevisionHeight
                }),
                timeout_timestamp: args.timeoutTimestamp
            }),
            args.relayer
        );

        // Local tokens are tracked, outstanding for the channel must be diminished by the amount
        assertEq(
            relay.getOutstanding(args.destinationChannel, denomAddress)
                + args.amount,
            outstandingBefore
        );
    }

    struct ReceiveRemoteToken {
        uint64 sequence;
        string sourcePort;
        string sourceChannel;
        string destinationPort;
        string destinationChannel;
        uint64 timeoutRevisionNumber;
        uint64 timeoutRevisionHeight;
        uint64 timeoutTimestamp;
        bytes sender;
        address receiver;
        address relayer;
        string denomName;
        uint128 amount;
        string extension;
    }

    function test_receive_remoteToken(ReceiveRemoteToken memory args) public {
        vm.assume(args.receiver != address(0));
        vm.assume(args.relayer != address(0));
        vm.assume(args.amount > 0);

        initChannel(
            args.sourcePort,
            args.sourceChannel,
            args.destinationPort,
            args.destinationChannel
        );

        receiveRemoteToken(
            args.sequence,
            args.sourcePort,
            args.sourceChannel,
            args.destinationPort,
            args.destinationChannel,
            args.timeoutRevisionNumber,
            args.timeoutRevisionHeight,
            args.timeoutTimestamp,
            args.sender,
            args.receiver,
            args.relayer,
            args.denomName,
            args.amount,
            args.extension
        );
    }

    function test_send_local(
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel,
        address sender,
        bytes memory receiver,
        string memory denomName,
        uint128 amount,
        string memory extension
    ) public {
        vm.assume(sender != address(0));
        vm.assume(amount > 0);

        initChannel(
            sourcePort, sourceChannel, destinationPort, destinationChannel
        );

        address denomAddress = address(new ERC20Denom(denomName));
        IERC20Denom(denomAddress).mint(sender, amount);

        LocalToken[] memory localTokens = new LocalToken[](1);
        localTokens[0].denom = denomAddress;
        localTokens[0].amount = amount;

        vm.prank(sender);
        IERC20Denom(denomAddress).approve(address(relay), amount);

        assertEq(relay.getOutstanding(destinationChannel, denomAddress), 0);

        vm.expectEmit();
        emit IERC20.Transfer(address(sender), address(relay), amount);

        vm.expectEmit(false, false, false, false);
        emit RelayLib.Sent(0, sourceChannel, address(0), "", "", address(0), 0);

        vm.prank(sender);
        relay.send(
            destinationChannel,
            receiver,
            localTokens,
            extension,
            IBCHeight.zero(),
            0
        );

        // Local tokens must be tracked as outstanding for the channel
        assertEq(relay.getOutstanding(destinationChannel, denomAddress), amount);
    }

    function test_send_remote(
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
        uint128 amount,
        string memory extension
    ) public {
        vm.assume(receiver != address(0));
        vm.assume(relayer != address(0));
        vm.assume(amount > 0);

        initChannel(
            sourcePort, sourceChannel, destinationPort, destinationChannel
        );

        receiveRemoteToken(
            sequence,
            sourcePort,
            sourceChannel,
            destinationPort,
            destinationChannel,
            timeoutRevisionNumber,
            timeoutRevisionHeight,
            timeoutTimestamp,
            sender,
            receiver,
            relayer,
            denomName,
            amount,
            extension
        );

        address denomAddress = relay.getDenomAddress(
            destinationChannel,
            RelayLib.makeForeignDenom(
                destinationPort, destinationChannel, denomName
            )
        );

        LocalToken[] memory localTokens = new LocalToken[](1);
        localTokens[0].denom = denomAddress;
        localTokens[0].amount = amount;

        vm.prank(receiver);
        IERC20Denom(denomAddress).approve(address(relay), amount);

        // Burn from sender to zero
        vm.expectEmit();
        emit IERC20.Transfer(address(receiver), address(0), amount);

        vm.expectEmit(false, false, false, false);
        emit RelayLib.Sent(
            sequence, sourceChannel, address(0), "", "", address(0), 0
        );

        uint256 outstandingBefore =
            relay.getOutstanding(destinationChannel, denomAddress);

        vm.prank(receiver);
        relay.send(
            destinationChannel,
            abi.encodePacked(receiver),
            localTokens,
            extension,
            IBCHeight.zero(),
            0
        );

        uint256 outstandingAfter =
            relay.getOutstanding(destinationChannel, denomAddress);

        // Remote tokens are not tracked as outstanding
        assertEq(outstandingBefore, outstandingAfter);
    }

    function test_send_local_from_remote(
        uint64 sequence,
        string memory destinationPort,
        string memory sourcePort,
        string memory sourceChannel,
        uint64 timeoutRevisionNumber,
        uint64 timeoutRevisionHeight,
        uint64 timeoutTimestamp,
        bytes memory sender,
        address receiver,
        address relayer,
        string memory denomName,
        uint128 amount,
        string memory extension
    ) public {
        vm.assume(sequence < 1000000000);
        vm.assume(receiver != address(0));
        vm.assume(relayer != address(0));
        vm.assume(amount > 0);

        // Open two different local channels with the same counterparty
        initChannel(sourcePort, sourceChannel, destinationPort, "channel-1");
        initChannel(sourcePort, sourceChannel, destinationPort, "channel-2");

        receiveRemoteToken(
            sequence,
            sourcePort,
            sourceChannel,
            destinationPort,
            "channel-1",
            timeoutRevisionNumber,
            timeoutRevisionHeight,
            timeoutTimestamp,
            sender,
            receiver,
            relayer,
            denomName,
            amount,
            extension
        );

        receiveRemoteToken(
            sequence + 1,
            sourcePort,
            sourceChannel,
            destinationPort,
            "channel-2",
            timeoutRevisionNumber,
            timeoutRevisionHeight,
            timeoutTimestamp,
            sender,
            receiver,
            relayer,
            denomName,
            amount,
            extension
        );

        {
            address denomAddress = relay.getDenomAddress(
                "channel-1",
                RelayLib.makeForeignDenom(
                    destinationPort, "channel-1", denomName
                )
            );

            LocalToken[] memory localTokens = new LocalToken[](1);
            localTokens[0].denom = denomAddress;
            localTokens[0].amount = amount;

            vm.prank(receiver);
            IERC20Denom(denomAddress).approve(address(relay), amount);

            uint256 outstandingBefore =
                relay.getOutstanding("channel-2", denomAddress);

            vm.expectEmit();
            emit IERC20.Transfer(address(receiver), address(relay), amount);

            vm.expectEmit(false, false, false, false);
            emit RelayLib.Sent(
                sequence, sourceChannel, address(0), "", "", address(0), 0
            );

            vm.prank(receiver);
            relay.send(
                "channel-2",
                abi.encodePacked(receiver),
                localTokens,
                extension,
                IBCHeight.zero(),
                0
            );

            uint256 outstandingAfter =
                relay.getOutstanding("channel-2", denomAddress);

            // Remote tokens are not tracked as outstanding
            assertEq(outstandingBefore + amount, outstandingAfter);
        }
    }

    struct NoCollision {
        uint64 sequence;
        string destinationPort;
        string sourcePort;
        string sourceChannel;
        uint64 timeoutRevisionNumber;
        uint64 timeoutRevisionHeight;
        uint64 timeoutTimestamp;
        bytes sender;
        address receiver;
        address relayer;
        string denomName;
        uint128 amount;
        string extension;
    }

    function test_receive_remote_no_collision(NoCollision calldata args)
        public
    {
        vm.assume(args.sequence < 1000000000);
        vm.assume(args.receiver != address(0));
        vm.assume(args.relayer != address(0));
        vm.assume(args.amount > 0);

        // Open two different local channels with the same counterparty
        initChannel(
            args.sourcePort,
            args.sourceChannel,
            args.destinationPort,
            "channel-1"
        );
        initChannel(
            args.sourcePort,
            args.sourceChannel,
            args.destinationPort,
            "channel-2"
        );

        receiveRemoteToken(
            args.sequence,
            args.sourcePort,
            args.sourceChannel,
            args.destinationPort,
            "channel-1",
            args.timeoutRevisionNumber,
            args.timeoutRevisionHeight,
            args.timeoutTimestamp,
            args.sender,
            args.receiver,
            args.relayer,
            args.denomName,
            args.amount,
            args.extension
        );

        receiveRemoteToken(
            args.sequence + 1,
            args.sourcePort,
            args.sourceChannel,
            args.destinationPort,
            "channel-2",
            args.timeoutRevisionNumber,
            args.timeoutRevisionHeight,
            args.timeoutTimestamp,
            args.sender,
            args.receiver,
            args.relayer,
            args.denomName,
            args.amount,
            args.extension
        );
    }

    function test_onTimeout_onlyIBC(
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel,
        address sender,
        bytes memory receiver,
        address relayer,
        string memory denomName,
        uint128 amount,
        string memory extension
    ) public {
        vm.assume(sender != address(0));
        vm.assume(relayer != address(0));
        vm.assume(amount > 0);

        initChannel(
            sourcePort, sourceChannel, destinationPort, destinationChannel
        );

        sendLocalToken(
            destinationPort,
            destinationChannel,
            sender,
            receiver,
            denomName,
            amount,
            extension
        );

        IbcCoreChannelV1Packet.Data memory packet = ibcHandler.lastPacket();

        vm.expectRevert(IBCAppLib.ErrNotIBC.selector);
        relay.onTimeoutPacket(packet, relayer);
    }

    function test_onTimeout_refund_local(
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel,
        address sender,
        bytes memory receiver,
        address relayer,
        string memory denomName,
        uint128 amount,
        string memory extension
    ) public {
        vm.assume(sender != address(0));
        vm.assume(relayer != address(0));
        vm.assume(amount > 0);

        initChannel(
            sourcePort, sourceChannel, destinationPort, destinationChannel
        );

        address denomAddress = sendLocalToken(
            destinationPort,
            destinationChannel,
            sender,
            receiver,
            denomName,
            amount,
            extension
        );

        IbcCoreChannelV1Packet.Data memory packet = ibcHandler.lastPacket();

        vm.expectEmit();
        emit IERC20.Transfer(address(relay), address(sender), amount);

        vm.expectEmit(false, false, false, false);
        emit RelayLib.Refunded(
            0, sourceChannel, address(0), "", "", address(this), 0
        );

        assertEq(relay.getOutstanding(destinationChannel, denomAddress), amount);

        vm.prank(address(ibcHandler));
        relay.onTimeoutPacket(packet, relayer);

        /* Tokens must be unescrowed and no longer outstanding */
        assertEq(relay.getOutstanding(destinationChannel, denomAddress), 0);
    }

    struct OnTimeoutRefundRemote {
        uint64 sequence;
        string sourcePort;
        string sourceChannel;
        string destinationPort;
        string destinationChannel;
        uint64 timeoutRevisionNumber;
        uint64 timeoutRevisionHeight;
        uint64 timeoutTimestamp;
        bytes sender;
        address receiver;
        address relayer;
        string denomName;
        uint128 amount;
        string extension;
    }

    function test_onTimeout_refund_remote(OnTimeoutRefundRemote memory args)
        public
    {
        vm.assume(
            !RelayLib.isFromChannel(
                args.destinationPort, args.destinationChannel, args.denomName
            )
        );
        vm.assume(args.receiver != address(0));
        vm.assume(args.relayer != address(0));
        vm.assume(args.amount > 0);

        initChannel(
            args.sourcePort,
            args.sourceChannel,
            args.destinationPort,
            args.destinationChannel
        );

        receiveRemoteToken(
            args.sequence,
            args.sourcePort,
            args.sourceChannel,
            args.destinationPort,
            args.destinationChannel,
            args.timeoutRevisionNumber,
            args.timeoutRevisionHeight,
            args.timeoutTimestamp,
            args.sender,
            args.receiver,
            args.relayer,
            args.denomName,
            args.amount,
            args.extension
        );

        address denomAddress = relay.getDenomAddress(
            args.destinationChannel,
            RelayLib.makeForeignDenom(
                args.destinationPort, args.destinationChannel, args.denomName
            )
        );

        sendRemoteToken(
            args.destinationPort,
            args.destinationChannel,
            args.sender,
            args.receiver,
            denomAddress,
            args.amount,
            args.extension
        );

        IbcCoreChannelV1Packet.Data memory packet = ibcHandler.lastPacket();

        vm.expectEmit();
        emit IERC20.Transfer(address(0), address(args.receiver), args.amount);

        vm.expectEmit(false, false, false, false);
        emit RelayLib.Refunded(
            args.sequence,
            args.sourceChannel,
            address(0),
            "",
            "",
            address(this),
            0
        );

        uint256 outstandingBefore =
            relay.getOutstanding(args.destinationChannel, denomAddress);

        vm.prank(address(ibcHandler));
        relay.onTimeoutPacket(packet, args.relayer);

        // Outstanding must not be touched
        assertEq(
            relay.getOutstanding(args.destinationChannel, denomAddress),
            outstandingBefore
        );
    }

    function test_ack_failure_refund_local(
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel,
        address sender,
        bytes memory receiver,
        address relayer,
        string memory denomName,
        uint128 amount,
        string memory extension
    ) public {
        vm.assume(sender != address(0));
        vm.assume(relayer != address(0));
        vm.assume(amount > 0);

        initChannel(
            sourcePort, sourceChannel, destinationPort, destinationChannel
        );

        address denomAddress = sendLocalToken(
            destinationPort,
            destinationChannel,
            sender,
            receiver,
            denomName,
            amount,
            extension
        );

        IbcCoreChannelV1Packet.Data memory packet = ibcHandler.lastPacket();

        vm.expectEmit();
        emit IERC20.Transfer(address(relay), address(sender), amount);

        vm.expectEmit(false, false, false, false);
        emit RelayLib.Refunded(
            0, sourceChannel, address(0), "", "", address(this), 0
        );

        assertEq(relay.getOutstanding(destinationChannel, denomAddress), amount);

        vm.prank(address(ibcHandler));
        relay.onAcknowledgementPacket(
            packet, abi.encodePacked(RelayLib.ACK_FAILURE), relayer
        );

        /* Tokens must be unescrowed and no longer outstanding */
        assertEq(relay.getOutstanding(destinationChannel, denomAddress), 0);
    }

    struct AckFailureRefundRemote {
        uint64 sequence;
        string sourcePort;
        string sourceChannel;
        string destinationPort;
        string destinationChannel;
        uint64 timeoutRevisionNumber;
        uint64 timeoutRevisionHeight;
        uint64 timeoutTimestamp;
        bytes sender;
        address receiver;
        address relayer;
        string denomName;
        uint128 amount;
        string extension;
    }

    function test_ack_failure_refund_remote(AckFailureRefundRemote memory args)
        public
    {
        vm.assume(
            !RelayLib.isFromChannel(
                args.destinationPort, args.destinationChannel, args.denomName
            )
        );
        vm.assume(args.receiver != address(0));
        vm.assume(args.relayer != address(0));
        vm.assume(args.amount > 0);

        initChannel(
            args.sourcePort,
            args.sourceChannel,
            args.destinationPort,
            args.destinationChannel
        );

        receiveRemoteToken(
            args.sequence,
            args.sourcePort,
            args.sourceChannel,
            args.destinationPort,
            args.destinationChannel,
            args.timeoutRevisionNumber,
            args.timeoutRevisionHeight,
            args.timeoutTimestamp,
            args.sender,
            args.receiver,
            args.relayer,
            args.denomName,
            args.amount,
            args.extension
        );

        address denomAddress = relay.getDenomAddress(
            args.destinationChannel,
            RelayLib.makeForeignDenom(
                args.destinationPort, args.destinationChannel, args.denomName
            )
        );

        sendRemoteToken(
            args.destinationPort,
            args.destinationChannel,
            args.sender,
            args.receiver,
            denomAddress,
            args.amount,
            args.extension
        );

        IbcCoreChannelV1Packet.Data memory packet = ibcHandler.lastPacket();

        vm.expectEmit();
        emit IERC20.Transfer(address(0), address(args.receiver), args.amount);

        vm.expectEmit(false, false, false, false);
        emit RelayLib.Refunded(
            args.sequence,
            args.sourceChannel,
            address(0),
            "",
            "",
            address(this),
            0
        );

        uint256 outstandingBefore =
            relay.getOutstanding(args.destinationChannel, denomAddress);

        vm.prank(address(ibcHandler));
        relay.onAcknowledgementPacket(
            packet, abi.encodePacked(RelayLib.ACK_FAILURE), args.relayer
        );

        // Outstanding must not be touched
        assertEq(
            relay.getOutstanding(args.destinationChannel, denomAddress),
            outstandingBefore
        );
    }

    function test_ack_success_noop_local(
        string memory sourcePort,
        string memory sourceChannel,
        string memory destinationPort,
        string memory destinationChannel,
        address sender,
        bytes memory receiver,
        address relayer,
        string memory denomName,
        uint128 amount,
        string memory extension
    ) public {
        vm.assume(sender != address(0));
        vm.assume(relayer != address(0));
        vm.assume(amount > 0);

        initChannel(
            sourcePort, sourceChannel, destinationPort, destinationChannel
        );

        sendLocalToken(
            destinationPort,
            destinationChannel,
            sender,
            receiver,
            denomName,
            amount,
            extension
        );

        IbcCoreChannelV1Packet.Data memory packet = ibcHandler.lastPacket();

        vm.record();

        vm.prank(address(ibcHandler));
        relay.onAcknowledgementPacket(
            packet, abi.encodePacked(RelayLib.ACK_SUCCESS), relayer
        );

        (, bytes32[] memory writes) = vm.accesses(address(relay));
        assertEq(writes.length, 0);
    }

    function test_ack_success_noop_remote(
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
        uint128 amount,
        string memory extension
    ) public {
        vm.assume(receiver != address(0));
        vm.assume(relayer != address(0));
        vm.assume(amount > 0);

        initChannel(
            sourcePort, sourceChannel, destinationPort, destinationChannel
        );

        receiveRemoteToken(
            sequence,
            sourcePort,
            sourceChannel,
            destinationPort,
            destinationChannel,
            timeoutRevisionNumber,
            timeoutRevisionHeight,
            timeoutTimestamp,
            sender,
            receiver,
            relayer,
            denomName,
            amount,
            extension
        );

        address denomAddress = relay.getDenomAddress(
            destinationChannel,
            RelayLib.makeForeignDenom(
                destinationPort, destinationChannel, denomName
            )
        );

        sendRemoteToken(
            destinationPort,
            destinationChannel,
            sender,
            receiver,
            denomAddress,
            amount,
            extension
        );

        IbcCoreChannelV1Packet.Data memory packet = ibcHandler.lastPacket();

        vm.record();

        vm.prank(address(ibcHandler));
        relay.onAcknowledgementPacket(
            packet, abi.encodePacked(RelayLib.ACK_SUCCESS), relayer
        );

        (, bytes32[] memory writes) = vm.accesses(address(relay));
        assertEq(writes.length, 0);
    }
}
