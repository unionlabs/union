pragma solidity ^0.8.23;

import "forge-std/Test.sol";

import "../core/IBCHandler.sol";
import "../core/LightClient.sol";
import "../core/Module.sol";

contract IBCChannelTests is Test {
    bytes32 public constant CLIENT_TYPE = keccak256("zkgm");

    TestIBCHandler handler;
    TestLightClient lightClient;
    TestModule module;

    uint32 clientId;
    uint32 connectionId;

    function setUp() public {
        handler = new TestIBCHandler();
        lightClient = new TestLightClient();
        module = new TestModule(address(handler));
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
    }

    function test_channelOpenInit_ok(IBCMsgs.MsgChannelOpenInit memory msg_)
        public
    {
        vm.pauseGasMetering();
        msg_.portId = address(module);
        msg_.channel.state = IBCChannelState.Init;
        msg_.channel.connectionId = connectionId;
        msg_.channel.ordering = IBCChannelOrder.Unordered;
        vm.expectEmit();
        emit IBCChannelLib.ChannelOpenInit(
            msg_.portId,
            IBCChannelLib.normalizePortId(msg_.portId),
            0,
            msg_.channel.counterparty.portId,
            msg_.channel.connectionId,
            msg_.channel.version
        );
        vm.resumeGasMetering();
        handler.channelOpenInit(msg_);
    }
}
