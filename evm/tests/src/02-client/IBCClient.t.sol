pragma solidity ^0.8.23;

import "forge-std/Test.sol";

import "../../../contracts/core/25-handler/IBCMsgs.sol";
import "../core/IBCClient.sol";
import "../core/LightClient.sol";

contract IBCClientTests is Test {
    TestIBCClient ibcClient;
    TestLightClient lightClient;

    function setUp() public {
        ibcClient = new TestIBCClient();
        lightClient = new TestLightClient();
    }

    function test_registerClient_ok(bytes32 typ, address impl) public {
        vm.pauseGasMetering();
        vm.assume(impl != address(0));
        vm.expectEmit();
        emit IBCClientLib.ClientRegistered(typ, impl);
        vm.resumeGasMetering();
        ibcClient.registerClient(typ, ILightClient(impl));
    }

    function test_registerClient_alreadyRegistered(
        bytes32 typ,
        address impl
    ) public {
        vm.assume(impl != address(0));
        vm.expectEmit();
        emit IBCClientLib.ClientRegistered(typ, impl);
        ibcClient.registerClient(typ, ILightClient(impl));
        vm.expectRevert(IBCClientLib.ErrClientTypeAlreadyExists.selector);
        ibcClient.registerClient(typ, ILightClient(impl));
    }

    function test_createClient_ok(IBCMsgs.MsgCreateClient calldata msg_)
        public
    {
        vm.pauseGasMetering();
        ibcClient.registerClient(msg_.clientType, lightClient);
        vm.expectEmit();
        emit IBCClientLib.ClientCreated(0);
        vm.resumeGasMetering();
        ibcClient.createClient(msg_);
    }

    function test_createClient_ko(IBCMsgs.MsgCreateClient calldata msg_)
        public
    {
        lightClient.setRevertCreate(true);
        ibcClient.registerClient(msg_.clientType, lightClient);
        vm.expectRevert();
        ibcClient.createClient(msg_);
    }

    function test_createClient_commitmentsSaved(
        IBCMsgs.MsgCreateClient calldata msg_
    ) public {
        vm.pauseGasMetering();
        ibcClient.registerClient(msg_.clientType, lightClient);
        vm.resumeGasMetering();
        uint32 clientId = ibcClient.createClient(msg_);
        assertEq(
            ibcClient.commitments(
                IBCCommitment.clientStateCommitmentKey(clientId)
            ),
            keccak256(msg_.clientStateBytes)
        );
        assertEq(
            ibcClient.commitments(
                IBCCommitment.consensusStateCommitmentKey(clientId, 0)
            ),
            keccak256(msg_.consensusStateBytes)
        );
    }

    function test_updateClient_ok(
        IBCMsgs.MsgCreateClient calldata msg_,
        bytes calldata clientMessage,
        address relayer
    ) public {
        vm.pauseGasMetering();
        ibcClient.registerClient(msg_.clientType, lightClient);
        uint32 clientId = ibcClient.createClient(msg_);
        vm.expectEmit();
        emit IBCClientLib.ClientUpdated(0, 1);
        vm.resumeGasMetering();
        ibcClient.updateClient(
            IBCMsgs.MsgUpdateClient({
                clientId: clientId,
                clientMessage: clientMessage,
                relayer: relayer
            })
        );
    }

    function test_updateClient_ko(
        IBCMsgs.MsgCreateClient calldata msg_,
        bytes calldata clientMessage,
        address relayer
    ) public {
        vm.pauseGasMetering();
        lightClient.setRevertUpdate(true);
        ibcClient.registerClient(msg_.clientType, lightClient);
        uint32 clientId = ibcClient.createClient(msg_);
        vm.resumeGasMetering();
        vm.expectRevert();
        ibcClient.updateClient(
            IBCMsgs.MsgUpdateClient({
                clientId: clientId,
                clientMessage: clientMessage,
                relayer: relayer
            })
        );
    }

    function test_updateClient_commitmentsSaved(
        IBCMsgs.MsgCreateClient calldata msg_,
        bytes calldata clientMessage,
        address relayer
    ) public {
        vm.pauseGasMetering();
        ibcClient.registerClient(msg_.clientType, lightClient);
        uint32 clientId = ibcClient.createClient(msg_);
        vm.resumeGasMetering();
        ibcClient.updateClient(
            IBCMsgs.MsgUpdateClient({
                clientId: clientId,
                clientMessage: clientMessage,
                relayer: relayer
            })
        );
        assertEq(
            ibcClient.commitments(
                IBCCommitment.clientStateCommitmentKey(clientId)
            ),
            keccak256(msg_.clientStateBytes)
        );
        assertEq(
            ibcClient.commitments(
                IBCCommitment.consensusStateCommitmentKey(clientId, 1)
            ),
            keccak256(clientMessage)
        );
    }
}
