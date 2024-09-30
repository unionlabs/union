pragma solidity ^0.8.23;

import "forge-std/Test.sol";

import "../core/IBCHandler.sol";
import "../core/LightClient.sol";

import "../../../contracts/core/25-handler/IBCMsgs.sol";

contract IBCClientTests is Test {
    TestIBCHandler handler;
    TestLightClient lightClient;

    function setUp() public {
        handler = new TestIBCHandler();
        lightClient = new TestLightClient();
    }

    function test_registerClient_ok(bytes32 typ, address impl) public {
        vm.pauseGasMetering();
        vm.assume(impl != address(0));
        vm.expectEmit();
        emit IBCClientLib.ClientRegistered(typ, impl);
        vm.resumeGasMetering();
        handler.registerClient(typ, ILightClient(impl));
    }

    function test_registerClient_alreadyRegistered(
        bytes32 typ,
        address impl
    ) public {
        vm.assume(impl != address(0));
        vm.expectEmit();
        emit IBCClientLib.ClientRegistered(typ, impl);
        handler.registerClient(typ, ILightClient(impl));
        vm.expectRevert(IBCClientLib.ErrClientTypeAlreadyExists.selector);
        handler.registerClient(typ, ILightClient(impl));
    }

    function test_createClient_ok(
        IBCMsgs.MsgCreateClient calldata msg_
    ) public {
        vm.pauseGasMetering();
        handler.registerClient(msg_.clientType, lightClient);
        vm.expectEmit();
        emit IBCClientLib.ClientCreated(0);
        vm.resumeGasMetering();
        handler.createClient(msg_);
    }

    function test_createClient_ko(
        IBCMsgs.MsgCreateClient calldata msg_
    ) public {
        lightClient.setRevertCreate(true);
        handler.registerClient(msg_.clientType, lightClient);
        vm.expectRevert();
        handler.createClient(msg_);
    }

    function test_createClient_commitmentsSaved(
        IBCMsgs.MsgCreateClient calldata msg_
    ) public {
        vm.pauseGasMetering();
        handler.registerClient(msg_.clientType, lightClient);
        vm.resumeGasMetering();
        uint32 clientId = handler.createClient(msg_);
        assertEq(
            handler.commitments(
                IBCCommitment.clientStateCommitmentKey(clientId)
            ),
            keccak256(msg_.clientStateBytes)
        );
        assertEq(
            handler.commitments(
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
        handler.registerClient(msg_.clientType, lightClient);
        uint32 clientId = handler.createClient(msg_);
        vm.expectEmit();
        emit IBCClientLib.ClientUpdated(0, 1);
        vm.resumeGasMetering();
        handler.updateClient(
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
        handler.registerClient(msg_.clientType, lightClient);
        uint32 clientId = handler.createClient(msg_);
        vm.resumeGasMetering();
        vm.expectRevert();
        handler.updateClient(
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
        handler.registerClient(msg_.clientType, lightClient);
        uint32 clientId = handler.createClient(msg_);
        vm.resumeGasMetering();
        handler.updateClient(
            IBCMsgs.MsgUpdateClient({
                clientId: clientId,
                clientMessage: clientMessage,
                relayer: relayer
            })
        );
        assertEq(
            handler.commitments(
                IBCCommitment.clientStateCommitmentKey(clientId)
            ),
            keccak256(msg_.clientStateBytes)
        );
        assertEq(
            handler.commitments(
                IBCCommitment.consensusStateCommitmentKey(clientId, 1)
            ),
            keccak256(clientMessage)
        );
    }
}
