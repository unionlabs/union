pragma solidity ^0.8.27;

import "forge-std/Test.sol";

import "../core/IBCHandler.sol";
import "../core/LightClient.sol";
import "../core/Module.sol";

contract IBCPacketTests is Test {
    bytes32 public constant CLIENT_TYPE = keccak256("zkgm");
    bytes32 public constant VERSION = keccak256("protocol-1");
    bytes32 public constant COUNTERPARTY_PORT_ID =
        keccak256(abi.encodePacked(address(0)));
    uint32 public constant COUNTERPARTY_CHANNEL_ID = 0xDEADC0DE;

    TestIBCHandler handler;
    TestLightClient lightClient;
    TestModule module;

    uint32 clientId;
    uint32 connectionId;
    uint32 channelId;

    function setUp() public {
        handler = new TestIBCHandler();
        lightClient = new TestLightClient();
        module = new TestModule(handler);

        // Create client
        handler.registerClient(CLIENT_TYPE, lightClient);
        clientId = handler.createClient(
            IBCMsgs.MsgCreateClient({
                clientType: CLIENT_TYPE,
                clientStateBytes: hex"CADEBABE",
                consensusStateBytes: hex"DEADC0DE",
                relayer: address(this)
            })
        );

        // Create connection
        IBCMsgs.MsgConnectionOpenTry memory msgTry_ = IBCMsgs
            .MsgConnectionOpenTry({
            counterparty: IBCConnectionCounterparty({
                clientId: 0xDEADC0DE,
                connectionId: 0xCAFE,
                merklePrefix: keccak256("root")
            }),
            clientId: clientId,
            proofInit: hex"",
            proofHeight: 0,
            relayer: address(this)
        });
        lightClient.pushValidMembership();
        connectionId = handler.connectionOpenTry(msgTry_);
        IBCMsgs.MsgConnectionOpenConfirm memory msgConfirm_ = IBCMsgs
            .MsgConnectionOpenConfirm({
            connectionId: connectionId,
            proofAck: hex"",
            proofHeight: 0,
            relayer: address(this)
        });
        lightClient.pushValidMembership();
        handler.connectionOpenConfirm(msgConfirm_);

        // Create channel
        IBCMsgs.MsgChannelOpenInit memory msgInit_ = IBCMsgs.MsgChannelOpenInit({
            portId: address(module),
            channel: IBCChannel({
                state: IBCChannelState.Init,
                connectionId: connectionId,
                ordering: IBCChannelOrder.Unordered,
                version: VERSION,
                counterparty: IBCChannelCounterparty({
                    portId: COUNTERPARTY_PORT_ID,
                    channelId: COUNTERPARTY_CHANNEL_ID
                })
            }),
            relayer: address(this)
        });
        channelId = handler.channelOpenInit(msgInit_);
        IBCMsgs.MsgChannelOpenAck memory msgAck_ = IBCMsgs.MsgChannelOpenAck({
            portId: address(module),
            channelId: channelId,
            counterpartyVersion: VERSION,
            counterpartyChannelId: COUNTERPARTY_CHANNEL_ID,
            proofTry: hex"",
            proofHeight: 0,
            relayer: address(this)
        });
        lightClient.pushValidMembership();
        handler.channelOpenAck(msgAck_);
    }

    function test_sendPacket_ok(
        uint64 timeoutTimestamp,
        uint64 timeoutHeight,
        bytes calldata packet
    ) public {
        vm.assume(timeoutTimestamp != 0 || timeoutHeight != 0);
        vm.pauseGasMetering();
        vm.prank(address(module));
        vm.resumeGasMetering();
        handler.sendPacket(channelId, timeoutTimestamp, timeoutHeight, packet);
    }

    function test_sendPacket_increaseSequence(
        uint64 timeoutTimestamp,
        uint64 timeoutHeight,
        bytes calldata packet
    ) public {
        vm.assume(timeoutTimestamp != 0 || timeoutHeight != 0);
        vm.prank(address(module));
        uint64 sequence = handler.sendPacket(
            channelId, timeoutTimestamp, timeoutHeight, packet
        );
        assertEq(
            handler.commitments(
                IBCCommitment.nextSequenceSendCommitmentKey(channelId)
            ),
            bytes32(uint256(sequence + 1))
        );
    }

    function test_sendPacket_commitmentSaved(
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes calldata message
    ) public {
        vm.assume(timeoutTimestamp != 0 || timeoutHeight != 0);
        vm.prank(address(module));
        uint64 sequence = handler.sendPacket(
            channelId, timeoutHeight, timeoutTimestamp, message
        );
        bytes32 normalizedSourcePort =
            keccak256(abi.encodePacked(address(module)));
        IBCPacket memory packet = IBCPacket({
            sequence: sequence,
            sourcePort: normalizedSourcePort,
            sourceChannel: channelId,
            destinationPort: COUNTERPARTY_PORT_ID,
            destinationChannel: COUNTERPARTY_CHANNEL_ID,
            data: message,
            timeoutHeight: timeoutHeight,
            timeoutTimestamp: timeoutTimestamp
        });
        assertEq(
            handler.commitments(
                IBCCommitment.batchPacketsCommitmentKey(
                    channelId, IBCPacketLib.commitPacketMemory(packet)
                )
            ),
            IBCPacketLib.COMMITMENT_MAGIC
        );
    }

    function test_sendPacket_missingTimeout(
        bytes calldata packet
    ) public {
        vm.pauseGasMetering();
        vm.prank(address(module));
        vm.expectRevert(IBCPacketLib.ErrTimeoutMustBeSet.selector);
        vm.resumeGasMetering();
        handler.sendPacket(channelId, 0, 0, packet);
    }

    function test_sendPacket_absentChannel(
        uint32 channelId_,
        uint64 timeoutTimestamp,
        uint64 timeoutHeight,
        bytes calldata packet
    ) public {
        vm.pauseGasMetering();
        vm.assume(channelId_ != channelId);
        vm.assume(timeoutTimestamp != 0 || timeoutHeight != 0);
        vm.expectRevert(IBCPacketLib.ErrUnauthorized.selector);
        vm.prank(address(module));
        vm.resumeGasMetering();
        handler.sendPacket(channelId_, timeoutTimestamp, timeoutHeight, packet);
    }

    function test_sendPacket_notOwner(
        uint64 timeoutTimestamp,
        uint64 timeoutHeight,
        bytes calldata packet
    ) public {
        vm.pauseGasMetering();
        vm.assume(timeoutTimestamp != 0 || timeoutHeight != 0);
        vm.expectRevert(IBCPacketLib.ErrUnauthorized.selector);
        vm.resumeGasMetering();
        handler.sendPacket(channelId, timeoutTimestamp, timeoutHeight, packet);
    }

    function createReceivePacket(
        bytes32 sourcePort,
        uint32 sourceChannel,
        bytes calldata message,
        uint8 nbPackets
    ) internal view returns (IBCMsgs.MsgPacketRecv memory) {
        bytes32 normalizedPortId = keccak256(abi.encodePacked(address(module)));
        IBCPacket[] memory packets = new IBCPacket[](nbPackets);
        bytes[] memory relayerMsgs = new bytes[](nbPackets);
        for (uint8 i = 0; i < nbPackets; i++) {
            packets[i] = IBCPacket({
                sequence: i,
                sourcePort: sourcePort,
                sourceChannel: sourceChannel,
                destinationPort: normalizedPortId,
                destinationChannel: channelId,
                data: message,
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: type(uint64).max
            });
            relayerMsgs[i] = abi.encodePacked(i);
        }
        IBCMsgs.MsgPacketRecv memory msg_ = IBCMsgs.MsgPacketRecv({
            packets: packets,
            relayerMsgs: relayerMsgs,
            relayer: address(this),
            proof: hex"",
            proofHeight: 0
        });
        return msg_;
    }

    function test_recvPacket_ok_1(
        bytes32 sourcePort,
        uint32 sourceChannel,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_recvPacket_ok(sourcePort, sourceChannel, message, 1);
    }

    function test_recvPacket_ok(
        bytes32 sourcePort,
        uint32 sourceChannel,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.pauseGasMetering();
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourcePort, sourceChannel, message, nbPackets);
        lightClient.pushValidMembership();
        vm.resumeGasMetering();
        handler.recvPacket(msg_);
    }

    function test_recvPacket_invalidProof(
        bytes32 sourcePort,
        uint32 sourceChannel,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourcePort, sourceChannel, message, nbPackets);
        vm.expectRevert(IBCPacketLib.ErrInvalidProof.selector);
        handler.recvPacket(msg_);
    }

    function test_recvPacket_invalidChannelState(
        bytes32 sourcePort,
        uint32 sourceChannel,
        uint32 destinationChannel,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(nbPackets > 0);
        vm.assume(destinationChannel != channelId);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourcePort, sourceChannel, message, nbPackets);
        // fake non existant channel
        msg_.packets[0].destinationChannel = destinationChannel;
        vm.expectRevert(IBCPacketLib.ErrInvalidChannelState.selector);
        handler.recvPacket(msg_);
    }

    function test_recvPacket_timeoutTimestamp(
        uint32 timeout,
        bytes32 sourcePort,
        uint32 sourceChannel,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(timeout > 0);
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourcePort, sourceChannel, message, nbPackets);
        // Timeout is expressed as nano because of ibc-go...
        msg_.packets[0].timeoutTimestamp = uint64(timeout) * 1e9;
        vm.warp(timeout);
        lightClient.pushValidMembership();
        vm.expectRevert(IBCPacketLib.ErrTimestampTimeout.selector);
        handler.recvPacket(msg_);
    }

    function test_recvPacket_timeoutHeight(
        uint64 timeout,
        bytes32 sourcePort,
        uint32 sourceChannel,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(timeout > 0);
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourcePort, sourceChannel, message, nbPackets);
        // fake non existant channel
        msg_.packets[0].timeoutHeight = timeout;
        vm.roll(timeout);
        lightClient.pushValidMembership();
        vm.expectRevert(IBCPacketLib.ErrHeightTimeout.selector);
        handler.recvPacket(msg_);
    }

    function test_recvPacket_commitmentSaved(
        bytes32 sourcePort,
        uint32 sourceChannel,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourcePort, sourceChannel, message, nbPackets);
        lightClient.pushValidMembership();
        handler.recvPacket(msg_);
        for (uint8 i = 0; i < nbPackets; i++) {
            assertEq(
                handler.commitments(
                    IBCCommitment.batchReceiptsCommitmentKey(
                        channelId,
                        IBCPacketLib.commitPacketMemory(msg_.packets[i])
                    )
                ),
                IBCPacketLib.commitAckMemory(TestModuleLib.ACKNOWLEDGEMENT)
            );
        }
    }

    function createReceiveIntentPacket(
        bytes32 sourcePort,
        uint32 sourceChannel,
        bytes calldata message,
        uint8 nbPackets
    ) internal view returns (IBCMsgs.MsgIntentPacketRecv memory) {
        bytes32 normalizedPortId = keccak256(abi.encodePacked(address(module)));
        IBCPacket[] memory packets = new IBCPacket[](nbPackets);
        bytes[] memory marketMakerMsgs = new bytes[](nbPackets);
        for (uint8 i = 0; i < nbPackets; i++) {
            packets[i] = IBCPacket({
                sequence: i,
                sourcePort: sourcePort,
                sourceChannel: sourceChannel,
                destinationPort: normalizedPortId,
                destinationChannel: channelId,
                data: message,
                timeoutHeight: type(uint64).max,
                timeoutTimestamp: type(uint64).max
            });
            marketMakerMsgs[i] = abi.encodePacked(i);
        }
        IBCMsgs.MsgIntentPacketRecv memory msg_ = IBCMsgs.MsgIntentPacketRecv({
            packets: packets,
            marketMakerMsgs: marketMakerMsgs,
            marketMaker: address(this),
            emptyProof: hex""
        });
        return msg_;
    }

    function test_recvIntentPacket_ok_1(
        bytes32 sourcePort,
        uint32 sourceChannel,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_recvIntentPacket_ok(sourcePort, sourceChannel, message, 1);
    }

    function test_recvIntentPacket_ok(
        bytes32 sourcePort,
        uint32 sourceChannel,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.pauseGasMetering();
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgIntentPacketRecv memory msg_ = createReceiveIntentPacket(
            sourcePort, sourceChannel, message, nbPackets
        );
        vm.resumeGasMetering();
        handler.recvIntentPacket(msg_);
    }

    function test_recvIntentPacket_commitmentSaved(
        bytes32 sourcePort,
        uint32 sourceChannel,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.pauseGasMetering();
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgIntentPacketRecv memory msg_ = createReceiveIntentPacket(
            sourcePort, sourceChannel, message, nbPackets
        );
        vm.resumeGasMetering();
        handler.recvIntentPacket(msg_);
        for (uint8 i = 0; i < nbPackets; i++) {
            assertEq(
                handler.commitments(
                    IBCCommitment.batchReceiptsCommitmentKey(
                        channelId,
                        IBCPacketLib.commitPacketMemory(msg_.packets[i])
                    )
                ),
                IBCPacketLib.commitAckMemory(TestModuleLib.ACKNOWLEDGEMENT)
            );
        }
    }

    function test_recvIntentPacket_timeoutTimestamp(
        uint32 timeout,
        bytes32 sourcePort,
        uint32 sourceChannel,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(timeout > 0);
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgIntentPacketRecv memory msg_ = createReceiveIntentPacket(
            sourcePort, sourceChannel, message, nbPackets
        );
        // Timeout is expressed as nano because of ibc-go...
        msg_.packets[0].timeoutTimestamp = uint64(timeout) * 1e9;
        vm.warp(timeout);
        vm.expectRevert(IBCPacketLib.ErrTimestampTimeout.selector);
        handler.recvIntentPacket(msg_);
    }

    function test_recvIntentPacket_timeoutHeight(
        uint64 timeout,
        bytes32 sourcePort,
        uint32 sourceChannel,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(timeout > 0);
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgIntentPacketRecv memory msg_ = createReceiveIntentPacket(
            sourcePort, sourceChannel, message, nbPackets
        );
        // Timeout is expressed as nano because of ibc-go...
        msg_.packets[0].timeoutHeight = timeout;
        vm.roll(timeout);
        vm.expectRevert(IBCPacketLib.ErrHeightTimeout.selector);
        handler.recvIntentPacket(msg_);
    }
}
