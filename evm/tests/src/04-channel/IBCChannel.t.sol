pragma solidity ^0.8.27;

import "forge-std/Test.sol";

import "solady/utils/LibString.sol";

import "../core/IBCHandler.sol";
import "../core/LightClient.sol";
import "../core/Module.sol";

contract IBCChannelTests is Test {
    using LibString for *;

    string public constant CLIENT_TYPE = "zkgm";
    string public constant COUNTERPARTY_PORT_ID = "wasm.abcdef";

    TestIBCHandler handler;
    TestLightClient lightClient;
    TestModule module;

    uint32 clientId;
    uint32 connectionId;

    function setUp() public {
        handler = new TestIBCHandler();
        lightClient = new TestLightClient();
        module = new TestModule(handler);
        handler.registerClient(CLIENT_TYPE, lightClient);
        clientId = handler.createClient(
            IBCMsgs.MsgCreateClient({
                clientType: CLIENT_TYPE,
                clientStateBytes: hex"CADEBABE",
                consensusStateBytes: hex"DEADC0DE",
                relayer: address(this)
            })
        );
        IBCMsgs.MsgConnectionOpenTry memory msgTry_ = IBCMsgs
            .MsgConnectionOpenTry({
            counterpartyConnectionId: 0xCAFE,
            counterpartyClientId: 0xDEADC0DE,
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
    }

    function test_channelOpenInit_ok(
        string calldata version,
        address relayer
    ) public {
        vm.pauseGasMetering();
        IBCMsgs.MsgChannelOpenInit memory msg_ = IBCMsgs.MsgChannelOpenInit({
            portId: address(module),
            counterpartyPortId: COUNTERPARTY_PORT_ID,
            connectionId: connectionId,
            ordering: IBCChannelOrder.Unordered,
            version: version,
            relayer: relayer
        });
        vm.expectEmit();
        emit IBCChannelLib.ChannelOpenInit(
            msg_.portId.toHexString(),
            0,
            msg_.counterpartyPortId,
            msg_.connectionId,
            msg_.version
        );
        vm.resumeGasMetering();
        handler.channelOpenInit(msg_);
    }

    function test_channelOpenInit_channelClaimed(
        string calldata version,
        address relayer
    ) public {
        IBCMsgs.MsgChannelOpenInit memory msg_ = IBCMsgs.MsgChannelOpenInit({
            portId: address(module),
            counterpartyPortId: COUNTERPARTY_PORT_ID,
            connectionId: connectionId,
            ordering: IBCChannelOrder.Unordered,
            version: version,
            relayer: relayer
        });
        uint32 channelId = handler.channelOpenInit(msg_);
        assertEq(handler.channelOwner(channelId), address(module));
    }

    function test_channelOpenInit_commitmentSaved(
        string calldata version,
        address relayer
    ) public {
        IBCChannel memory channel = IBCChannel({
            state: IBCChannelState.Init,
            ordering: IBCChannelOrder.Unordered,
            connectionId: connectionId,
            counterpartyChannelId: 0,
            counterpartyPortId: COUNTERPARTY_PORT_ID,
            version: version
        });
        IBCMsgs.MsgChannelOpenInit memory msg_ = IBCMsgs.MsgChannelOpenInit({
            portId: address(module),
            counterpartyPortId: COUNTERPARTY_PORT_ID,
            connectionId: channel.connectionId,
            ordering: channel.ordering,
            version: channel.version,
            relayer: relayer
        });
        uint32 channelId = handler.channelOpenInit(msg_);
        assertEq(
            handler.commitments(IBCCommitment.channelCommitmentKey(channelId)),
            keccak256(abi.encode(channel))
        );
    }

    function test_channelOpenTry_ok(
        uint32 counterpartyChannelId,
        string calldata version,
        string calldata counterpartyVersion,
        address relayer
    ) public {
        vm.pauseGasMetering();
        IBCMsgs.MsgChannelOpenTry memory msg_ = IBCMsgs.MsgChannelOpenTry({
            portId: address(module),
            channel: IBCChannel({
                state: IBCChannelState.TryOpen,
                ordering: IBCChannelOrder.Unordered,
                connectionId: connectionId,
                counterpartyChannelId: counterpartyChannelId,
                counterpartyPortId: COUNTERPARTY_PORT_ID,
                version: version
            }),
            counterpartyVersion: counterpartyVersion,
            proofInit: hex"",
            proofHeight: 0,
            relayer: relayer
        });
        lightClient.pushValidMembership();
        vm.expectEmit();
        emit IBCChannelLib.ChannelOpenTry(
            msg_.portId.toHexString(),
            0,
            msg_.channel.counterpartyPortId,
            msg_.channel.counterpartyChannelId,
            msg_.channel.connectionId,
            msg_.counterpartyVersion
        );
        vm.resumeGasMetering();
        handler.channelOpenTry(msg_);
    }

    function test_channelOpenTry_invalidState(
        uint32 counterpartyChannelId,
        string calldata version,
        string calldata counterpartyVersion,
        address relayer
    ) public {
        IBCMsgs.MsgChannelOpenTry memory msg_ = IBCMsgs.MsgChannelOpenTry({
            portId: address(module),
            channel: IBCChannel({
                state: IBCChannelState.Unspecified,
                connectionId: connectionId,
                ordering: IBCChannelOrder.Unordered,
                version: version,
                counterpartyPortId: COUNTERPARTY_PORT_ID,
                counterpartyChannelId: counterpartyChannelId
            }),
            counterpartyVersion: counterpartyVersion,
            proofInit: hex"",
            proofHeight: 0,
            relayer: relayer
        });
        vm.expectRevert(IBCErrors.ErrInvalidChannelState.selector);
        handler.channelOpenTry(msg_);
    }

    function test_channelOpenTry_invalidOrdering(
        uint32 counterpartyChannelId,
        string calldata version,
        string calldata counterpartyVersion,
        address relayer
    ) public {
        IBCMsgs.MsgChannelOpenTry memory msg_ = IBCMsgs.MsgChannelOpenTry({
            portId: address(module),
            channel: IBCChannel({
                state: IBCChannelState.TryOpen,
                connectionId: connectionId,
                ordering: IBCChannelOrder.Unspecified,
                version: version,
                counterpartyPortId: COUNTERPARTY_PORT_ID,
                counterpartyChannelId: counterpartyChannelId
            }),
            counterpartyVersion: counterpartyVersion,
            proofInit: hex"",
            proofHeight: 0,
            relayer: relayer
        });
        vm.expectRevert(IBCErrors.ErrInvalidChannelOrdering.selector);
        handler.channelOpenTry(msg_);
    }

    function test_channelOpenTry_invalidProof(
        uint32 counterpartyChannelId,
        string calldata version,
        string calldata counterpartyVersion,
        address relayer
    ) public {
        IBCMsgs.MsgChannelOpenTry memory msg_ = IBCMsgs.MsgChannelOpenTry({
            portId: address(module),
            channel: IBCChannel({
                state: IBCChannelState.TryOpen,
                connectionId: connectionId,
                ordering: IBCChannelOrder.Unordered,
                version: version,
                counterpartyPortId: COUNTERPARTY_PORT_ID,
                counterpartyChannelId: counterpartyChannelId
            }),
            counterpartyVersion: counterpartyVersion,
            proofInit: hex"",
            proofHeight: 0,
            relayer: relayer
        });
        vm.expectRevert(IBCErrors.ErrInvalidProof.selector);
        handler.channelOpenTry(msg_);
    }

    function test_channelOpenTry_channelClaimed(
        uint32 counterpartyChannelId,
        string calldata version,
        string calldata counterpartyVersion,
        address relayer
    ) public {
        IBCMsgs.MsgChannelOpenTry memory msg_ = IBCMsgs.MsgChannelOpenTry({
            portId: address(module),
            channel: IBCChannel({
                state: IBCChannelState.TryOpen,
                connectionId: connectionId,
                ordering: IBCChannelOrder.Unordered,
                version: version,
                counterpartyPortId: COUNTERPARTY_PORT_ID,
                counterpartyChannelId: counterpartyChannelId
            }),
            counterpartyVersion: counterpartyVersion,
            proofInit: hex"",
            proofHeight: 0,
            relayer: relayer
        });
        lightClient.pushValidMembership();
        uint32 channelId = handler.channelOpenTry(msg_);
        assertEq(handler.channelOwner(channelId), address(module));
    }

    function test_channelOpenTry_commitmentSaved(
        uint32 counterpartyChannelId,
        string calldata version,
        string calldata counterpartyVersion,
        address relayer
    ) public {
        IBCChannel memory channel = IBCChannel({
            state: IBCChannelState.TryOpen,
            connectionId: connectionId,
            ordering: IBCChannelOrder.Unordered,
            version: version,
            counterpartyPortId: COUNTERPARTY_PORT_ID,
            counterpartyChannelId: counterpartyChannelId
        });
        IBCMsgs.MsgChannelOpenTry memory msg_ = IBCMsgs.MsgChannelOpenTry({
            portId: address(module),
            channel: channel,
            counterpartyVersion: counterpartyVersion,
            proofInit: hex"",
            proofHeight: 0,
            relayer: relayer
        });
        lightClient.pushValidMembership();
        uint32 channelId = handler.channelOpenTry(msg_);
        assertEq(
            handler.commitments(IBCCommitment.channelCommitmentKey(channelId)),
            keccak256(abi.encode(channel))
        );
    }

    function test_channelOpenInitOpenAck_ok(
        uint32 counterpartyChannelId,
        string calldata version,
        string calldata counterpartyVersion,
        address relayer
    ) public {
        vm.pauseGasMetering();
        IBCMsgs.MsgChannelOpenInit memory msgInit_ = IBCMsgs.MsgChannelOpenInit({
            portId: address(module),
            counterpartyPortId: COUNTERPARTY_PORT_ID,
            connectionId: connectionId,
            ordering: IBCChannelOrder.Unordered,
            version: version,
            relayer: relayer
        });
        uint32 channelId = handler.channelOpenInit(msgInit_);
        IBCMsgs.MsgChannelOpenAck memory msgAck_ = IBCMsgs.MsgChannelOpenAck({
            channelId: channelId,
            counterpartyVersion: counterpartyVersion,
            counterpartyChannelId: counterpartyChannelId,
            proofTry: hex"",
            proofHeight: 0,
            relayer: relayer
        });
        lightClient.pushValidMembership();
        vm.expectEmit();
        emit IBCChannelLib.ChannelOpenAck(
            msgInit_.portId.toHexString(),
            0,
            msgInit_.counterpartyPortId,
            counterpartyChannelId,
            connectionId
        );
        vm.resumeGasMetering();
        handler.channelOpenAck(msgAck_);
    }

    function test_channelOpenInitOpenAck_invalidState(
        uint32 channelId,
        uint32 counterpartyChannelId,
        string calldata counterpartyVersion,
        address relayer
    ) public {
        IBCMsgs.MsgChannelOpenAck memory msgAck_ = IBCMsgs.MsgChannelOpenAck({
            channelId: channelId,
            counterpartyVersion: counterpartyVersion,
            counterpartyChannelId: counterpartyChannelId,
            proofTry: hex"",
            proofHeight: 0,
            relayer: relayer
        });
        vm.expectRevert(IBCErrors.ErrInvalidChannelState.selector);
        handler.channelOpenAck(msgAck_);
    }

    function test_channelOpenInitOpenAck_commitmentSaved(
        uint32 counterpartyChannelId,
        string calldata version,
        string calldata counterpartyVersion,
        address relayer
    ) public {
        IBCChannel memory channel = IBCChannel({
            state: IBCChannelState.Init,
            connectionId: connectionId,
            ordering: IBCChannelOrder.Unordered,
            version: version,
            counterpartyPortId: COUNTERPARTY_PORT_ID,
            counterpartyChannelId: counterpartyChannelId
        });
        IBCMsgs.MsgChannelOpenInit memory msgInit_ = IBCMsgs.MsgChannelOpenInit({
            portId: address(module),
            counterpartyPortId: COUNTERPARTY_PORT_ID,
            connectionId: channel.connectionId,
            ordering: channel.ordering,
            version: channel.version,
            relayer: relayer
        });
        uint32 channelId = handler.channelOpenInit(msgInit_);
        IBCMsgs.MsgChannelOpenAck memory msgAck_ = IBCMsgs.MsgChannelOpenAck({
            channelId: channelId,
            counterpartyVersion: counterpartyVersion,
            counterpartyChannelId: counterpartyChannelId,
            proofTry: hex"",
            proofHeight: 0,
            relayer: relayer
        });
        channel.version = counterpartyVersion;
        channel.state = IBCChannelState.Open;
        lightClient.pushValidMembership();
        handler.channelOpenAck(msgAck_);
        assertEq(
            handler.commitments(IBCCommitment.channelCommitmentKey(channelId)),
            keccak256(abi.encode(channel))
        );
    }

    function test_channelOpenTryOpenConfirm_ok(
        uint32 counterpartyChannelId,
        string calldata version,
        string calldata counterpartyVersion,
        address relayer
    ) public {
        vm.pauseGasMetering();
        IBCMsgs.MsgChannelOpenTry memory msgTry_ = IBCMsgs.MsgChannelOpenTry({
            portId: address(module),
            channel: IBCChannel({
                state: IBCChannelState.TryOpen,
                connectionId: connectionId,
                ordering: IBCChannelOrder.Unordered,
                version: version,
                counterpartyPortId: COUNTERPARTY_PORT_ID,
                counterpartyChannelId: counterpartyChannelId
            }),
            counterpartyVersion: counterpartyVersion,
            proofInit: hex"",
            proofHeight: 0,
            relayer: relayer
        });
        lightClient.pushValidMembership();
        uint32 channelId = handler.channelOpenTry(msgTry_);
        IBCMsgs.MsgChannelOpenConfirm memory msgConfirm_ = IBCMsgs
            .MsgChannelOpenConfirm({
            channelId: channelId,
            proofAck: hex"",
            proofHeight: 0,
            relayer: relayer
        });
        lightClient.pushValidMembership();
        vm.expectEmit();
        emit IBCChannelLib.ChannelOpenConfirm(
            msgTry_.portId.toHexString(),
            0,
            msgTry_.channel.counterpartyPortId,
            counterpartyChannelId,
            connectionId
        );
        vm.resumeGasMetering();
        handler.channelOpenConfirm(msgConfirm_);
    }

    function test_channelOpenTryOpenConfirm_invalidState(
        uint32 channelId,
        address relayer
    ) public {
        IBCMsgs.MsgChannelOpenConfirm memory msgConfirm_ = IBCMsgs
            .MsgChannelOpenConfirm({
            channelId: channelId,
            proofAck: hex"",
            proofHeight: 0,
            relayer: relayer
        });
        lightClient.pushValidMembership();
        vm.expectRevert(IBCErrors.ErrInvalidChannelState.selector);
        handler.channelOpenConfirm(msgConfirm_);
    }

    function test_channelOpenTryOpenConfirm_commitmentSaved(
        uint32 counterpartyChannelId,
        string calldata version,
        string calldata counterpartyVersion,
        address relayer
    ) public {
        IBCChannel memory channel = IBCChannel({
            state: IBCChannelState.TryOpen,
            connectionId: connectionId,
            ordering: IBCChannelOrder.Unordered,
            version: version,
            counterpartyPortId: COUNTERPARTY_PORT_ID,
            counterpartyChannelId: counterpartyChannelId
        });
        IBCMsgs.MsgChannelOpenTry memory msgTry_ = IBCMsgs.MsgChannelOpenTry({
            portId: address(module),
            channel: channel,
            counterpartyVersion: counterpartyVersion,
            proofInit: hex"",
            proofHeight: 0,
            relayer: relayer
        });
        lightClient.pushValidMembership();
        uint32 channelId = handler.channelOpenTry(msgTry_);
        IBCMsgs.MsgChannelOpenConfirm memory msgConfirm_ = IBCMsgs
            .MsgChannelOpenConfirm({
            channelId: channelId,
            proofAck: hex"",
            proofHeight: 0,
            relayer: relayer
        });
        lightClient.pushValidMembership();
        handler.channelOpenConfirm(msgConfirm_);
        channel.state = IBCChannelState.Open;
        assertEq(
            handler.commitments(IBCCommitment.channelCommitmentKey(channelId)),
            keccak256(abi.encode(channel))
        );
    }
}
