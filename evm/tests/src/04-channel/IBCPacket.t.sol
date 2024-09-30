pragma solidity ^0.8.23;

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
        vm.pauseGasMetering();
        vm.prank(address(module));
        vm.resumeGasMetering();
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
        vm.pauseGasMetering();
        vm.prank(address(module));
        vm.resumeGasMetering();
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
                    channelId,
                    IBCPacketLib.commitPacketsMemory(
                        IBCPacketLib.batchSingleMemory(packet)
                    )
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
        vm.assume(channelId_ != channelId);
        vm.assume(timeoutTimestamp != 0 || timeoutHeight != 0);
        vm.pauseGasMetering();
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
        vm.assume(timeoutTimestamp != 0 || timeoutHeight != 0);
        vm.pauseGasMetering();
        vm.expectRevert(IBCPacketLib.ErrUnauthorized.selector);
        vm.resumeGasMetering();
        handler.sendPacket(channelId, timeoutTimestamp, timeoutHeight, packet);
    }
}
