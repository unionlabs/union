pragma solidity ^0.8.27;

import "forge-std/Test.sol";

import "@openzeppelin/contracts/utils/math/Math.sol";

import "../core/UnionTests.sol";
import "../core/IBCHandler.sol";
import "../core/LightClient.sol";
import "../core/Module.sol";

import "../../../contracts/Manager.sol";

contract IBCPacketTests is UnionTests {
    string public constant CLIENT_TYPE = "zkgm";
    string public constant VERSION = "zkgm-1";
    uint32 public constant COUNTERPARTY_CHANNEL_ID = 0xDEADC0DE;
    bytes public constant COUNTERPARTY_PORT_ID = "wasm.abcdef";

    Manager manager;
    TestIBCHandler handler;
    TestLightClient lightClient;
    TestModule module;

    uint32 clientId;
    uint32 connectionId;
    uint32 channelId;

    function setUp() public {
        (manager, handler) = setupHandler();
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
            counterpartyConnectionId: 0xCAFE,
            counterpartyClientId: 0xDEADC0DE,
            clientId: clientId,
            proofInit: hex"",
            proofHeight: 0
        });
        lightClient.pushValidMembership();
        connectionId = handler.connectionOpenTry(msgTry_);
        IBCMsgs.MsgConnectionOpenConfirm memory msgConfirm_ = IBCMsgs
            .MsgConnectionOpenConfirm({
            connectionId: connectionId,
            proofAck: hex"",
            proofHeight: 0
        });
        lightClient.pushValidMembership();
        handler.connectionOpenConfirm(msgConfirm_);

        // Create channel
        IBCMsgs.MsgChannelOpenInit memory msgInit_ = IBCMsgs.MsgChannelOpenInit({
            portId: address(module),
            counterpartyPortId: COUNTERPARTY_PORT_ID,
            connectionId: connectionId,
            version: VERSION,
            relayer: address(this)
        });
        channelId = handler.channelOpenInit(msgInit_);
        IBCMsgs.MsgChannelOpenAck memory msgAck_ = IBCMsgs.MsgChannelOpenAck({
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
        bytes calldata packet
    ) public {
        vm.pauseGasMetering();
        vm.assume(timeoutTimestamp > 0);
        vm.prank(address(module));
        vm.resumeGasMetering();
        handler.sendPacket(channelId, 0, timeoutTimestamp, packet);
    }

    function test_sendPacket_commitmentSaved(
        uint64 timeoutTimestamp,
        bytes calldata message
    ) public {
        vm.assume(timeoutTimestamp > 0);
        vm.prank(address(module));
        handler.sendPacket(channelId, 0, timeoutTimestamp, message);
        IBCPacket memory packet = IBCPacket({
            sourceChannelId: channelId,
            destinationChannelId: COUNTERPARTY_CHANNEL_ID,
            data: message,
            timeoutHeight: 0,
            timeoutTimestamp: timeoutTimestamp
        });
        assertEq(
            handler.commitments(
                IBCCommitment.batchPacketsCommitmentKey(
                    IBCPacketLib.commitPacket(packet)
                )
            ),
            IBCPacketLib.COMMITMENT_MAGIC
        );
    }

    function test_sendPacket_missingTimeout(
        bytes calldata packet
    ) public {
        vm.expectRevert(IBCErrors.ErrTimeoutMustBeSet.selector);
        vm.prank(address(module));
        handler.sendPacket(channelId, 0, 0, packet);
    }

    function test_sendPacket_channelDoesntExist(
        uint32 channelId_,
        uint64 timeoutTimestamp,
        bytes calldata packet
    ) public {
        vm.assume(channelId_ != channelId);
        vm.assume(timeoutTimestamp > 0);
        vm.expectRevert(IBCErrors.ErrUnauthorized.selector);
        vm.prank(address(module));
        handler.sendPacket(channelId_, 0, timeoutTimestamp, packet);
    }

    function test_sendPacket_moduleIsntChannelOwner(
        uint64 timeoutTimestamp,
        bytes calldata packet,
        address malicious
    ) public {
        vm.assume(malicious != address(module));
        vm.assume(timeoutTimestamp > 0);
        vm.expectRevert(IBCErrors.ErrUnauthorized.selector);
        vm.prank(malicious);
        handler.sendPacket(channelId, 0, timeoutTimestamp, packet);
    }

    function createReceivePacket(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) internal view returns (IBCMsgs.MsgPacketRecv memory) {
        IBCPacket[] memory packets = new IBCPacket[](nbPackets);
        bytes[] memory relayerMsgs = new bytes[](nbPackets);
        for (uint8 i = 0; i < nbPackets; i++) {
            packets[i] = IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: channelId,
                data: abi.encodePacked(message, i),
                timeoutHeight: 0,
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
        uint32 sourceChannelId,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_recvPacket_ok(sourceChannelId, message, 1);
    }

    function test_recvPacket_ok_5(
        uint32 sourceChannelId,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_recvPacket_ok(sourceChannelId, message, 5);
    }

    function test_recvPacket_ok_10(
        uint32 sourceChannelId,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_recvPacket_ok(sourceChannelId, message, 10);
    }

    function test_recvPacket_ok_15(
        uint32 sourceChannelId,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_recvPacket_ok(sourceChannelId, message, 15);
    }

    function test_recvPacket_ok_20(
        uint32 sourceChannelId,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_recvPacket_ok(sourceChannelId, message, 20);
    }

    function test_recvPacket_ok(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) public returns (IBCMsgs.MsgPacketRecv memory) {
        vm.pauseGasMetering();
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourceChannelId, message, nbPackets);
        lightClient.pushValidMembership();
        for (uint8 i = 0; i < nbPackets; i++) {
            vm.expectEmit();
            emit IBCPacketLib.PacketRecv(
                channelId,
                IBCPacketLib.commitPacket(msg_.packets[i]),
                msg_.relayer,
                msg_.relayerMsgs[i]
            );
            emit IBCPacketLib.WriteAck(
                channelId,
                IBCPacketLib.commitPacket(msg_.packets[i]),
                TestModuleLib.ACKNOWLEDGEMENT
            );
        }
        vm.resumeGasMetering();
        handler.recvPacket(msg_);
        vm.pauseGasMetering();
        return msg_;
    }

    function test_recvPacket_invalidProof(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourceChannelId, message, nbPackets);
        vm.expectRevert(IBCErrors.ErrInvalidProof.selector);
        handler.recvPacket(msg_);
    }

    function test_recvPacket_invalidChannelState(
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(nbPackets > 0);
        vm.assume(destinationChannelId != channelId);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourceChannelId, message, nbPackets);
        // fake non existant channel
        msg_.packets[0].destinationChannelId = destinationChannelId;
        vm.expectRevert(IBCErrors.ErrInvalidChannelState.selector);
        handler.recvPacket(msg_);
    }

    function test_recvPacket_batchShareSameChannel(
        uint32 sourceChannelId,
        uint32 fakeDestinationChannelId,
        bytes calldata message,
        uint8 nbPackets,
        uint8 tamperIndex
    ) public {
        vm.assume(nbPackets > 1);
        vm.assume(0 < tamperIndex && tamperIndex < nbPackets);
        vm.assume(fakeDestinationChannelId != channelId);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourceChannelId, message, nbPackets);
        msg_.packets[tamperIndex].destinationChannelId =
            fakeDestinationChannelId;
        lightClient.pushValidMembership();
        vm.expectRevert(IBCErrors.ErrBatchSameChannelOnly.selector);
        handler.recvPacket(msg_);
    }

    function test_recvPacket_timeoutTimestamp(
        uint32 timeout,
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(timeout > 0);
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourceChannelId, message, nbPackets);
        // Timeout is expressed as nano because of ibc-go...
        msg_.packets[0].timeoutTimestamp = uint64(timeout) * 1e9;
        vm.warp(timeout);
        lightClient.pushValidMembership();
        vm.expectRevert(IBCErrors.ErrTimestampTimeout.selector);
        handler.recvPacket(msg_);
    }

    function test_recvPacket_ackCommitmentSaved(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourceChannelId, message, nbPackets);
        lightClient.pushValidMembership();
        handler.recvPacket(msg_);
        for (uint8 i = 0; i < nbPackets; i++) {
            assertEq(
                handler.commitments(
                    IBCCommitment.batchReceiptsCommitmentKey(
                        IBCPacketLib.commitPacket(msg_.packets[i])
                    )
                ),
                IBCPacketLib.commitAck(TestModuleLib.ACKNOWLEDGEMENT)
            );
        }
    }

    function test_recvPacket_noAck_receiptCommitmentSaved(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourceChannelId, message, nbPackets);
        lightClient.pushValidMembership();
        module.pauseAck();
        handler.recvPacket(msg_);
        for (uint8 i = 0; i < nbPackets; i++) {
            assertEq(
                handler.commitments(
                    IBCCommitment.batchReceiptsCommitmentKey(
                        IBCPacketLib.commitPacket(msg_.packets[i])
                    )
                ),
                IBCPacketLib.COMMITMENT_MAGIC
            );
        }
    }

    function createReceiveIntentPacket(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) internal view returns (IBCMsgs.MsgIntentPacketRecv memory) {
        IBCPacket[] memory packets = new IBCPacket[](nbPackets);
        bytes[] memory marketMakerMsgs = new bytes[](nbPackets);
        for (uint8 i = 0; i < nbPackets; i++) {
            packets[i] = IBCPacket({
                sourceChannelId: sourceChannelId,
                destinationChannelId: channelId,
                data: abi.encodePacked(message, i),
                timeoutHeight: 0,
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
        uint32 sourceChannelId,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_recvIntentPacket_ok(sourceChannelId, message, 1);
    }

    function test_recvIntentPacket_ok_5(
        uint32 sourceChannelId,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_recvIntentPacket_ok(sourceChannelId, message, 5);
    }

    function test_recvIntentPacket_ok_10(
        uint32 sourceChannelId,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_recvIntentPacket_ok(sourceChannelId, message, 10);
    }

    function test_recvIntentPacket_ok_15(
        uint32 sourceChannelId,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_recvIntentPacket_ok(sourceChannelId, message, 15);
    }

    function test_recvIntentPacket_ok_20(
        uint32 sourceChannelId,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_recvIntentPacket_ok(sourceChannelId, message, 20);
    }

    function test_recvIntentPacket_ok(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.pauseGasMetering();
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgIntentPacketRecv memory msg_ =
            createReceiveIntentPacket(sourceChannelId, message, nbPackets);
        for (uint8 i = 0; i < nbPackets; i++) {
            vm.expectEmit();
            emit IBCPacketLib.IntentPacketRecv(
                channelId,
                IBCPacketLib.commitPacket(msg_.packets[i]),
                msg_.marketMaker,
                msg_.marketMakerMsgs[i]
            );
            emit IBCPacketLib.WriteAck(
                channelId,
                IBCPacketLib.commitPacket(msg_.packets[i]),
                TestModuleLib.ACKNOWLEDGEMENT
            );
        }
        vm.resumeGasMetering();
        handler.recvIntentPacket(msg_);
        vm.pauseGasMetering();
    }

    function test_recvIntentPacket_commitmentSaved(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgIntentPacketRecv memory msg_ =
            createReceiveIntentPacket(sourceChannelId, message, nbPackets);
        handler.recvIntentPacket(msg_);
        for (uint8 i = 0; i < nbPackets; i++) {
            assertEq(
                handler.commitments(
                    IBCCommitment.batchReceiptsCommitmentKey(
                        IBCPacketLib.commitPacket(msg_.packets[i])
                    )
                ),
                IBCPacketLib.commitAck(TestModuleLib.ACKNOWLEDGEMENT)
            );
        }
    }

    function test_recvIntentPacket_timeoutTimestamp(
        uint32 timeout,
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(timeout > 0);
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgIntentPacketRecv memory msg_ =
            createReceiveIntentPacket(sourceChannelId, message, nbPackets);
        // Timeout is expressed as nano because of ibc-go...
        msg_.packets[0].timeoutTimestamp = uint64(timeout) * 1e9;
        vm.warp(timeout);
        vm.expectRevert(IBCErrors.ErrTimestampTimeout.selector);
        handler.recvIntentPacket(msg_);
    }

    function createPacketAcknowledgement(
        uint32 destinationChannel,
        bytes calldata message,
        uint8 nbPackets
    ) internal view returns (IBCMsgs.MsgPacketAcknowledgement memory) {
        IBCPacket[] memory packets = new IBCPacket[](nbPackets);
        bytes[] memory acknowledgements = new bytes[](nbPackets);
        for (uint8 i = 0; i < nbPackets; i++) {
            packets[i] = IBCPacket({
                sourceChannelId: channelId,
                destinationChannelId: destinationChannel,
                data: abi.encodePacked(message, i),
                timeoutHeight: 0,
                timeoutTimestamp: type(uint64).max
            });
            acknowledgements[i] = abi.encodePacked(i);
        }
        IBCMsgs.MsgPacketAcknowledgement memory msg_ = IBCMsgs
            .MsgPacketAcknowledgement({
            packets: packets,
            acknowledgements: acknowledgements,
            relayer: address(this),
            proof: hex"",
            proofHeight: 0
        });
        return msg_;
    }

    function test_acknowledgePacket_ok(
        uint32 destinationChannel,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.pauseGasMetering();
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketAcknowledgement memory msg_ =
            createPacketAcknowledgement(destinationChannel, message, nbPackets);
        lightClient.pushValidMembership();
        for (uint8 i = 0; i < nbPackets; i++) {
            handler.assumePacketSent(msg_.packets[i]);
        }
        for (uint8 i = 0; i < nbPackets; i++) {
            vm.expectEmit();
            emit IBCPacketLib.PacketAck(
                channelId,
                IBCPacketLib.commitPacket(msg_.packets[i]),
                abi.encodePacked(i),
                msg_.relayer
            );
        }
        vm.resumeGasMetering();
        handler.acknowledgePacket(msg_);
    }

    function test_acknowledgePacket_ok_1(
        uint32 destinationChannel,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_acknowledgePacket_ok(destinationChannel, message, 1);
    }

    function test_acknowledgePacket_ok_5(
        uint32 destinationChannel,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_acknowledgePacket_ok(destinationChannel, message, 5);
    }

    function test_acknowledgePacket_ok_10(
        uint32 destinationChannel,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_acknowledgePacket_ok(destinationChannel, message, 10);
    }

    function test_acknowledgePacket_ok_15(
        uint32 destinationChannel,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_acknowledgePacket_ok(destinationChannel, message, 15);
    }

    function test_acknowledgePacket_ok_20(
        uint32 destinationChannel,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_acknowledgePacket_ok(destinationChannel, message, 20);
    }

    function test_acknowledgePacket_tampered(
        uint32 destinationChannel,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketAcknowledgement memory msg_ =
            createPacketAcknowledgement(destinationChannel, message, nbPackets);
        lightClient.pushValidMembership();
        for (uint8 i = 0; i < nbPackets; i++) {
            handler.assumePacketSent(msg_.packets[i]);
        }
        msg_.packets[0].data = abi.encodePacked(msg_.packets[0].data, hex"1337");
        vm.expectRevert(IBCErrors.ErrPacketCommitmentNotFound.selector);
        handler.acknowledgePacket(msg_);
    }

    function test_acknowledgePacket_batchShareSameChannel(
        uint32 destinationChannel,
        uint32 fakeSourceChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(fakeSourceChannelId != channelId);
        vm.assume(nbPackets > 1);
        IBCMsgs.MsgPacketAcknowledgement memory msg_ =
            createPacketAcknowledgement(destinationChannel, message, nbPackets);
        lightClient.pushValidMembership();
        for (uint8 i = 0; i < nbPackets; i++) {
            handler.assumePacketSent(msg_.packets[i]);
        }
        msg_.packets[1].sourceChannelId = fakeSourceChannelId;
        vm.expectRevert(IBCErrors.ErrBatchSameChannelOnly.selector);
        handler.acknowledgePacket(msg_);
    }

    function test_acknowledgePacket_notSent(
        uint32 destinationChannel,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketAcknowledgement memory msg_ =
            createPacketAcknowledgement(destinationChannel, message, nbPackets);
        lightClient.pushValidMembership();
        vm.expectRevert(IBCErrors.ErrPacketCommitmentNotFound.selector);
        handler.acknowledgePacket(msg_);
    }

    function test_acknowledgePacket_commitmentMarked(
        uint32 destinationChannel,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketAcknowledgement memory msg_ =
            createPacketAcknowledgement(destinationChannel, message, nbPackets);
        for (uint8 i = 0; i < nbPackets; i++) {
            handler.assumePacketSent(msg_.packets[i]);
        }
        lightClient.pushValidMembership();
        handler.acknowledgePacket(msg_);
        for (uint8 i = 0; i < nbPackets; i++) {
            assertEq(
                handler.commitments(
                    IBCCommitment.batchPacketsCommitmentKey(
                        IBCPacketLib.commitPacket(msg_.packets[i])
                    )
                ),
                IBCPacketLib.COMMITMENT_MAGIC_ACK
            );
        }
    }

    function test_acknowledgePacket_invalidProof(
        uint32 destinationChannel,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketAcknowledgement memory msg_ =
            createPacketAcknowledgement(destinationChannel, message, nbPackets);
        for (uint8 i = 0; i < nbPackets; i++) {
            handler.assumePacketSent(msg_.packets[i]);
        }
        vm.expectRevert(IBCErrors.ErrInvalidProof.selector);
        handler.acknowledgePacket(msg_);
    }

    function createPacketTimeout(
        uint32 destinationChannel,
        bytes calldata message
    ) internal view returns (IBCMsgs.MsgPacketTimeout memory) {
        IBCPacket memory packet = IBCPacket({
            sourceChannelId: channelId,
            destinationChannelId: destinationChannel,
            data: message,
            timeoutHeight: 0,
            timeoutTimestamp: type(uint64).max
        });
        IBCMsgs.MsgPacketTimeout memory msg_ = IBCMsgs.MsgPacketTimeout({
            packet: packet,
            relayer: address(this),
            proof: hex"",
            proofHeight: 0
        });
        return msg_;
    }

    function test_timeoutPacket_timestamp_ok(
        uint32 destinationChannel,
        bytes calldata message,
        uint32 timestamp,
        uint32 k
    ) public returns (IBCMsgs.MsgPacketTimeout memory) {
        vm.pauseGasMetering();
        vm.assume(timestamp > 0);
        vm.assume(k <= timestamp);
        IBCMsgs.MsgPacketTimeout memory msg_ =
            createPacketTimeout(destinationChannel, message);
        // fake timeout
        msg_.packet.timeoutTimestamp = timestamp;
        handler.assumePacketSent(msg_.packet);
        lightClient.pushValidNonMembership();
        lightClient.setLatestTimestamp(uint64(timestamp) + k);
        vm.expectEmit();
        emit IBCPacketLib.PacketTimeout(
            channelId, IBCPacketLib.commitPacket(msg_.packet), msg_.relayer
        );
        vm.resumeGasMetering();
        handler.timeoutPacket(msg_);
        vm.pauseGasMetering();
        return msg_;
    }

    function test_timeoutPacket_timestamp_commitmentMarked(
        uint32 destinationChannel,
        bytes calldata message,
        uint32 timestamp,
        uint32 k
    ) public {
        IBCMsgs.MsgPacketTimeout memory msg_ = test_timeoutPacket_timestamp_ok(
            destinationChannel, message, timestamp, k
        );
        assertEq(
            handler.commitments(
                IBCCommitment.batchPacketsCommitmentKey(
                    IBCPacketLib.commitPacket(msg_.packet)
                )
            ),
            IBCPacketLib.COMMITMENT_MAGIC_ACK
        );
    }

    function test_timeoutPacket_timestamp_notReached(
        uint32 destinationChannel,
        bytes calldata message,
        uint32 timestamp,
        uint32 k
    ) public {
        vm.assume(timestamp > 0);
        vm.assume(k <= timestamp);
        IBCMsgs.MsgPacketTimeout memory msg_ =
            createPacketTimeout(destinationChannel, message);
        // fake timeout
        msg_.packet.timeoutTimestamp = uint64(timestamp) + k + 1;
        handler.assumePacketSent(msg_.packet);
        lightClient.pushValidNonMembership();
        lightClient.setLatestTimestamp(timestamp);
        vm.expectRevert(IBCErrors.ErrTimeoutTimestampNotReached.selector);
        handler.timeoutPacket(msg_);
    }

    function test_timeoutPacket_timestamp_invalidProof(
        uint32 destinationChannel,
        bytes calldata message,
        uint32 timestamp,
        uint32 k
    ) public {
        vm.assume(timestamp > 0);
        vm.assume(k <= timestamp);
        IBCMsgs.MsgPacketTimeout memory msg_ =
            createPacketTimeout(destinationChannel, message);
        // fake timeout
        msg_.packet.timeoutTimestamp = timestamp;
        handler.assumePacketSent(msg_.packet);
        lightClient.setLatestTimestamp(uint64(timestamp) + k);
        vm.expectRevert(IBCErrors.ErrInvalidProof.selector);
        handler.timeoutPacket(msg_);
    }

    function test_writeAcknowledgement_ok(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) public returns (IBCMsgs.MsgPacketRecv memory) {
        vm.pauseGasMetering();
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourceChannelId, message, nbPackets);
        lightClient.pushValidMembership();
        module.pauseAck();
        for (uint8 i = 0; i < nbPackets; i++) {
            vm.expectEmit();
            emit IBCPacketLib.PacketRecv(
                channelId,
                IBCPacketLib.commitPacket(msg_.packets[i]),
                msg_.relayer,
                msg_.relayerMsgs[i]
            );
        }
        handler.recvPacket(msg_);
        for (uint8 i = 0; i < nbPackets; i++) {
            bytes memory ack = abi.encodePacked(i);
            vm.expectEmit();
            emit IBCPacketLib.WriteAck(
                channelId, IBCPacketLib.commitPacket(msg_.packets[i]), ack
            );
            vm.prank(address(module));
            vm.resumeGasMetering();
            handler.writeAcknowledgement(msg_.packets[i], ack);
            vm.pauseGasMetering();
        }
        return msg_;
    }

    function test_writeAcknowledgement_ok_1(
        uint32 sourceChannelId,
        bytes calldata message
    ) public {
        vm.pauseGasMetering();
        test_writeAcknowledgement_ok(sourceChannelId, message, 1);
    }

    function test_writeAcknowledgement_moduleIsntChannelOwner(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets,
        address malicious
    ) public {
        vm.assume(nbPackets > 0);
        vm.assume(malicious != address(module));
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourceChannelId, message, nbPackets);
        lightClient.pushValidMembership();
        module.pauseAck();
        for (uint8 i = 0; i < nbPackets; i++) {
            vm.expectEmit();
            emit IBCPacketLib.PacketRecv(
                channelId,
                IBCPacketLib.commitPacket(msg_.packets[i]),
                msg_.relayer,
                msg_.relayerMsgs[i]
            );
        }
        handler.recvPacket(msg_);
        for (uint8 i = 0; i < nbPackets; i++) {
            vm.expectRevert(IBCErrors.ErrUnauthorized.selector);
            vm.prank(malicious);
            handler.writeAcknowledgement(msg_.packets[i], hex"1337");
        }
    }

    function test_writeAcknowledgement_packetNotReceived(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(nbPackets > 0);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourceChannelId, message, nbPackets);
        lightClient.pushValidMembership();
        module.pauseAck();
        for (uint8 i = 0; i < nbPackets; i++) {
            vm.expectRevert(IBCErrors.ErrPacketNotReceived.selector);
            vm.prank(address(module));
            handler.writeAcknowledgement(msg_.packets[i], hex"1337");
        }
    }

    function test_writeAcknowledgement_alreadyExists(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        IBCMsgs.MsgPacketRecv memory msg_ =
            test_writeAcknowledgement_ok(sourceChannelId, message, nbPackets);
        for (uint8 i = 0; i < nbPackets; i++) {
            vm.expectRevert(IBCErrors.ErrAcknowledgementAlreadyExists.selector);
            vm.prank(address(module));
            handler.writeAcknowledgement(msg_.packets[i], hex"1337");
        }
    }

    function test_writeAcknowledgement_commitmentSaved(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        IBCMsgs.MsgPacketRecv memory msg_ =
            test_writeAcknowledgement_ok(sourceChannelId, message, nbPackets);
        for (uint8 i = 0; i < nbPackets; i++) {
            assertEq(
                handler.commitments(
                    IBCCommitment.batchReceiptsCommitmentKey(
                        IBCPacketLib.commitPacket(msg_.packets[i])
                    )
                ),
                IBCPacketLib.commitAck(abi.encodePacked(i))
            );
        }
    }

    function test_batchSend_ok(
        uint64 timeoutTimestamp,
        uint8 nbPackets
    ) public returns (IBCPacket[] memory) {
        vm.pauseGasMetering();
        vm.assume(nbPackets > 1);
        vm.assume(timeoutTimestamp > 0);
        IBCPacket[] memory packets = new IBCPacket[](nbPackets);
        for (uint8 i = 0; i < nbPackets; i++) {
            vm.prank(address(module));
            bytes memory message = abi.encodePacked(i);
            handler.sendPacket(channelId, 0, timeoutTimestamp, message);
            IBCPacket memory packet = IBCPacket({
                sourceChannelId: channelId,
                destinationChannelId: COUNTERPARTY_CHANNEL_ID,
                data: message,
                timeoutHeight: 0,
                timeoutTimestamp: timeoutTimestamp
            });
            packets[i] = packet;
        }
        vm.resumeGasMetering();
        handler.batchSend(IBCMsgs.MsgBatchSend({packets: packets}));
        vm.pauseGasMetering();
        return packets;
    }

    function test_batchSend_ok_2(
        uint64 timeoutTimestamp
    ) public {
        vm.pauseGasMetering();
        test_batchSend_ok(timeoutTimestamp, 2);
    }

    function test_batchSend_ok_5(
        uint64 timeoutTimestamp
    ) public {
        vm.pauseGasMetering();
        test_batchSend_ok(timeoutTimestamp, 5);
    }

    function test_batchSend_ok_10(
        uint64 timeoutTimestamp
    ) public {
        vm.pauseGasMetering();
        test_batchSend_ok(timeoutTimestamp, 10);
    }

    function test_batchSend_ok_15(
        uint64 timeoutTimestamp
    ) public {
        vm.pauseGasMetering();
        test_batchSend_ok(timeoutTimestamp, 15);
    }

    function test_batchSend_ok_20(
        uint64 timeoutTimestamp
    ) public {
        vm.pauseGasMetering();
        test_batchSend_ok(timeoutTimestamp, 20);
    }

    function test_batchSend_commitmentSaved(
        uint64 timeoutTimestamp,
        uint8 nbPackets
    ) public {
        IBCPacket[] memory packets =
            test_batchSend_ok(timeoutTimestamp, nbPackets);
        assertEq(
            handler.commitments(
                IBCCommitment.batchPacketsCommitmentKey(
                    IBCPacketLib.commitPacketsMemory(packets)
                )
            ),
            IBCPacketLib.COMMITMENT_MAGIC
        );
    }

    function test_batchSend_packetNotSent(
        uint64 timeoutTimestamp,
        uint8 nbPackets
    ) public {
        vm.pauseGasMetering();
        vm.assume(nbPackets > 1);
        vm.assume(timeoutTimestamp > 0);
        IBCPacket[] memory packets = new IBCPacket[](nbPackets);
        for (uint8 i = 0; i < nbPackets; i++) {
            vm.prank(address(module));
            bytes memory message = abi.encodePacked(i);
            handler.sendPacket(channelId, 0, timeoutTimestamp, message);
            IBCPacket memory packet = IBCPacket({
                sourceChannelId: channelId,
                destinationChannelId: COUNTERPARTY_CHANNEL_ID,
                data: message,
                timeoutHeight: 0,
                timeoutTimestamp: timeoutTimestamp
            });
            packets[i] = packet;
        }
        // tamper the data such that the commitment mismatch
        packets[0].data = abi.encodePacked(packets[0].data, hex"C0DE");
        vm.resumeGasMetering();
        vm.expectRevert(IBCErrors.ErrPacketCommitmentNotFound.selector);
        handler.batchSend(IBCMsgs.MsgBatchSend({packets: packets}));
    }

    function test_batchAcks_afterRecvPacket_ok(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets,
        bytes calldata ack
    ) public returns (IBCMsgs.MsgPacketRecv memory, bytes[] memory) {
        vm.pauseGasMetering();
        vm.assume(ack.length > 0);
        vm.assume(nbPackets > 1);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourceChannelId, message, nbPackets);
        lightClient.pushValidMembership();
        module.setAck(ack);
        handler.recvPacket(msg_);
        bytes[] memory acks = new bytes[](nbPackets);
        for (uint8 i = 0; i < nbPackets; i++) {
            acks[i] = ack;
        }
        vm.resumeGasMetering();
        handler.batchAcks(
            IBCMsgs.MsgBatchAcks({packets: msg_.packets, acks: acks})
        );
        vm.pauseGasMetering();
        return (msg_, acks);
    }

    function test_batchAcks_afterRecvPacket_ok_2(
        uint32 sourceChannelId,
        bytes calldata message,
        bytes calldata ack
    ) public {
        vm.pauseGasMetering();
        test_batchAcks_afterRecvPacket_ok(sourceChannelId, message, 2, ack);
    }

    function test_batchAcks_afterRecvPacket_ok_5(
        uint32 sourceChannelId,
        bytes calldata message,
        bytes calldata ack
    ) public {
        vm.pauseGasMetering();
        test_batchAcks_afterRecvPacket_ok(sourceChannelId, message, 5, ack);
    }

    function test_batchAcks_afterRecvPacket_ok_10(
        uint32 sourceChannelId,
        bytes calldata message,
        bytes calldata ack
    ) public {
        vm.pauseGasMetering();
        test_batchAcks_afterRecvPacket_ok(sourceChannelId, message, 10, ack);
    }

    function test_batchAcks_afterRecvPacket_ok_15(
        uint32 sourceChannelId,
        bytes calldata message,
        bytes calldata ack
    ) public {
        vm.pauseGasMetering();
        test_batchAcks_afterRecvPacket_ok(sourceChannelId, message, 15, ack);
    }

    function test_batchAcks_afterRecvPacket_ok_20(
        uint32 sourceChannelId,
        bytes calldata message,
        bytes calldata ack
    ) public {
        vm.pauseGasMetering();
        test_batchAcks_afterRecvPacket_ok(sourceChannelId, message, 20, ack);
    }

    function test_batchAcks_afterRecvPacket_commitmentSaved(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets,
        bytes calldata ack
    ) public {
        (IBCMsgs.MsgPacketRecv memory msg_, bytes[] memory acks) =
        test_batchAcks_afterRecvPacket_ok(
            sourceChannelId, message, nbPackets, ack
        );
        assertEq(
            handler.commitments(
                IBCCommitment.batchReceiptsCommitmentKey(
                    IBCPacketLib.commitPacketsMemory(msg_.packets)
                )
            ),
            IBCPacketLib.commitAcksMemory(acks)
        );
    }

    function test_batchAcks_packetNotReceived(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets,
        bytes calldata ack
    ) public {
        vm.assume(ack.length > 0);
        vm.assume(nbPackets > 1);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourceChannelId, message, nbPackets);
        module.setAck(ack);
        bytes[] memory acks = new bytes[](nbPackets);
        for (uint8 i = 0; i < nbPackets; i++) {
            acks[i] = ack;
        }
        vm.expectRevert(IBCErrors.ErrAcknowledgementIsEmpty.selector);
        handler.batchAcks(
            IBCMsgs.MsgBatchAcks({packets: msg_.packets, acks: acks})
        );
    }

    function test_batchAcks_afterRecvPacket_tamperedPacket(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets,
        bytes calldata ack
    ) public {
        vm.assume(ack.length > 0);
        vm.assume(nbPackets > 1);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourceChannelId, message, nbPackets);
        lightClient.pushValidMembership();
        module.setAck(ack);
        handler.recvPacket(msg_);
        bytes[] memory acks = new bytes[](nbPackets);
        for (uint8 i = 0; i < nbPackets; i++) {
            acks[i] = ack;
        }
        // tamper one packet
        msg_.packets[0].data = abi.encodePacked(msg_.packets[0].data, hex"1337");
        vm.expectRevert(IBCErrors.ErrAcknowledgementIsEmpty.selector);
        handler.batchAcks(
            IBCMsgs.MsgBatchAcks({packets: msg_.packets, acks: acks})
        );
    }

    function test_batchAcks_afterRecvPacket_asyncAck(
        uint32 sourceChannelId,
        bytes calldata message,
        uint8 nbPackets
    ) public {
        vm.assume(nbPackets > 1);
        IBCMsgs.MsgPacketRecv memory msg_ =
            createReceivePacket(sourceChannelId, message, nbPackets);
        lightClient.pushValidMembership();
        module.setAck(hex"");
        handler.recvPacket(msg_);
        bytes[] memory acks = new bytes[](nbPackets);
        vm.expectRevert(IBCErrors.ErrAcknowledgementIsEmpty.selector);
        handler.batchAcks(
            IBCMsgs.MsgBatchAcks({packets: msg_.packets, acks: acks})
        );
    }
}
