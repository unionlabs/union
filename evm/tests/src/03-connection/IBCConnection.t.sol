pragma solidity ^0.8.27;

import "forge-std/Test.sol";

import "../core/UnionTests.sol";
import "../core/IBCHandler.sol";
import "../core/LightClient.sol";

import "../../../contracts/Manager.sol";

contract IBCConnectionTests is UnionTests {
    string public constant CLIENT_TYPE = "zkgm";

    Manager manager;
    TestIBCHandler handler;
    TestLightClient lightClient;
    uint32 clientId;

    function setUp() public {
        (manager, handler) = setupHandler();
        lightClient = new TestLightClient();
        handler.registerClient(CLIENT_TYPE, lightClient);
        clientId = handler.createClient(
            IBCMsgs.MsgCreateClient({
                clientType: CLIENT_TYPE,
                clientStateBytes: hex"CADEBABE",
                consensusStateBytes: hex"DEADC0DE",
                relayer: address(this)
            })
        );
    }

    function test_connectionOpenInit_ok(
        IBCMsgs.MsgConnectionOpenInit calldata msg_
    ) public {
        vm.pauseGasMetering();
        vm.expectEmit();
        emit IBCConnectionLib.ConnectionOpenInit(
            1, msg_.clientId, msg_.counterpartyClientId
        );
        vm.resumeGasMetering();
        handler.connectionOpenInit(msg_);
    }

    function test_connectionOpenInit_commitmentSaved(
        IBCMsgs.MsgConnectionOpenInit calldata msg_
    ) public {
        uint32 connectionId = handler.connectionOpenInit(msg_);
        assertEq(
            handler.commitments(
                IBCCommitment.connectionCommitmentKey(connectionId)
            ),
            keccak256(
                abi.encode(
                    IBCConnection({
                        clientId: msg_.clientId,
                        state: IBCConnectionState.Init,
                        counterpartyClientId: msg_.counterpartyClientId,
                        counterpartyConnectionId: 0
                    })
                )
            )
        );
    }

    function test_connectionOpenTry_ok(
        IBCMsgs.MsgConnectionOpenTry memory msg_
    ) public {
        vm.pauseGasMetering();
        msg_.clientId = clientId;
        lightClient.pushValidMembership();
        vm.expectEmit();
        emit IBCConnectionLib.ConnectionOpenTry(
            1,
            msg_.clientId,
            msg_.counterpartyClientId,
            msg_.counterpartyConnectionId
        );
        vm.resumeGasMetering();
        handler.connectionOpenTry(msg_);
    }

    function test_connectionOpenTry_clientNotFound(
        IBCMsgs.MsgConnectionOpenTry memory msg_
    ) public {
        vm.assume(msg_.clientId != clientId);
        vm.expectRevert(IBCErrors.ErrClientNotFound.selector);
        handler.connectionOpenTry(msg_);
    }

    function test_connectionOpenTry_invalidProof(
        IBCMsgs.MsgConnectionOpenTry memory msg_
    ) public {
        msg_.clientId = clientId;
        vm.expectRevert(IBCErrors.ErrInvalidProof.selector);
        handler.connectionOpenTry(msg_);
    }

    function test_connectionOpenTry_commitmentSaved(
        IBCMsgs.MsgConnectionOpenTry memory msg_
    ) public {
        msg_.clientId = clientId;
        lightClient.pushValidMembership();
        uint32 connectionId = handler.connectionOpenTry(msg_);
        assertEq(
            handler.commitments(
                IBCCommitment.connectionCommitmentKey(connectionId)
            ),
            keccak256(
                abi.encode(
                    IBCConnection({
                        clientId: msg_.clientId,
                        state: IBCConnectionState.TryOpen,
                        counterpartyClientId: msg_.counterpartyClientId,
                        counterpartyConnectionId: msg_.counterpartyConnectionId
                    })
                )
            )
        );
    }

    function test_connectionOpenInitOpenAck_ok(
        IBCMsgs.MsgConnectionOpenInit memory msg_,
        IBCMsgs.MsgConnectionOpenAck memory msgAck_
    ) public {
        vm.pauseGasMetering();
        msg_.clientId = clientId;
        uint32 connectionId = handler.connectionOpenInit(msg_);
        msgAck_.connectionId = connectionId;
        lightClient.pushValidMembership();
        vm.expectEmit();
        emit IBCConnectionLib.ConnectionOpenAck(
            msgAck_.connectionId,
            msg_.clientId,
            msg_.counterpartyClientId,
            // The connectionId of the counterparty must be updated after the ack.
            msgAck_.counterpartyConnectionId
        );
        vm.resumeGasMetering();
        handler.connectionOpenAck(msgAck_);
    }

    function test_connectionOpenInitOpenAck_invalidProof(
        IBCMsgs.MsgConnectionOpenInit memory msg_,
        IBCMsgs.MsgConnectionOpenAck memory msgAck_
    ) public {
        msg_.clientId = clientId;
        uint32 connectionId = handler.connectionOpenInit(msg_);
        msgAck_.connectionId = connectionId;
        vm.expectRevert(IBCErrors.ErrInvalidProof.selector);
        handler.connectionOpenAck(msgAck_);
    }

    function test_connectionOpenInitOpenAck_commitmentSaved(
        IBCMsgs.MsgConnectionOpenInit memory msgInit_,
        IBCMsgs.MsgConnectionOpenAck memory msgAck_
    ) public {
        msgInit_.clientId = clientId;
        uint32 connectionId = handler.connectionOpenInit(msgInit_);
        msgAck_.connectionId = connectionId;
        lightClient.pushValidMembership();
        handler.connectionOpenAck(msgAck_);
        assertEq(
            handler.commitments(
                IBCCommitment.connectionCommitmentKey(connectionId)
            ),
            keccak256(
                abi.encode(
                    IBCConnection({
                        clientId: msgInit_.clientId,
                        state: IBCConnectionState.Open,
                        counterpartyClientId: msgInit_.counterpartyClientId,
                        counterpartyConnectionId: msgAck_.counterpartyConnectionId
                    })
                )
            )
        );
    }

    function test_connectionOpenTryConfirm_ok(
        IBCMsgs.MsgConnectionOpenTry memory msgTry_,
        IBCMsgs.MsgConnectionOpenConfirm memory msgConfirm_
    ) public {
        vm.pauseGasMetering();
        msgTry_.clientId = clientId;
        lightClient.pushValidMembership();
        uint32 connectionId = handler.connectionOpenTry(msgTry_);
        msgConfirm_.connectionId = connectionId;
        lightClient.pushValidMembership();
        vm.expectEmit();
        emit IBCConnectionLib.ConnectionOpenConfirm(
            connectionId,
            msgTry_.clientId,
            msgTry_.counterpartyClientId,
            msgTry_.counterpartyConnectionId
        );
        vm.resumeGasMetering();
        handler.connectionOpenConfirm(msgConfirm_);
    }

    function test_connectionOpenTryConfirm_invalidProof(
        IBCMsgs.MsgConnectionOpenTry memory msgTry_,
        IBCMsgs.MsgConnectionOpenConfirm memory msgConfirm_
    ) public {
        msgTry_.clientId = clientId;
        lightClient.pushValidMembership();
        uint32 connectionId = handler.connectionOpenTry(msgTry_);
        msgConfirm_.connectionId = connectionId;
        vm.expectRevert(IBCErrors.ErrInvalidProof.selector);
        handler.connectionOpenConfirm(msgConfirm_);
    }

    function test_connectionOpenTryConfirm_commitmentSaved(
        IBCMsgs.MsgConnectionOpenTry memory msgTry_,
        IBCMsgs.MsgConnectionOpenConfirm memory msgConfirm_
    ) public {
        msgTry_.clientId = clientId;
        lightClient.pushValidMembership();
        uint32 connectionId = handler.connectionOpenTry(msgTry_);
        msgConfirm_.connectionId = connectionId;
        lightClient.pushValidMembership();
        handler.connectionOpenConfirm(msgConfirm_);
        assertEq(
            handler.commitments(
                IBCCommitment.connectionCommitmentKey(connectionId)
            ),
            keccak256(
                abi.encode(
                    IBCConnection({
                        clientId: msgTry_.clientId,
                        state: IBCConnectionState.Open,
                        counterpartyClientId: msgTry_.counterpartyClientId,
                        counterpartyConnectionId: msgTry_.counterpartyConnectionId
                    })
                )
            )
        );
    }
}
