pragma solidity ^0.8.27;

import "forge-std/Test.sol";

import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

import "../core/UnionTests.sol";
import "../core/IBCHandler.sol";
import "../core/LightClient.sol";

import "../../../contracts/core/25-handler/IBCMsgs.sol";
import "../../../contracts/Manager.sol";

contract IBCClientTests is UnionTests {
    Manager manager;
    TestIBCHandler handler;
    TestLightClient lightClient;

    function setUp() public {
        (manager, handler) = setupHandler();
        lightClient = new TestLightClient();
    }

    function test_registerClient_ok(string calldata typ, address impl) public {
        vm.pauseGasMetering();
        vm.assume(impl != address(0));
        vm.expectEmit();
        emit IBCClientLib.RegisterClient(typ, typ, impl);
        vm.resumeGasMetering();
        handler.registerClient(typ, ILightClient(impl));
    }

    function test_registerClient_alreadyRegistered(
        string calldata typ,
        address impl
    ) public {
        vm.assume(impl != address(0));
        vm.expectEmit();
        emit IBCClientLib.RegisterClient(typ, typ, impl);
        handler.registerClient(typ, ILightClient(impl));
        vm.expectRevert(IBCErrors.ErrClientTypeAlreadyExists.selector);
        handler.registerClient(typ, ILightClient(impl));
    }

    function test_createClient_ok(
        IBCMsgs.MsgCreateClient calldata msg_
    ) public {
        vm.pauseGasMetering();
        handler.registerClient(msg_.clientType, lightClient);
        vm.expectEmit();
        emit IBCClientLib.CreateClient(msg_.clientType, msg_.clientType, 1, "");
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
        emit IBCClientLib.UpdateClient(1, 1);
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
